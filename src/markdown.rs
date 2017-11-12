use pulldown_cmark::OPTION_ENABLE_TABLES;
use pulldown_cmark::{Parser, Options,  html};
use comrak::{markdown_to_html as comrak_markdown_to_html, ComrakOptions};
use ::errors::*;

#[derive(Debug)]
pub enum MarkdownParser {
    CommonMark,
    GithubFlavoredMarkdown,
}

pub struct Markdown<'a> {
    frontmatter: Option<&'a str>,
    content: &'a str,
}

impl<'a> Markdown<'a> {
    pub fn parse(text: &'a str) -> Result<Markdown<'a>> {
        if text.starts_with("---\n") {
            match String::from(&text[4..]).find("---\n") {
                Some(end) => {
                    Ok(Markdown {
                        frontmatter: Some(&text[..end + 4]),
                        content: &text[end + 8..],
                    })
                },
                None => bail!(ErrorKind::NeverEndingFrontmatter),
            }
        } else {
            Ok(Markdown {
                frontmatter: None,
                content: text,
            })
        }
    }

    pub fn frontmatter(&self) -> Option<&'a str> {
        self.frontmatter
    }

    pub fn content(&self) -> &'a str {
        self.content
    }
}

pub fn markdown_to_html(markdown: &str, parser: &MarkdownParser) -> String {
    match *parser {
        MarkdownParser::CommonMark => {
            let mut html = String::with_capacity(markdown.len() * 3 / 2);
            let mut options = Options::empty();
            options.insert(OPTION_ENABLE_TABLES);

            let parser = Parser::new_ext(markdown, options);
            html::push_html(&mut html, parser);

            html
        },
        MarkdownParser::GithubFlavoredMarkdown => {
            let mut options = ComrakOptions::default();

            options.github_pre_lang = true;
            options.ext_strikethrough = true;
            options.ext_table = true;
            options.ext_autolink = true;
            options.ext_tasklist = true;
            options.ext_superscript = true;
            options.ext_header_ids = Some("".to_string());

            comrak_markdown_to_html(markdown, &options)
        },
    }
}
