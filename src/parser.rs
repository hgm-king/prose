use crate::Markdown;
use crate::MarkdownInline;
use crate::MarkdownText;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_while1},
    character::is_digit,
    combinator::{map, not},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

pub fn parse_markdown(i: &str) -> IResult<&str, Vec<Markdown>> {
    many1(alt((
        map(parse_header, |e| Markdown::Heading(e.0, e.1)),
        map(parse_unordered_list, |e| Markdown::UnorderedList(e)),
        map(parse_ordered_list, |e| Markdown::OrderedList(e)),
        map(parse_code_block, |e| {
            Markdown::Codeblock(e.0.to_string(), e.1.to_string())
        }),
        map(parse_markdown_text, |e| Markdown::Line(e)),
    )))(i)
}

fn parse_boldtext(i: &str) -> IResult<&str, &str> {
    delimited(tag("**"), is_not("**"), tag("**"))(i)
}

fn parse_italics(i: &str) -> IResult<&str, &str> {
    delimited(tag("*"), is_not("*"), tag("*"))(i)
}

fn parse_inline_code(i: &str) -> IResult<&str, &str> {
    delimited(tag("`"), is_not("`"), tag("`"))(i)
}

fn parse_link(i: &str) -> IResult<&str, (&str, &str)> {
    pair(
        delimited(tag("["), is_not("]"), tag("]")),
        delimited(tag("("), is_not(")"), tag(")")),
    )(i)
}

fn parse_image(i: &str) -> IResult<&str, (&str, &str)> {
    pair(
        delimited(tag("!["), is_not("]"), tag("]")),
        delimited(tag("("), is_not(")"), tag(")")),
    )(i)
}

// we want to match many things that are not any of our specail tags
// but since we have no tools available to match and consume in the negative case (without regex)
// we need to match against our tags, then consume one char
// we repeat this until we run into one of our special characters
// then we join our array of characters into a String
fn parse_plaintext(i: &str) -> IResult<&str, String> {
    map(
        many1(preceded(
            not(alt((tag("*"), tag("`"), tag("["), tag("!["), tag("\n")))),
            take(1u8),
        )),
        |vec| vec.join(""),
    )(i)
}

fn parse_markdown_inline(i: &str) -> IResult<&str, MarkdownInline> {
    alt((
        map(parse_italics, |s: &str| {
            MarkdownInline::Italic(s.to_string())
        }),
        map(parse_inline_code, |s: &str| {
            MarkdownInline::InlineCode(s.to_string())
        }),
        map(parse_boldtext, |s: &str| {
            MarkdownInline::Bold(s.to_string())
        }),
        map(parse_image, |(tag, url): (&str, &str)| {
            MarkdownInline::Image(tag.to_string(), url.to_string())
        }),
        map(parse_link, |(tag, url): (&str, &str)| {
            MarkdownInline::Link(tag.to_string(), url.to_string())
        }),
        map(parse_plaintext, |s| MarkdownInline::Plaintext(s)),
    ))(i)
}

fn parse_markdown_text(i: &str) -> IResult<&str, MarkdownText> {
    terminated(many0(parse_markdown_inline), tag("\n"))(i)
}

// this guy matches the literal character #
fn parse_header_tag(i: &str) -> IResult<&str, usize> {
    map(
        terminated(take_while1(|c| c == '#'), tag(" ")),
        |s: &str| s.len(),
    )(i)
}

// this combines a tuple of the header tag and the rest of the line
fn parse_header(i: &str) -> IResult<&str, (usize, MarkdownText)> {
    tuple((parse_header_tag, parse_markdown_text))(i)
}

fn parse_unordered_list_tag(i: &str) -> IResult<&str, &str> {
    terminated(tag("-"), tag(" "))(i)
}

fn parse_unordered_list_element(i: &str) -> IResult<&str, MarkdownText> {
    preceded(parse_unordered_list_tag, parse_markdown_text)(i)
}

fn parse_unordered_list(i: &str) -> IResult<&str, Vec<MarkdownText>> {
    many1(parse_unordered_list_element)(i)
}

fn parse_ordered_list_tag(i: &str) -> IResult<&str, &str> {
    terminated(
        terminated(take_while1(|d| is_digit(d as u8)), tag(".")),
        tag(" "),
    )(i)
}

fn parse_ordered_list_element(i: &str) -> IResult<&str, MarkdownText> {
    preceded(parse_ordered_list_tag, parse_markdown_text)(i)
}

fn parse_ordered_list(i: &str) -> IResult<&str, Vec<MarkdownText>> {
    many1(parse_ordered_list_element)(i)
}

fn parse_code_block(i: &str) -> IResult<&str, (String, &str)> {
    tuple((parse_code_block_lang, parse_code_block_body))(i)
}

