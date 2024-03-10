use once_cell::sync::OnceCell;
use regex::Regex;

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

static SEPARATOR_REGEX: OnceCell<Regex> = OnceCell::new();

fn parse_command_part(input: &str) -> (&str, (&str, Option<NextValueKind>)) {
    let re = SEPARATOR_REGEX
        .get_or_init(|| Regex::new(r"(!!|#|~)").unwrap())
        .clone();
    let Some(cap) = re.captures(input) else {
        return ("", (input, None));
    };
    let m = cap.get(0).unwrap();
    let bang = cap.get(1).unwrap().as_str();
    let kind = match bang {
        "!!" => Some(NextValueKind::Args),
        "#" => Some(NextValueKind::EnvVars),
        "~" => Some(NextValueKind::WorkingDir),
        s => panic!("regex bug: unexpected bang match {}", s),
    };

    (&input[m.end()..], (&input[..m.start()], kind))
}

pub fn parse(source: &str) -> InputValue<'_> {
    let (mut input, (search_string, mut next_kind)) = parse_command_part(source);
    let mut command = InputValue {
        source,
        search_string,
        args: None,
        env_vars: None,
        workind_dir: None,
    };

    while let Some(kind) = next_kind.take() {
        let (left, (cmd, new_kind)) = parse_command_part(input);

        match kind {
            NextValueKind::Args => command.args = Some(cmd),
            NextValueKind::EnvVars => command.env_vars = Some(cmd),
            NextValueKind::WorkingDir => command.workind_dir = Some(cmd),
        }

        input = left;
        next_kind = new_kind;
    }

    debug_assert!(input.is_empty(), "trailing input: {}", input);

    command
}

#[cfg(test)]
mod tests {
    use super::{parse, InputValue};

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
            parse(input),
            InputValue {
                source: input,
                ..input_value
            }
        );
    }

    #[quickcheck]
    fn test_parse_all(input: String) {
        parse(&input);
    }
}
