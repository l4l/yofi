use std::ffi::CString;

use crate::input_parser::InputValue;

pub fn exec(
    term: Option<Vec<CString>>,
    command_string: impl IntoIterator<Item = impl Into<CString>>,
    input_value: &InputValue,
) -> std::convert::Infallible {
    let InputValue {
        search_string: _,
        args,
        env_vars,
        workind_dir,
    } = input_value;

    if let Some(workind_dir) = &workind_dir {
        nix::unistd::chdir(*workind_dir).expect("chdir failed");
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

        term.push(CString::new(command).unwrap());
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
        let env_vars = shlex::split(env_vars)
            .expect("invalid envs")
            .into_iter()
            .map(|s| CString::new(s).expect("invalid envs"))
            .collect::<Vec<_>>();

        let (prog, args) = (&command[0], &command[0..]);
        log::debug!("execvpe: {:?} {:?}", prog, args);
        nix::unistd::execvpe(prog, args, &env_vars).expect("execvpe failed")
    } else {
        let (prog, args) = (&command[0], &command[0..]);
        log::debug!("execvp: {:?} {:?}", prog, args);
        nix::unistd::execvp(prog, args).expect("execvp failed")
    }
}
