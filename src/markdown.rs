// markdown.rs
pub mod markdown {

    pub type MarkdownText = Vec<MarkdownInline>;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Markdown {
        Heading(usize, MarkdownText),
        OrderedList(Vec<MarkdownText>),
        UnorderedList(Vec<MarkdownText>),
        Line(MarkdownText),
        Codeblock(String),
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum MarkdownInline {
        Link(String, String),
        Image(String, String),
        InlineCode(String),
        Bold(String),
        Italic(String),
        Plaintext(String),
    }

    pub fn markdown(md: &str) -> String {
        match parser::parse_markdown(md) {
            Ok((_, m)) => translator::translate(m),
            Err(_) => String::from("Sorry, this did not seem to work! Maybe your markdown was not well formed, have you hit [Enter] after your last line?"),
        }
    }

    pub(self) mod parser {
        use super::Markdown;
        use super::MarkdownInline;
        use super::MarkdownText;
        use nom::{
            branch::alt,
            bytes::complete::{is_not, tag, take, take_while1},
            character::is_digit,
            combinator::{map, not},
            error::ErrorKind,
            multi::{many0, many1},
            sequence::{delimited, pair, preceded, terminated, tuple},
            Err::Error,
            IResult,
        };

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

        fn parse_code_block(i: &str) -> IResult<&str, &str> {
            delimited(tag("```"), is_not("```"), tag("```"))(i)
        }

        pub fn parse_markdown(i: &str) -> IResult<&str, Vec<Markdown>> {
            many1(alt((
                map(parse_header, |e| Markdown::Heading(e.0, e.1)),
                map(parse_unordered_list, |e| Markdown::UnorderedList(e)),
                map(parse_ordered_list, |e| Markdown::OrderedList(e)),
                map(parse_code_block, |e| Markdown::Codeblock(e.to_string())),
                map(parse_markdown_text, |e| Markdown::Line(e)),
            )))(i)
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_parse_italics() {
                assert_eq!(
                    parse_italics("*here is italic*"),
                    Ok(("", "here is italic"))
                );
                assert_eq!(
                    parse_italics("*here is italic"),
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_italics("here is italic*"),
                    Err(Error(("here is italic*", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_italics("here is italic"),
                    Err(Error(("here is italic", ErrorKind::Tag)))
                );
                assert_eq!(parse_italics("*"), Err(Error(("", ErrorKind::IsNot))));
                assert_eq!(parse_italics("**"), Err(Error(("*", ErrorKind::IsNot))));
                assert_eq!(parse_italics(""), Err(Error(("", ErrorKind::Tag))));
                assert_eq!(
                    parse_italics("**we are doing bold**"),
                    Err(Error(("*we are doing bold**", ErrorKind::IsNot)))
                );
            }

            #[test]
            fn test_parse_boldtext() {
                assert_eq!(parse_boldtext("**here is bold**"), Ok(("", "here is bold")));
                assert_eq!(
                    parse_boldtext("**here is bold"),
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_boldtext("here is bold**"),
                    Err(Error(("here is bold**", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_boldtext("here is bold"),
                    Err(Error(("here is bold", ErrorKind::Tag)))
                );
                assert_eq!(parse_boldtext("****"), Err(Error(("**", ErrorKind::IsNot))));
                assert_eq!(parse_boldtext("**"), Err(Error(("", ErrorKind::IsNot))));
                assert_eq!(parse_boldtext("*"), Err(Error(("*", ErrorKind::Tag))));
                assert_eq!(parse_boldtext(""), Err(Error(("", ErrorKind::Tag))));
                assert_eq!(
                    parse_boldtext("*this is italic*"),
                    Err(Error(("*this is italic*", ErrorKind::Tag)))
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
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_inline_code("here is code`"),
                    Err(Error(("here is code`", ErrorKind::Tag)))
                );
                assert_eq!(parse_inline_code("``"), Err(Error(("`", ErrorKind::IsNot))));
                assert_eq!(parse_inline_code("`"), Err(Error(("", ErrorKind::IsNot))));
                assert_eq!(parse_inline_code(""), Err(Error(("", ErrorKind::Tag))));
            }

            #[test]
            fn test_parse_link() {
                assert_eq!(
                    parse_link("[title](https://www.example.com)"),
                    Ok(("", ("title", "https://www.example.com")))
                );
                assert_eq!(parse_inline_code(""), Err(Error(("", ErrorKind::Tag))));
            }

            #[test]
            fn test_parse_image() {
                assert_eq!(
                    parse_image("![alt text](image.jpg)"),
                    Ok(("", ("alt text", "image.jpg")))
                );
                assert_eq!(parse_inline_code(""), Err(Error(("", ErrorKind::Tag))));
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
                    Err(Error(("*bold babey bold*", ErrorKind::Not)))
                );
                assert_eq!(
                    parse_plaintext("[link babey](and then somewhat)"),
                    Err(Error(("[link babey](and then somewhat)", ErrorKind::Not)))
                );
                assert_eq!(
                    parse_plaintext("`codeblock for bums`"),
                    Err(Error(("`codeblock for bums`", ErrorKind::Not)))
                );
                assert_eq!(
                    parse_plaintext("![ but wait theres more](jk)"),
                    Err(Error(("![ but wait theres more](jk)", ErrorKind::Not)))
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
                    Err(Error(("*here is italic*", ErrorKind::Not)))
                );
                assert_eq!(
                    parse_plaintext("**here is bold**"),
                    Err(Error(("**here is bold**", ErrorKind::Not)))
                );
                assert_eq!(
                    parse_plaintext("`here is code`"),
                    Err(Error(("`here is code`", ErrorKind::Not)))
                );
                assert_eq!(
                    parse_plaintext("[title](https://www.example.com)"),
                    Err(Error(("[title](https://www.example.com)", ErrorKind::Not)))
                );
                assert_eq!(
                    parse_plaintext("![alt text](image.jpg)"),
                    Err(Error(("![alt text](image.jpg)", ErrorKind::Not)))
                );
                assert_eq!(parse_plaintext(""), Err(Error(("", ErrorKind::Eof))));
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
                    parse_markdown_inline("here is some plaintext \n*but what if we italicize?"),
                    Ok((
                        "\n*but what if we italicize?",
                        MarkdownInline::Plaintext(String::from("here is some plaintext "))
                    ))
                );
                assert_eq!(
                    parse_markdown_inline("\n"),
                    Err(Error(("\n", ErrorKind::Not)))
                );
                assert_eq!(parse_markdown_inline(""), Err(Error(("", ErrorKind::Eof))));
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
                    Err(Error(("*but what if we italicize?", ErrorKind::Tag))) // Ok(("*but what if we italicize?", vec![MarkdownInline::Plaintext(String::from("here is some plaintext "))]))
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
                    Err(Error((" ", ErrorKind::TakeWhile1)))
                );
                assert_eq!(parse_header_tag("#"), Err(Error(("", ErrorKind::Tag))));
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
                assert_eq!(parse_header("###h3"), Err(Error(("h3", ErrorKind::Tag))));
                assert_eq!(parse_header("###"), Err(Error(("", ErrorKind::Tag))));
                assert_eq!(parse_header(""), Err(Error(("", ErrorKind::TakeWhile1))));
                assert_eq!(parse_header("#"), Err(Error(("", ErrorKind::Tag))));
                assert_eq!(parse_header("# \n"), Ok(("", (1, vec![]))));
                assert_eq!(parse_header("# test"), Err(Error(("", ErrorKind::Tag))));
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
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_unordered_list_tag("-and some more"),
                    Err(Error(("and some more", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_unordered_list_tag("--"),
                    Err(Error(("-", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_unordered_list_tag(""),
                    Err(Error(("", ErrorKind::Tag)))
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
                    parse_unordered_list_element("- this is an element\n- this is another element\n"),
                    Ok((
                        "- this is another element\n",
                        vec![MarkdownInline::Plaintext(String::from(
                            "this is an element"
                        ))]
                    ))
                );
                assert_eq!(
                    parse_unordered_list_element(""),
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(parse_unordered_list_element("- \n"), Ok(("", vec![])));
                assert_eq!(
                    parse_unordered_list_element("- "),
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_unordered_list_element("- test"),
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_unordered_list_element("-"),
                    Err(Error(("", ErrorKind::Tag)))
                );
            }

            #[test]
            fn test_parse_unordered_list() {
                assert_eq!(
                    parse_unordered_list("- this is an element"),
                    Err(Error(("", ErrorKind::Tag)))
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
                    parse_unordered_list("- this is an element\n- here is another\n"),
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
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_ordered_list_tag("1.and some more"),
                    Err(Error(("and some more", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_ordered_list_tag("1111."),
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_ordered_list_tag(""),
                    Err(Error(("", ErrorKind::TakeWhile1)))
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
                    parse_ordered_list_element("1. this is an element\n1. here is another\n"),
                    Ok((
                        "1. here is another\n",
                        vec![MarkdownInline::Plaintext(String::from(
                            "this is an element"
                        ))]
                    ))
                );
                assert_eq!(
                    parse_ordered_list_element(""),
                    Err(Error(("", ErrorKind::TakeWhile1)))
                );
                assert_eq!(
                    parse_ordered_list_element(""),
                    Err(Error(("", ErrorKind::TakeWhile1)))
                );
                assert_eq!(parse_ordered_list_element("1. \n"), Ok(("", vec![])));
                assert_eq!(
                    parse_ordered_list_element("1. test"),
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_ordered_list_element("1. "),
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_ordered_list_element("1."),
                    Err(Error(("", ErrorKind::Tag)))
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
                    Err(Error(("", ErrorKind::Tag)))
                );
                assert_eq!(
                    parse_ordered_list("1. this is an element\n2. here is another\n"),
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
                    parse_code_block("```bash\n pip install foobar\n```"),
                    Ok(("", "bash\n pip install foobar\n"))
                );
                assert_eq!(
    				parse_code_block("```python\nimport foobar\n\nfoobar.pluralize('word') # returns 'words'\nfoobar.pluralize('goose') # returns 'geese'\nfoobar.singularize('phenomena') # returns 'phenomenon'\n```"),
    				Ok(("", "python\nimport foobar\n\nfoobar.pluralize('word') # returns 'words'\nfoobar.pluralize('goose') # returns 'geese'\nfoobar.singularize('phenomena') # returns 'phenomenon'\n"))
    			);
                // assert_eq!(
                // 	parse_code_block("```bash\n pip `install` foobar\n```"),
                // 	Ok(("", "bash\n pip `install` foobar\n"))
                // );
            }

            fn test_parse_markdown() {
                assert_eq!(
    				parse_markdown("# Foobar\n\nFoobar is a Python library for dealing with word pluralization.\n\n```bash\n pip install foobar\n```\n\n## Installation\n\nUse the package manager [pip](https://pip.pypa.io/en/stable/) to install foobar.\n```python\nimport foobar\n\nfoobar.pluralize('word') # returns 'words'\nfoobar.pluralize('goose') # returns 'geese'\nfoobar.singularize('phenomena') # returns 'phenomenon'\n```"),
    				Ok(("", vec![
    					Markdown::Heading(1, vec![MarkdownInline::Plaintext(String::from("Foobar"))]),
    					Markdown::Line(vec![]),
    					Markdown::Line(vec![MarkdownInline::Plaintext(String::from("Foobar is a Python library for dealing with word pluralization."))]),
    					Markdown::Line(vec![]),
    					Markdown::Codeblock(String::from("bash\n pip install foobar\n")),
    					Markdown::Line(vec![]),
    					Markdown::Heading(2, vec![MarkdownInline::Plaintext(String::from("Installation"))]),
    					Markdown::Line(vec![]),
    					Markdown::Line(vec![
    						MarkdownInline::Plaintext(String::from("Use the package manager ")),
    						MarkdownInline::Link(String::from("pip"), String::from("https://pip.pypa.io/en/stable/")),
    						MarkdownInline::Plaintext(String::from(" to install foobar.")),
    					]),
    					Markdown::Codeblock(String::from("python\nimport foobar\n\nfoobar.pluralize('word') # returns 'words'\nfoobar.pluralize('goose') # returns 'geese'\nfoobar.singularize('phenomena') # returns 'phenomenon'\n")),
    				]))
    			)
            }
        }
    }

    pub(self) mod translator {
        use super::Markdown;
        use super::MarkdownInline;
        use super::MarkdownText;

        fn translate_boldtext(boldtext: String) -> String {
            format!("<b>{}</b>", boldtext)
        }

        fn translate_italic(italic: String) -> String {
            format!("<i>{}</i>", italic)
        }

        fn translate_inline_code(code: String) -> String {
            format!("<code>{}</code>", code)
        }

        fn translate_link(text: String, url: String) -> String {
            format!("<a href=\"{}\">{}</a>", url, text)
        }

        fn translate_image(text: String, url: String) -> String {
            format!("<img src=\"{}\" alt=\"{}\" />", url, text)
        }

        fn translate_list_elements(lines: Vec<MarkdownText>) -> String {
            lines
                .iter()
                .map(|line| format!("<li>{}</li>", translate_text(line.to_vec())))
                .collect::<Vec<String>>()
                .join("")
        }

        fn translate_header(size: usize, text: MarkdownText) -> String {
            format!("<h{}>{}</h{}>", size, translate_text(text), size)
        }

        fn translate_unordered_list(lines: Vec<MarkdownText>) -> String {
            format!("<ul>{}</ul>", translate_list_elements(lines.to_vec()))
        }

        fn translate_ordered_list(lines: Vec<MarkdownText>) -> String {
            format!("<ol>{}</ol>", translate_list_elements(lines.to_vec()))
        }

        fn translate_code(code: MarkdownText) -> String {
            format!("<code>{}</code>", translate_text(code))
        }

        fn translate_codeblock(code: String) -> String {
            format!("<code><pre>{}</pre></code>", code)
        }

        fn translate_line(text: MarkdownText) -> String {
    		let line = translate_text(text);
    		if line.len() > 0   {
    			format!("<p>{}</p>", line)
    		} else {
    			format!("{}", line)
    		}

        }

        fn translate_text(text: MarkdownText) -> String {
            text.iter()
                .map(|part| match part {
                    MarkdownInline::Bold(text) => translate_boldtext(text.to_string()),
                    MarkdownInline::Italic(text) => translate_italic(text.to_string()),
                    MarkdownInline::InlineCode(code) => translate_inline_code(code.to_string()),
                    MarkdownInline::Link(text, url) => {
                        translate_link(text.to_string(), url.to_string())
                    }
                    MarkdownInline::Image(text, url) => {
                        translate_image(text.to_string(), url.to_string())
                    }
                    MarkdownInline::Plaintext(text) => text.to_string(),
                })
                .collect::<Vec<String>>()
                .join("")
        }

        pub fn translate(md: Vec<Markdown>) -> String {
            md.iter()
                .map(|bit| match bit {
                    Markdown::Heading(size, line) => translate_header(*size, line.to_vec()),
                    Markdown::UnorderedList(lines) => translate_unordered_list(lines.to_vec()),
                    Markdown::OrderedList(lines) => translate_ordered_list(lines.to_vec()),
                    Markdown::Codeblock(code) => translate_codeblock(code.to_string()),
                    Markdown::Line(line) => translate_line(line.to_vec()),
                })
                .collect::<Vec<String>>()
                .join("")
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_translate_boldtext() {
                assert_eq!(
                    translate_boldtext(String::from("bold af")),
                    String::from("<b>bold af</b>")
                );
            }

            #[test]
            fn test_translate_italic() {
                assert_eq!(
                    translate_italic(String::from("italic af")),
                    String::from("<i>italic af</i>")
                );
            }

            #[test]
            fn test_translate_inline_code() {
                assert_eq!(
                    translate_inline_code(String::from("code af")),
                    String::from("<code>code af</code>")
                );
            }

            #[test]
            fn test_translate_link() {
                assert_eq!(
                    translate_link(
                        String::from("click me!"),
                        String::from("https://github.com")
                    ),
                    String::from("<a href=\"https://github.com\">click me!</a>")
                );
            }

            #[test]
            fn test_translate_image() {
                assert_eq!(
                    translate_image(String::from("alt text"), String::from("https://github.com")),
                    String::from("<img src=\"https://github.com\" alt=\"alt text\" />")
                );
            }

            #[test]
            fn test_translate_text() {
                let x = translate_text(vec![
                    MarkdownInline::Plaintext(String::from(
                        "Foobar is a Python library for dealing with word pluralization.",
                    )),
                    MarkdownInline::Bold(String::from("bold")),
                    MarkdownInline::Italic(String::from("italic")),
                    MarkdownInline::InlineCode(String::from("code")),
                    MarkdownInline::Link(String::from("tag"), String::from("https://link.com")),
                    MarkdownInline::Image(String::from("tag"), String::from("https://link.com")),
                    MarkdownInline::Plaintext(String::from(". the end!")),
                ]);
                assert_eq!(x, String::from("Foobar is a Python library for dealing with word pluralization.<b>bold</b><i>italic</i><code>code</code><a href=\"https://link.com\">tag</a><img src=\"https://link.com\" alt=\"tag\" />. the end!"));
                let x = translate_text(vec![]);
                assert_eq!(x, String::from(""));
            }

            #[test]
            fn test_translate_header() {
                assert_eq!(
                    translate_header(1, vec![MarkdownInline::Plaintext(String::from("Foobar"))]),
                    String::from("<h1>Foobar</h1>")
                );
            }

            #[test]
            fn test_translate_list_elements() {
                assert_eq!(
                    translate_list_elements(vec![
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                    ]),
                    String::from("<li>Foobar</li><li>Foobar</li><li>Foobar</li><li>Foobar</li>")
                );
            }

            #[test]
            fn test_translate_unordered_list() {
                assert_eq!(
                    translate_unordered_list(vec![
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                    ]),
                    String::from(
                        "<ul><li>Foobar</li><li>Foobar</li><li>Foobar</li><li>Foobar</li></ul>"
                    )
                );
            }

            #[test]
            fn test_translate_ordered_list() {
                assert_eq!(
                    translate_ordered_list(vec![
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                        vec![MarkdownInline::Plaintext(String::from("Foobar"))],
                    ]),
                    String::from(
                        "<ol><li>Foobar</li><li>Foobar</li><li>Foobar</li><li>Foobar</li></ol>"
                    )
                );
            }

            #[test]
            fn test_translate_codeblock() {
                assert_eq!(
    				translate_codeblock(String::from("python\nimport foobar\n\nfoobar.pluralize(\'word\') # returns \'words\'\nfoobar.pluralize(\'goose\') # returns \'geese\'\nfoobar.singularize(\'phenomena\') # returns \'phenomenon\'\n")),
    				String::from("<code><pre>python\nimport foobar\n\nfoobar.pluralize(\'word\') # returns \'words\'\nfoobar.pluralize(\'goose\') # returns \'geese\'\nfoobar.singularize(\'phenomena\') # returns \'phenomenon\'\n</pre></code>")
    			);
            }

            #[test]
            fn test_translate_line() {
                assert_eq!(
                    translate_line(vec![
                        MarkdownInline::Plaintext(String::from("Foobar")),
                        MarkdownInline::Bold(String::from("Foobar")),
                        MarkdownInline::Italic(String::from("Foobar")),
                        MarkdownInline::InlineCode(String::from("Foobar")),
                    ]),
                    String::from("<p>Foobar<b>Foobar</b><i>Foobar</i><code>Foobar</code></p>")
                );
            }
        }
    }
}
