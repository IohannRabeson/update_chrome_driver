use crate::Version;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1, space0};
use nom::combinator::map_res;
use nom::sequence::tuple;
use nom::IResult;

fn from_dec(input: &str) -> Result<u32, std::num::ParseIntError> {
    input.parse::<u32>()
}

fn parse_dec(input: &str) -> IResult<&str, u32> {
    map_res(digit1, from_dec)(input)
}

/// Parse the version numbers.
/// Example: `89.0.4389.23`
pub fn parse_version_numbers(input: &str) -> IResult<&str, Version> {
    let (input, (major, _, minor, _, build, _, patch)) = tuple((
        parse_dec,
        char('.'),
        parse_dec,
        char('.'),
        parse_dec,
        char('.'),
        parse_dec,
    ))(input)?;

    Ok((
        input,
        Version {
            major,
            minor,
            build,
            patch,
        },
    ))
}

pub fn parse_version_output<'a>(input: &'a str, application: &'a str) -> IResult<&'a str, Version> {
    let (input, _) = tag(application)(input)?;
    let (input, _) = space0(input)?;

    parse_version_numbers(input)
}

/// Parse the version in the output of the command `chromedriver --version`.
/// Example: `ChromeDriver 89.0.4389.23 (61b08ee2c50024bab004e48d2b1b083cdbdac579-refs/branch-heads/4389@{#294})`
pub fn parse_chromedriver_version_output(input: &str) -> IResult<&str, Version> {
    parse_version_output(input, "ChromeDriver")
}

#[cfg(not(target_os = "windows"))]
pub fn parse_chromium_version_output(input: &str) -> IResult<&str, Version> {
    parse_version_output(input, "Google Chrome")
}

#[cfg(target_os = "windows")]
pub fn parse_wmic_version(input: &str) -> IResult<&str, Version> {
    let (input, _) = tag("\r\r\n\r\r\nVersion=")(input)?;

    parse_version_numbers(input)
}

#[cfg(test)]
mod tests {
    use crate::parsers::parse_chromedriver_version_output;
    #[cfg(not(target_os = "windows"))]
    use crate::parsers::parse_chromium_version_output;
    use crate::Version;
    use nom::Finish;
    use test_case::test_case;

    #[test_case("ChromeDriver 89.0.4389.23 (61b08ee2c50024bab004e48d2b1b083cdbdac579-refs/branch-heads/4389@{#294})", Some(Version::new(89, 0, 4389, 23)) ; "basic")]
    fn test_parse_driver_version_output(input: &str, expected: Option<Version>) {
        let result = parse_chromedriver_version_output(input)
            .finish()
            .ok()
            .map(|(_, result)| result);

        assert_eq!(expected, result);
    }

    #[cfg(not(target_os = "windows"))]
    #[test_case("Google Chrome 109.0.5414.87", Some(Version::new(109, 0, 5414, 87)) ; "basic")]
    fn test_parse_browser_version_output(input: &str, expected: Option<Version>) {
        let result = parse_chromium_version_output(input)
            .finish()
            .ok()
            .map(|(_, result)| result);

        assert_eq!(expected, result);
    }
}