fn parse_code_block_body(i: &str) -> IResult<&str, &str> {
    delimited(tag("\n"), is_not("```"), tag("```"))(i)
}

fn parse_code_block_lang(i: &str) -> IResult<&str, String> {
    alt((
        preceded(tag("```"), parse_plaintext),
        map(tag("```"), |_| "__UNKNOWN__".to_string()),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{error::Error, error::ErrorKind, Err as NomErr};

    #[test]
    fn test_parse_italics() {
        assert_eq!(
            parse_italics("*here is italic*"),
            Ok(("", "here is italic"))
        );
        assert_eq!(
            parse_italics("*here is italic"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italics("here is italic*"),
            Err(NomErr::Error(Error {
                input: "here is italic*",
                code: ErrorKind::Tag,
            }))
        );
        assert_eq!(
            parse_italics("here is italic"),
            Err(NomErr::Error(Error {
                input: "here is italic",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_italics("*"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_italics("**"),
            Err(NomErr::Error(Error {
                input: "*",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_italics(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_italics("**we are doing bold**"),
            Err(NomErr::Error(Error {
                input: "*we are doing bold**",
                code: ErrorKind::IsNot
            }))
        );
    }

    #[test]
    fn test_parse_boldtext() {
        assert_eq!(parse_boldtext("**here is bold**"), Ok(("", "here is bold")));
        assert_eq!(
            parse_boldtext("**here is bold"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_boldtext("here is bold**"),
            Err(NomErr::Error(Error {
                input: "here is bold**",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_boldtext("here is bold"),
            Err(NomErr::Error(Error {
                input: "here is bold",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_boldtext("****"),
            Err(NomErr::Error(Error {
                input: "**",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_boldtext("**"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_boldtext("*"),
            Err(NomErr::Error(Error {
                input: "*",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_boldtext(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_boldtext("*this is italic*"),
            Err(NomErr::Error(Error {
                input: "*this is italic*",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_inline_code() {
        assert_eq!(
            parse_boldtext("**here is bold**\n"),
            Ok(("\n", "here is bold"))
        );
        assert_eq!(
            parse_inline_code("`here is code"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_inline_code("here is code`"),
            Err(NomErr::Error(Error {
                input: "here is code`",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_inline_code("``"),
            Err(NomErr::Error(Error {
                input: "`",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_inline_code("`"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_inline_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_link() {
        assert_eq!(
            parse_link("[title](https://www.example.com)"),
            Ok(("", ("title", "https://www.example.com")))
        );
        assert_eq!(
            parse_inline_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_image() {
        assert_eq!(
            parse_image("![alt text](image.jpg)"),
            Ok(("", ("alt text", "image.jpg")))
        );
        assert_eq!(
            parse_inline_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_plaintext() {
        assert_eq!(
            parse_plaintext("1234567890"),
            Ok(("", String::from("1234567890")))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!"),
            Ok(("", String::from("oh my gosh!")))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!["),
            Ok(("![", String::from("oh my gosh")))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!*"),
            Ok(("*", String::from("oh my gosh!")))
        );
        assert_eq!(
            parse_plaintext("*bold babey bold*"),
            Err(NomErr::Error(Error {
                input: "*bold babey bold*",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("[link babey](and then somewhat)"),
            Err(NomErr::Error(Error {
                input: "[link babey](and then somewhat)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("`codeblock for bums`"),
            Err(NomErr::Error(Error {
                input: "`codeblock for bums`",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("![ but wait theres more](jk)"),
            Err(NomErr::Error(Error {
                input: "![ but wait theres more](jk)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("here is plaintext"),
            Ok(("", String::from("here is plaintext")))
        );
        assert_eq!(
            parse_plaintext("here is plaintext!"),
            Ok(("", String::from("here is plaintext!")))
        );
        assert_eq!(
            parse_plaintext("here is plaintext![image starting"),
            Ok(("![image starting", String::from("here is plaintext")))
        );
        assert_eq!(
            parse_plaintext("here is plaintext\n"),
            Ok(("\n", String::from("here is plaintext")))
        );
        assert_eq!(
            parse_plaintext("*here is italic*"),
            Err(NomErr::Error(Error {
                input: "*here is italic*",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("**here is bold**"),
            Err(NomErr::Error(Error {
                input: "**here is bold**",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("`here is code`"),
            Err(NomErr::Error(Error {
                input: "`here is code`",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("[title](https://www.example.com)"),
            Err(NomErr::Error(Error {
                input: "[title](https://www.example.com)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("![alt text](image.jpg)"),
            Err(NomErr::Error(Error {
                input: "![alt text](image.jpg)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Eof
            }))
        );
    }

    #[test]
    fn test_parse_markdown_inline() {
        assert_eq!(
            parse_markdown_inline("*here is italic*"),
            Ok(("", MarkdownInline::Italic(String::from("here is italic"))))
        );
        assert_eq!(
            parse_markdown_inline("**here is bold**"),
            Ok(("", MarkdownInline::Bold(String::from("here is bold"))))
        );
        assert_eq!(
            parse_markdown_inline("`here is code`"),
            Ok(("", MarkdownInline::InlineCode(String::from("here is code"))))
        );
        assert_eq!(
            parse_markdown_inline("[title](https://www.example.com)"),
            Ok((
                "",
                (MarkdownInline::Link(
                    String::from("title"),
                    String::from("https://www.example.com")
                ))
            ))
        );
        assert_eq!(
            parse_markdown_inline("![alt text](image.jpg)"),
            Ok((
                "",
                (MarkdownInline::Image(String::from("alt text"), String::from("image.jpg")))
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is plaintext!"),
            Ok((
                "",
                MarkdownInline::Plaintext(String::from("here is plaintext!"))
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is some plaintext *but what if we italicize?"),
            Ok((
                "*but what if we italicize?",
                MarkdownInline::Plaintext(String::from("here is some plaintext "))
            ))
        );
        assert_eq!(
            parse_markdown_inline(
                r#"here is some plaintext 
*but what if we italicize?"#
            ),
            Ok((
                "\n*but what if we italicize?",
                MarkdownInline::Plaintext(String::from("here is some plaintext "))
            ))
        );
        assert_eq!(
            parse_markdown_inline("\n"),
            Err(NomErr::Error(Error {
                input: "\n",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_markdown_inline(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Eof
            }))
        );
    }

    #[test]
    fn test_parse_markdown_text() {
        assert_eq!(parse_markdown_text("\n"), Ok(("", vec![])));
        assert_eq!(
            parse_markdown_text("here is some plaintext\n"),
            Ok((
                "",
                vec![MarkdownInline::Plaintext(String::from(
                    "here is some plaintext"
                ))]
            ))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                "",
                vec![
                    MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                    MarkdownInline::Italic(String::from("but what if we italicize?")),
                ]
            ))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?* I guess it doesnt **matter** in my `code`\n"),
            Ok(("", vec![
                MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                MarkdownInline::Italic(String::from("but what if we italicize?")),
                MarkdownInline::Plaintext(String::from(" I guess it doesnt ")),
                MarkdownInline::Bold(String::from("matter")),
                MarkdownInline::Plaintext(String::from(" in my ")),
                MarkdownInline::InlineCode(String::from("code")),
            ]))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                "",
                vec![
                    MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                    MarkdownInline::Italic(String::from("but what if we italicize?")),
                ]
            ))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?"),
            Err(NomErr::Error(Error {
                input: "*but what if we italicize?",
                code: ErrorKind::Tag
            })) // Ok(("*but what if we italicize?", vec![MarkdownInline::Plaintext(String::from("here is some plaintext "))]))
        );
    }

    #[test]
    fn test_parse_header_tag() {
        assert_eq!(parse_header_tag("# "), Ok(("", 1)));
        assert_eq!(parse_header_tag("### "), Ok(("", 3)));
        assert_eq!(parse_header_tag("# h1"), Ok(("h1", 1)));
        assert_eq!(parse_header_tag("# h1"), Ok(("h1", 1)));
        assert_eq!(
            parse_header_tag(" "),
            Err(NomErr::Error(Error {
                input: " ",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_header_tag("#"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_header() {
        assert_eq!(
            parse_header("# h1\n"),
            Ok(("", (1, vec![MarkdownInline::Plaintext(String::from("h1"))])))
        );
        assert_eq!(
            parse_header("## h2\n"),
            Ok(("", (2, vec![MarkdownInline::Plaintext(String::from("h2"))])))
        );
        assert_eq!(
            parse_header("###  h3\n"),
            Ok((
                "",
                (3, vec![MarkdownInline::Plaintext(String::from(" h3"))])
            ))
        );
        assert_eq!(
            parse_header("###h3"),
            Err(NomErr::Error(Error {
                input: "h3",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_header("###"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_header(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_header("#"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(parse_header("# \n"), Ok(("", (1, vec![]))));
        assert_eq!(
            parse_header("# test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list_tag() {
        assert_eq!(parse_unordered_list_tag("- "), Ok(("", "-")));
        assert_eq!(
            parse_unordered_list_tag("- and some more"),
            Ok(("and some more", "-"))
        );
        assert_eq!(
            parse_unordered_list_tag("-"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag("-and some more"),
            Err(NomErr::Error(Error {
                input: "and some more",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag("--"),
            Err(NomErr::Error(Error {
                input: "-",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list_element() {
        assert_eq!(
            parse_unordered_list_element("- this is an element\n"),
            Ok((
                "",
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(
                r#"- this is an element
- this is another element
"#
            ),
            Ok((
                "- this is another element\n",
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(parse_unordered_list_element("- \n"), Ok(("", vec![])));
        assert_eq!(
            parse_unordered_list_element("- "),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_element("- test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_element("-"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list() {
        assert_eq!(
            parse_unordered_list("- this is an element"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list("- this is an element\n"),
            Ok((
                "",
                vec![vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]]
            ))
        );
        assert_eq!(
            parse_unordered_list(
                r#"- this is an element
- here is another
"#
            ),
            Ok((
                "",
                vec![
                    vec![MarkdownInline::Plaintext(String::from(
                        "this is an element"
                    ))],
                    vec![MarkdownInline::Plaintext(String::from("here is another"))]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_ordered_list_tag() {
        assert_eq!(parse_ordered_list_tag("1. "), Ok(("", "1")));
        assert_eq!(parse_ordered_list_tag("1234567. "), Ok(("", "1234567")));
        assert_eq!(
            parse_ordered_list_tag("3. and some more"),
            Ok(("and some more", "3"))
        );
        assert_eq!(
            parse_ordered_list_tag("1"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag("1.and some more"),
            Err(NomErr::Error(Error {
                input: "and some more",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag("1111."),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
    }

    #[test]
    fn test_parse_ordered_list_element() {
        assert_eq!(
            parse_ordered_list_element("1. this is an element\n"),
            Ok((
                "",
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(
                r#"1. this is an element
1. here is another
"#
            ),
            Ok((
                "1. here is another\n",
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(parse_ordered_list_element("1. \n"), Ok(("", vec![])));
        assert_eq!(
            parse_ordered_list_element("1. test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_element("1. "),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_element("1."),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_ordered_list() {
        assert_eq!(
            parse_ordered_list("1. this is an element\n"),
            Ok((
                "",
                vec![vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]]
            ))
        );
        assert_eq!(
            parse_ordered_list("1. test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list(
                r#"1. this is an element
2. here is another
"#
            ),
            Ok((
                "",
                vec![
                    vec!(MarkdownInline::Plaintext(String::from(
                        "this is an element"
                    ))),
                    vec![MarkdownInline::Plaintext(String::from("here is another"))]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_codeblock() {
        assert_eq!(
            parse_code_block(
                r#"```bash
pip install foobar
```"#
            ),
            Ok((
                "",
                (
                    String::from("bash"),
                    r#"pip install foobar
"#
                )
            ))
        );
        assert_eq!(
            parse_code_block(
                r#"```python
import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
```"#
            ),
            Ok((
                "",
                (
                    String::from("python"),
                    r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                )
            ))
        );
        // assert_eq!(
        // 	parse_code_block("```bash\n pip `install` foobar\n```"),
        // 	Ok(("", "bash\n pip `install` foobar\n"))
        // );
    }

    #[test]
    fn test_parse_codeblock_no_language() {
        assert_eq!(
            parse_code_block(
                r#"```
pip install foobar
```"#
            ),
            Ok((
                "",
                (
                    String::from("__UNKNOWN__"),
                    r#"pip install foobar
"#
                )
            ))
        );
    }

    #[test]
    fn test_parse_markdown() {
        assert_eq!(
            parse_markdown(
                r#"# Foobar

Foobar is a Python library for dealing with word pluralization.

```bash
pip install foobar
```
## Installation

Use the package manager [pip](https://pip.pypa.io/en/stable/) to install foobar.
```python
import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
```"#
            ),
            Ok((
                "",
                vec![
                    Markdown::Heading(1, vec![MarkdownInline::Plaintext(String::from("Foobar"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![MarkdownInline::Plaintext(String::from(
                        "Foobar is a Python library for dealing with word pluralization."
                    ))]),
                    Markdown::Line(vec![]),
                    Markdown::Codeblock(String::from("bash"), String::from("pip install foobar\n")),
                    Markdown::Line(vec![]),
                    Markdown::Heading(
                        2,
                        vec![MarkdownInline::Plaintext(String::from("Installation"))]
                    ),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![
                        MarkdownInline::Plaintext(String::from("Use the package manager ")),
                        MarkdownInline::Link(
                            String::from("pip"),
                            String::from("https://pip.pypa.io/en/stable/")
                        ),
                        MarkdownInline::Plaintext(String::from(" to install foobar.")),
                    ]),
                    Markdown::Codeblock(
                        String::from("python"),
                        String::from(
                            r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                        )
                    ),
                ]
            ))
        )
    }
}
