use std::path::{Path, PathBuf};

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

pub use desktop::{Entry as DesktopEntry, XDG_DIRS};

mod config;
mod desktop;
mod draw;
mod input;
mod mode;
mod state;
mod surface;

sctk::default_environment!(Env,
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

const DEFAULT_LOG_PATH: &str = concat!(concat!("/tmp/", prog_name!()), ".log");

fn setup_logger(level: LevelFilter, log_file: impl AsRef<Path>) {
    fern::Dispatch::new()
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
        .chain(fern::log_file(log_file).unwrap())
        .apply()
        .unwrap();
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
    #[structopt(long)]
    log_file: Option<PathBuf>,
    #[structopt(long)]
    config_file: Option<PathBuf>,
    #[structopt(subcommand)]
    mode: Option<ModeArg>,
}

#[derive(StructOpt)]
enum ModeArg {
    Apps,
    Dialog,
}

impl Default for ModeArg {
    fn default() -> Self {
        ModeArg::Apps
    }
}

fn main() {
    let mut args = Args::from_args();

    let config = config::Config::load(args.config_file.take());

    setup_logger(
        match (args.verbose, args.quiet) {
            (true, _) => LevelFilter::Debug,
            (_, true) => LevelFilter::Warn,
            _ => LevelFilter::Info,
        },
        args.log_file.unwrap_or_else(|| DEFAULT_LOG_PATH.into()),
    );

    let (env, display, queue) =
        sctk::new_default_environment!(Env, fields = [layer_shell: SimpleGlobal::new()])
            .expect("Initial roundtrip failed!");
    let mut event_loop = calloop::EventLoop::<()>::new().unwrap();

    let mut surface = surface::Surface::new(&env, config.param());

    let (_input, key_stream) = input::InputHandler::new(&env, &event_loop);

    WaylandSource::new(queue)
        .quick_insert(event_loop.handle())
        .unwrap();

    let cmd = match args.mode.take().unwrap_or_default() {
        ModeArg::Apps => mode::Mode::apps(desktop::find_entries(), config.terminal_command()),
        ModeArg::Dialog => mode::Mode::dialog(),
    };

    let mut state = state::State::new(cmd);

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
            use std::iter::once;

            state.process_entries();

            let background = draw::Widget::background(config.param());
            let input_widget = draw::Widget::input_text(&state.input_buf(), config.param());
            let list_view_widget = draw::Widget::list_view(
                state.processed_entries(),
                state.selected_item(),
                config.param(),
            );

            surface.redraw(
                once(background)
                    .chain(once(input_widget))
                    .chain(once(list_view_widget)),
            );
        }

        display.flush().unwrap();
        event_loop.dispatch(None, &mut ()).unwrap();
    }
}
