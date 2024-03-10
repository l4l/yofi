use std::ffi::CString;

use anyhow::{Context, Result};

use crate::input_parser::InputValue;

pub fn exec(
    term: Option<Vec<CString>>,
    command_string: impl IntoIterator<Item = impl Into<CString>>,
    input_value: &InputValue,
) -> Result<std::convert::Infallible> {
    let InputValue {
        source: _,
        search_string: _,
        args,
        env_vars,
        working_dir,
    } = input_value;

    if let Some(working_dir) = &working_dir {
        nix::unistd::chdir(*working_dir)
            .with_context(|| format!("chdir to {working_dir} failed"))?;
    }

    let command_iter = command_string.into_iter().map(Into::into);

    let command: Vec<_> = if let Some(mut term) = term {
        let mut command = command_iter.fold(Vec::new(), |mut v, item| {
            v.extend(item.into_bytes());
            v
        });
        if let Some(args) = args {
            command.push(b' ');
            command.extend(args.as_bytes());
        }

        term.push(CString::new(command).expect("invalid command"));
        term
    } else {
        let args_iter = args.iter().flat_map(|args| {
            shlex::split(args)
                .expect("invalid arguments")
                .into_iter()
                .map(|s| CString::new(s).expect("invalid arguments"))
        });
        command_iter.chain(args_iter).collect()
    };

    if let Some(env_vars) = env_vars {
        let env_vars = std::env::vars()
            .map(|(k, v)| format!("{}={}", k, v))
            .chain(shlex::split(env_vars).expect("invalid envs"))
            .map(|s| CString::new(s).expect("invalid envs"))
            .collect::<Vec<_>>();

        let (prog, args) = (&command[0], &command[0..]);
        log::debug!("execvpe: {:?} {:?} (envs: {:?})", prog, args, env_vars);
        nix::unistd::execvpe(prog, args, &env_vars).context("execvpe failed")
    } else {
        let (prog, args) = (&command[0], &command[0..]);
        log::debug!("execvp: {:?} {:?}", prog, args);
        nix::unistd::execvp(prog, args).context("execvp failed")
    }
}
