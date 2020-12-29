use std::ffi::CString;

use crate::input_parser::InputValue;

pub fn exec(
    mut term: Vec<CString>,
    command_string: impl IntoIterator<Item = impl Into<CString>>,
    input_value: &InputValue,
) -> std::convert::Infallible {
    let InputValue {
        has_exact_prefix: _,
        search_string: _,
        args,
        env_vars,
        workind_dir,
    } = input_value;

    if let Some(workind_dir) = &workind_dir {
        nix::unistd::chdir(*workind_dir).expect("chdir failed");
    }

    term.extend(command_string.into_iter().map(Into::into));

    if let Some(args) = args {
        term.extend(
            shlex::split(args)
                .expect("invalid arguments")
                .into_iter()
                .map(|s| CString::new(s).expect("invalid arguments")),
        );
    }

    if let Some(env_vars) = env_vars {
        let env_vars = shlex::split(env_vars)
            .expect("invalid envs")
            .into_iter()
            .map(|s| CString::new(s).expect("invalid envs"))
            .collect::<Vec<_>>();

        let (prog, args) = (&term[0], &term[0..]);
        log::debug!("execvpe: {:?} {:?}", prog, args);
        nix::unistd::execvpe(prog, args, &env_vars).expect("execvpe failed")
    } else {
        let (prog, args) = (&term[0], &term[0..]);
        log::debug!("execvp: {:?} {:?}", prog, args);
        nix::unistd::execvp(prog, args).expect("execvp failed")
    }
}
