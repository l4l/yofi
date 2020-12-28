use nom::{
    bytes::complete::{tag, take_till},
    combinator::opt,
    error::{Error, ErrorKind},
    IResult,
};
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct InputValue<'a> {
    pub has_exact_prefix: bool,
    pub search_string: &'a str,
    pub args: Option<&'a str>,
    pub env_vars: Option<&'a str>,
    pub workind_dir: Option<&'a str>,
}

fn parse_prefix(input: &str) -> IResult<&str, bool> {
    let (left, parsed) = opt(tag("@"))(input)?;
    if parsed.is_none() {
        return Ok((left, false));
    }

    let (left, _) = take_till(|c| c == ' ')(input)?;
    Ok((left, true))
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
    let res = match nom::regexp::str::re_capture::<Error<_>>(re)(input) {
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

pub fn parser(input: &str) -> IResult<&str, InputValue<'_>> {
    let (input, has_exact_prefix) = parse_prefix(input)?;
    let (mut input, (search_string, mut next_kind)) = parse_command_part(input)?;
    let mut command = InputValue {
        has_exact_prefix,
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
