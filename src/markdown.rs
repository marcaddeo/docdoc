use std::path::{Component, PathBuf};
use std::borrow::Cow;
use pulldown_cmark::OPTION_ENABLE_TABLES;
use pulldown_cmark::{Parser, Options, Event, Tag, html};
use typed_arena::Arena;
use comrak::{parse_document, format_html, ComrakOptions};
use comrak::nodes::{AstNode, NodeValue};
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

pub fn markdown_to_html(markdown: &str, parser: &MarkdownParser) -> Result<String> {
    match *parser {
        MarkdownParser::CommonMark => {
            let mut html = String::with_capacity(markdown.len() * 3 / 2);
            let mut options = Options::empty();
            options.insert(OPTION_ENABLE_TABLES);

            let parser = Parser::new_ext(markdown, options);
            let parser = parser.map(|event| match event {
                Event::Start(Tag::Link(link, title)) => {
                    Event::Start(Tag::Link(
                        Cow::from(handle_link(String::from(link)).unwrap()),
                        title
                    ))
                },
                _ => event,
            });
            html::push_html(&mut html, parser);

            Ok(html)
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

            let arena = Arena::new();
            let root = parse_document(&arena, markdown, &options);

            iter_nodes(root, &|node| {
                match &mut node.data.borrow_mut().value {
                    &mut NodeValue::Link(ref mut link) => {
                        let url = link.url.clone();
                        link.url = handle_link(String::from_utf8(url).unwrap()).unwrap()
                            .as_bytes()
                            .to_vec();
                    },
                    _ => (),
                }
            });

            let mut html = vec![];
            format_html(root, &options, &mut html)?;

            Ok(String::from_utf8(html)?)
        },
    }
}

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
    where F: Fn(&'a AstNode<'a>) {
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

fn handle_link(url: String) -> Result<String> {
    let path = PathBuf::from(url.clone());

    if !path.has_root() {
        return Ok(url);
    }

    if path.extension().ok_or("")? != "md" {
        return Ok(url);
    }

    let path = path.with_extension("html");

    let mut destination = String::new();
    let mut skipped_first_directory = false;
    for component in path.components() {
        if let Component::Normal(part) = component {
            if !skipped_first_directory {
                skipped_first_directory = true;
                continue;
            }

            destination = format!(
                "{}/{}",
                destination,
                part.to_str().ok_or("")?
            );
        }
    }

    Ok(destination)
}
