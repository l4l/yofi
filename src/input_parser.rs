use nom::{
    error::{Error, ErrorKind},
    IResult,
};
use once_cell::sync::OnceCell;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct InputValue<'a> {
    pub source: &'a str,
    pub search_string: &'a str,
    pub args: Option<&'a str>,
    pub env_vars: Option<&'a str>,
    pub workind_dir: Option<&'a str>,
}

impl InputValue<'static> {
    pub fn empty() -> Self {
        InputValue {
            source: "",
            search_string: "",
            args: None,
            env_vars: None,
            workind_dir: None,
        }
    }
}

enum NextValueKind {
    Args,
    EnvVars,
    WorkingDir,
}

static SEPARATOR_REGEX: OnceCell<regex::Regex> = OnceCell::new();

fn parse_command_part(input: &str) -> IResult<&str, (&str, Option<NextValueKind>)> {
    let re = SEPARATOR_REGEX
        .get_or_init(|| regex::Regex::new(r"(.*?)(!!|#|~)").unwrap())
        .clone();
    let res = match nom_regex::str::re_capture::<Error<_>>(re)(input) {
        Ok((left, matches)) => {
            let parsed = matches[1];
            let kind = match matches[2] {
                "!!" => Some(NextValueKind::Args),
                "#" => Some(NextValueKind::EnvVars),
                "~" => Some(NextValueKind::WorkingDir),
                _ => None,
            };
            (left, (parsed, kind))
        }
        Err(nom::Err::Error(Error {
            input,
            code: ErrorKind::RegexpCapture,
        })) => ("", (input, None)),
        Err(err) => return Err(err),
    };
    Ok(res)
}

pub fn parser(source: &str) -> IResult<&str, InputValue<'_>> {
    let (mut input, (search_string, mut next_kind)) = parse_command_part(source)?;
    let mut command = InputValue {
        source,
        search_string,
        args: None,
        env_vars: None,
        workind_dir: None,
    };

    while let Some(kind) = next_kind.take() {
        let (left, (cmd, new_kind)) = parse_command_part(input)?;

        match kind {
            NextValueKind::Args => command.args = Some(cmd),
            NextValueKind::EnvVars => command.env_vars = Some(cmd),
            NextValueKind::WorkingDir => command.workind_dir = Some(cmd),
        }

        input = left;
        next_kind = new_kind;
    }

    Ok((input, command))
}

#[cfg(test)]
mod tests {
    use super::{parser, InputValue};

    use quickcheck_macros::quickcheck;
    use test_case::test_case;

    #[test_case("", InputValue::empty(); "empty string")]
    #[test_case("qwdqwd asd asd", InputValue {
        search_string: "qwdqwd asd asd",
        ..InputValue::empty()
    }; "search string")]
    #[test_case("qwdqwd asd asd", InputValue {
        search_string: "qwdqwd asd asd",
        ..InputValue::empty()
    }; "search string with exact prefix")]
    #[test_case("qwdqwd!!asd#dsa", InputValue {
        search_string: "qwdqwd",
        args: Some("asd"),
        env_vars: Some("dsa"),
        ..InputValue::empty()
    }; "search string with args then env")]
    #[test_case("qwdqwd#dsa!!asd", InputValue {
        search_string: "qwdqwd",
        args: Some("asd"),
        env_vars: Some("dsa"),
        ..InputValue::empty()
    }; "search string with env then args")]
    #[test_case("qwdqwd~zx,c#qwe !!asd", InputValue {
        search_string: "qwdqwd",
        args: Some("asd"),
        env_vars: Some("qwe "),
        workind_dir: Some("zx,c"),
        ..InputValue::empty()
    }; "search string with working dir then env then args")]
    #[test_case("#qwe~zx,c!!asd", InputValue {
        search_string: "",
        args: Some("asd"),
        env_vars: Some("qwe"),
        workind_dir: Some("zx,c"),
        ..InputValue::empty()
    }; "all but search string")]
    #[test_case("ffx!!--new-instance#MOZ_ENABLE_WAYLAND=1~/run/user/1000", InputValue {
        search_string: "ffx",
        args: Some("--new-instance"),
        env_vars: Some("MOZ_ENABLE_WAYLAND=1"),
        workind_dir: Some("/run/user/1000"),
        ..InputValue::empty()
    }; "with all params")]
    fn test_parse(input: &str, input_value: InputValue) {
        assert_eq!(
            parser(input),
            Ok((
                "",
                InputValue {
                    source: input,
                    ..input_value
                }
            ))
        );
    }

    #[quickcheck]
    fn test_parse_all(input: String) {
        let (left, _) = parser(&input).unwrap();
        assert_eq!(left, "");
    }
}
