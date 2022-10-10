use std::collections::HashSet;
use std::path::PathBuf;

use log::LevelFilter;
use sctk::{
    environment::SimpleGlobal,
    reexports::{
        calloop,
        protocols::wlr::unstable::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1,
    },
    WaylandSource,
};
use structopt::{clap::ArgGroup, StructOpt};

pub use color::Color;
pub use desktop::Entry as DesktopEntry;
pub use draw::DrawTarget;

mod color;
mod config;
mod desktop;
mod draw;
mod exec;
mod font;
mod icon;
mod input;
mod input_parser;
mod mode;
mod state;
mod style;
mod surface;
mod usage_cache;

sctk::default_environment!(Env,
    desktop,
    fields = [
        layer_shell: SimpleGlobal<ZwlrLayerShellV1>,
    ],
    singles = [
        ZwlrLayerShellV1 => layer_shell
    ]
);

#[macro_export]
macro_rules! prog_name {
    () => {
        "yofi"
    };
}

fn setup_logger(level: LevelFilter, args: &Args) {
    let dispatcher = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout());

    let dispatcher = if let Some(log_file) = &args.log_file {
        dispatcher.chain(fern::log_file(log_file).unwrap())
    } else {
        dispatcher
    };

    let dispatcher = if !args.disable_syslog_logger {
        use log::Log;
        let formatter = syslog::Formatter3164 {
            facility: syslog::Facility::LOG_USER,
            hostname: None,
            process: prog_name!().into(),
            pid: 0,
        };

        match syslog::unix(formatter) {
            Err(e) => {
                eprintln!("cann't connect to syslog: {:?}", e);
                dispatcher
            }
            Ok(writer) => {
                let syslog_logger = syslog::BasicLogger::new(writer);
                dispatcher.chain(fern::Output::call(move |record| syslog_logger.log(record)))
            }
        }
    } else {
        dispatcher
    };

    dispatcher.apply().unwrap();
}

#[derive(StructOpt)]
#[structopt(
    group = ArgGroup::with_name("verbosity").multiple(false),
)]
struct Args {
    #[structopt(short, long, group = "verbosity")]
    verbose: bool,
    #[structopt(short, long, group = "verbosity")]
    quiet: bool,
    #[structopt(short, long)]
    prompt: Option<String>,
    #[structopt(long)]
    password: bool,
    #[structopt(long)]
    log_file: Option<PathBuf>,
    #[structopt(short, long = "disable-logger")]
    disable_syslog_logger: bool,
    #[structopt(long)]
    config_file: Option<PathBuf>,
    #[structopt(subcommand)]
    mode: Option<ModeArg>,
}

#[derive(StructOpt)]
enum ModeArg {
    Apps {
        /// Optional path to ignored desktop files.
        blacklist: Option<PathBuf>,
        /// Flag for listing desktop files for entries names.
        #[structopt(short, long)]
        list: bool,
    },
    Binapps,
    Dialog,
}

impl Default for ModeArg {
    fn default() -> Self {
        let file = xdg::BaseDirectories::with_prefix(prog_name!())
            .expect("failed to get xdg dirs")
            .place_config_file("blacklist")
            .expect("failed to crate default blacklist");
        ModeArg::Apps {
            blacklist: Some(file),
            list: false,
        }
    }
}

fn main_inner() {
    let mut args = Args::from_args();

    let mut config = config::Config::load(args.config_file.take());

    let log_level = match (args.verbose, args.quiet) {
        (true, _) => LevelFilter::Debug,
        (_, true) => LevelFilter::Warn,
        _ => LevelFilter::Info,
    };

    setup_logger(log_level, &args);

    let (env, display, queue) =
        sctk::new_default_environment!(Env, desktop, fields = [layer_shell: SimpleGlobal::new()])
            .expect("Initial roundtrip failed!");
    let mut event_loop = calloop::EventLoop::try_new().unwrap();

    let mut surface = surface::Surface::new(&env, config.param());

    let (_input, key_stream) = input::InputHandler::new(&env, &event_loop);

    if let Some(prompt) = args.prompt.take() {
        config.set_prompt(prompt);
    }

    if args.password {
        config.set_password();
    }

    let cmd = match args.mode.take().unwrap_or_default() {
        ModeArg::Apps { blacklist, list } => {
            let blacklist_filter = blacklist
                .and_then(|file| {
                    let entries = std::fs::read_to_string(&file)
                        .map_err(|e| log::warn!("cannot read blacklist file {:?}: {}", file, e))
                        .ok()?
                        .lines()
                        .map(std::ffi::OsString::from)
                        .collect::<HashSet<_>>();

                    Some(Box::new(move |e: &_| !entries.contains(e)) as Box<dyn Fn(&_) -> bool>)
                })
                .unwrap_or_else(|| Box::new(|_| true));

            let entries = desktop::Traverser::new(config.param(), blacklist_filter)
                .expect("cannot load desktop file traverser")
                .find_entries();

            if list {
                for e in entries {
                    println!("{}: {}", e.entry.name, e.desktop_fname);
                }
                return;
            }

            mode::Mode::apps(entries, config.terminal_command())
        }
        ModeArg::Binapps => {
            config.disable_icons();
            mode::Mode::bins(config.terminal_command())
        }
        ModeArg::Dialog => mode::Mode::dialog(),
    };

    let mut state = state::State::new(cmd);

    if !env.get_shell().unwrap().needs_configure() {
        draw(&mut state, &config, &mut surface);
    }

    WaylandSource::new(queue)
        .quick_insert(event_loop.handle())
        .unwrap();

    loop {
        let mut should_redraw = false;
        for event in key_stream.try_iter() {
            should_redraw = true;

            if state.process_event(event) {
                return;
            }
        }

        match surface.handle_events() {
            surface::EventStatus::Finished => break,
            surface::EventStatus::ShouldRedraw => should_redraw = true,
            surface::EventStatus::Idle => {}
        };

        if should_redraw {
            draw(&mut state, &config, &mut surface);
        }

        display.flush().unwrap();
        event_loop.dispatch(None, &mut ()).unwrap();
    }
}

fn main() {
    let res = std::panic::catch_unwind(main_inner);

    if let Err(err) = res {
        let msg = if let Some(msg) = err.downcast_ref::<String>() {
            msg.as_str()
        } else if let Some(msg) = err.downcast_ref::<&str>() {
            msg
        } else {
            "unknown panic"
        };

        let _ = std::process::Command::new("notify-send")
            .args(&[
                concat!("--app-name=", prog_name!()),
                concat!(prog_name!(), " has panicked!"),
                msg,
            ])
            .status();

        log::error!("panic: {}", msg);

        std::process::exit(42);
    }
}

fn draw(state: &mut state::State, config: &config::Config, surface: &mut surface::Surface) {
    use std::iter::once;

    state.process_entries();

    let (tx, rx) = oneshot::channel();

    let background = draw::Widget::background(config.param());
    let input_widget = draw::Widget::input_text(state.raw_input(), config.param());
    let list_view_widget = draw::Widget::list_view(
        state.processed_entries(),
        state.skip_offset(),
        state.selected_item(),
        tx,
        config.param(),
    );

    surface.redraw(
        once(background)
            .chain(once(input_widget))
            .chain(once(list_view_widget)),
    );

    state.update_skip_offset(rx.recv().unwrap());
}
