use nom::branch::alt;
use nom::bytes::complete::{escaped, tag, tag_no_case};
use nom::character::complete::{alpha1, digit0, multispace0, multispace1, none_of};
use nom::combinator::{map, recognize};
use nom::{IResult, Parser};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};

pub enum UsingOption {
    File(String),
    Worksheet(String),
    Range(String),
}

pub fn parse_option(input: &str) -> IResult<&str, UsingOption> {
    parse_with_spaces(alt((
        parse_filename_option,
        parse_worksheet_option,
        parse_range_option,
        ))).parse(input)
}

fn parse_filename_option(input: &str) -> IResult<&str, UsingOption> {
    let option = alt((tag_no_case("FILENAME"), tag_no_case("FILE")));
    let path = parse_quoted;

    map(separated_pair(option, multispace1, path),
        |(_, p)| UsingOption::File(p.to_string()))(input)
}

fn parse_worksheet_option(input: &str) -> IResult<&str, UsingOption> {
    let option = alt((tag_no_case("WORKSHEET"), tag_no_case("SHEET")));
    let name = parse_quoted;

    map(separated_pair(option, multispace1, name),
        |(_, n)| UsingOption::Worksheet(n.to_string()))(input)
}

fn parse_range_option(input: &str) -> IResult<&str, UsingOption> {
    let option = tag_no_case("RANGE");

    let range = recognize(
        tuple((alpha1, digit0, tag(":"), alpha1, digit0))
    );

    let value = preceded(
        tag("'"), terminated(range, tag("'")));

    map(separated_pair(option, multispace1, value),
        |t: (&str, &str)| UsingOption::Range(t.1.to_string()))(input)
}

fn parse_with_spaces<'a, T>(parser: impl Parser<&'a str, T, nom::error::Error<&'a str>>)
    -> impl Parser<&'a str, T, nom::error::Error<&'a str>> {
    preceded(multispace0, terminated(parser, multispace0))
}

fn parse_quoted(input: &str) -> IResult<&str, &str> {
    let esc = escaped(none_of("\\\'"), '\\', tag("'"));
    let esc_or_empty = alt((esc, tag("")));
    delimited(tag("'"), esc_or_empty, tag("'"))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_option_produces_filename() {
        let (output, option) = parse_filename_option("FILE '/path/to/file.xls'").unwrap();

        assert_eq!(output, "");
        match option {
            UsingOption::File(filename) => assert_eq!(filename, "/path/to/file.xls"),
            _ => panic!("Expected file option")
        }
    }

    #[test]
    fn parse_filename_option_produces_filename() {
        let (output, option) = parse_filename_option("FILENAME '/path/to/file.xls'").unwrap();

        assert_eq!(output, "");
        match option {
            UsingOption::File(filename) => assert_eq!(filename, "/path/to/file.xls"),
            _ => panic!("Expected file option")
        }
    }

    #[test]
    fn parse_worksheet_option_produces_sheet_name() {
        let (output, option) = parse_worksheet_option("WORKSHEET 'Sheet 1'").unwrap();

        assert_eq!(output, "");
        match option {
            UsingOption::Worksheet(sheet_name) => assert_eq!(sheet_name, "Sheet 1"),
            _ => panic!("Expected worksheet option")
        }
    }

    #[test]
    fn parse_sheet_option_produces_sheet_name() {
        let (output, option) = parse_worksheet_option("SHEET 'Sheet 1'").unwrap();

        assert_eq!(output, "");
        match option {
            UsingOption::Worksheet(sheet_name) => assert_eq!(sheet_name, "Sheet 1"),
            _ => panic!("Expected worksheet option")
        }
    }

    #[test]
    fn parse_range_option_produces_range() {
        let (output, option) = parse_range_option("RANGE 'A1:ZZ99'").unwrap();

        assert_eq!(output, "");
        match option {
            UsingOption::Range(range) => {
                assert_eq!(range, "A1:ZZ99");
            },
            _ => panic!("Expected range option")
        }
    }
}
