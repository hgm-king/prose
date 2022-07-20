use crate::Markdown;
use crate::MarkdownInline;
use crate::MarkdownText;

pub fn translate(md: Vec<Markdown>) -> String {
    md.iter()
        .map(|bit| match bit {
            Markdown::Heading(size, line) => translate_header(*size, line.to_vec()),
            Markdown::UnorderedList(lines) => translate_unordered_list(lines.to_vec()),
            Markdown::OrderedList(lines) => translate_ordered_list(lines.to_vec()),
            Markdown::Codeblock(lang, code) => {
                translate_codeblock(lang.to_string(), code.to_string())
            }
            Markdown::Line(line) => translate_line(line.to_vec()),
        })
        .collect::<Vec<String>>()
        .join("")
}

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

// fn translate_code(code: MarkdownText) -> String {
//     format!("<code>{}</code>", translate_text(code))
// }

fn translate_codeblock(lang: String, code: String) -> String {
    format!("<pre><code class=\"lang-{}\">{}</code></pre>", lang, code)
}

fn translate_line(text: MarkdownText) -> String {
    let line = translate_text(text);
    if line.len() > 0 {
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
            MarkdownInline::Link(text, url) => translate_link(text.to_string(), url.to_string()),
            MarkdownInline::Image(text, url) => translate_image(text.to_string(), url.to_string()),
            MarkdownInline::Plaintext(text) => text.to_string(),
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
            String::from("<ul><li>Foobar</li><li>Foobar</li><li>Foobar</li><li>Foobar</li></ul>")
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
            String::from("<ol><li>Foobar</li><li>Foobar</li><li>Foobar</li><li>Foobar</li></ol>")
        );
    }

    #[test]
    fn test_translate_codeblock() {
        assert_eq!(
            translate_codeblock(
                String::from("python"),
                String::from(
                    r#"
import foobar

foobar.pluralize(\'word\') # returns \'words\'
foobar.pluralize(\'goose\') # returns \'geese\'
foobar.singularize(\'phenomena\') # returns \'phenomenon\'
"#
                )
            ),
            String::from(
                r#"<pre><code class="lang-python">
import foobar

foobar.pluralize(\'word\') # returns \'words\'
foobar.pluralize(\'goose\') # returns \'geese\'
foobar.singularize(\'phenomena\') # returns \'phenomenon\'
</code></pre>"#
            )
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
