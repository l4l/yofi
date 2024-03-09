use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result};
use log::LevelFilter;

use yofi::{config, desktop, mode, prog_name, state, window};

fn setup_logger(level: LevelFilter, args: &Args) -> Result<()> {
    let dispatcher = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout());

    let dispatcher = if let Some(log_file) = &args.log_file {
        dispatcher.chain(fern::log_file(log_file)?)
    } else {
        dispatcher
    };

    let dispatcher = if !args.disable_syslog_logger {
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
            Ok(writer) => dispatcher.chain(writer),
        }
    } else {
        dispatcher
    };

    dispatcher.apply()?;
    Ok(())
}

use argh::FromArgs;

/// Minimalistic menu launcher
#[derive(FromArgs)]
struct Args {
    /// increases log verbosity
    #[argh(switch, short = 'v')]
    verbose: bool,
    /// reduces log verbosity
    #[argh(switch, short = 'q')]
    quiet: bool,
    /// prompt to be displayed as a hint
    #[argh(option, short = 'p')]
    prompt: Option<String>,
    /// password mode, i.e all characters displayed as `*`
    #[argh(switch)]
    password: bool,
    /// path to log file
    #[argh(option)]
    log_file: Option<PathBuf>,
    /// disable syslog
    #[argh(switch, short = 'd')]
    disable_syslog_logger: bool,
    /// path to config file
    #[argh(option)]
    config_file: Option<PathBuf>,
    /// mode to operate
    #[argh(subcommand)]
    mode: Option<ModeArg>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum ModeArg {
    Apps(AppsMode),
    Binapps(BinappsMode),
    Dialog(DialogMode),
}

/// Desktop apps mode
#[derive(FromArgs)]
#[argh(subcommand, name = "apps")]
struct AppsMode {
    /// optional path to ignored desktop files.
    #[argh(option)]
    blacklist: Option<PathBuf>,
    /// flag for listing desktop files for entries names.
    #[argh(switch, short = 'l')]
    list: bool,
}

/// Binaries mode
#[derive(FromArgs)]
#[argh(subcommand, name = "binapps")]
struct BinappsMode {}

/// Dialog mode
#[derive(FromArgs)]
#[argh(subcommand, name = "dialog")]
struct DialogMode {}

impl ModeArg {
    fn try_default() -> Result<Self> {
        let blacklist = xdg::BaseDirectories::with_prefix(prog_name!())
            .context("failed to get xdg dirs")?
            .place_config_file("blacklist")
            .map_err(|e| log::error!("failed to create default blacklist file: {}", e))
            .ok();
        Ok(ModeArg::Apps(AppsMode {
            blacklist,
            list: false,
        }))
    }
}

fn main_inner() -> Result<()> {
    let mut args: Args = argh::from_env();

    let mut config = config::Config::load(args.config_file.take())?;

    let log_level = match (args.verbose, args.quiet) {
        (true, true) => panic!("either verbose or quiet could be specified, not both"),
        (true, _) => LevelFilter::Debug,
        (_, true) => LevelFilter::Warn,
        (false, false) => LevelFilter::Info,
    };

    setup_logger(log_level, &args)?;

    if let Some(prompt) = args.prompt.take() {
        config.override_prompt(prompt);
    }

    if args.password {
        config.override_password();
    }

    let default_mode_arg;
    let mode_arg = match args.mode.as_ref() {
        Some(m) => m,
        None => {
            default_mode_arg = ModeArg::try_default()?;
            &default_mode_arg
        }
    };
    let cmd = match mode_arg {
        ModeArg::Apps(AppsMode { blacklist, list }) => {
            let blacklist_filter = blacklist
                .as_ref()
                .and_then(|file| {
                    let entries = std::fs::read_to_string(file)
                        .map_err(|e| log::debug!("cannot read blacklist file {:?}: {}", file, e))
                        .ok()?
                        .lines()
                        .map(std::ffi::OsString::from)
                        .collect::<HashSet<_>>();

                    Some(Box::new(move |e: &_| !entries.contains(e)) as Box<dyn Fn(&_) -> bool>)
                })
                .unwrap_or_else(|| Box::new(|_| true));

            let entries = desktop::Traverser::new(config.param(), blacklist_filter)
                .context("cannot load desktop file traverser")?
                .find_entries();

            if *list {
                for e in entries {
                    println!("{}: {}", e.entry.name, e.desktop_fname);
                }
                return Ok(());
            }

            mode::Mode::apps(entries, config.terminal_command())
        }
        ModeArg::Binapps(BinappsMode {}) => {
            config.disable_icons();
            mode::Mode::bins(config.terminal_command())
        }
        ModeArg::Dialog(DialogMode {}) => mode::Mode::dialog()?,
    };

    let state = state::State::new(cmd);
    let (mut window, mut event_loop) =
        window::Window::new(config, state).context("unable create a window")?;

    while !window.asked_exit() {
        event_loop.dispatch(None, &mut window)?;
        if let Some(err) = window.take_error() {
            return Err(err);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let res = std::panic::catch_unwind(main_inner);

    match res {
        Ok(res) => res,
        Err(err) => {
            let msg = if let Some(msg) = err.downcast_ref::<String>() {
                msg.clone()
            } else if let Some(msg) = err.downcast_ref::<&str>() {
                msg.to_string()
            } else {
                "unknown panic".into()
            };

            let _ = std::process::Command::new("timeout")
                .args([
                    "1s",
                    "notify-send",
                    concat!("--app-name=", prog_name!()),
                    concat!(prog_name!(), " has panicked!"),
                    &msg,
                ])
                .status();

            log::error!("panic: {}", msg);

            Err(anyhow::Error::msg(msg))
        }
    }
}
