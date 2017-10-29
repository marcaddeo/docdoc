extern crate docdoc;
extern crate pulldown_cmark;
extern crate docopt;
extern crate serde_yaml;
extern crate fs_extra;
extern crate comrak;
#[macro_use]
extern crate tera;
#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::prelude::*;
use std::collections::BTreeMap;
use std::path::Path;
use pulldown_cmark::{Parser, Options,  html};
use tera::Context;
use docopt::Docopt;
use fs_extra::dir::{copy, CopyOptions};
use comrak::{markdown_to_html as comrak_markdown_to_html, ComrakOptions};

#[derive(Serialize)]
pub struct DocumentContext<'a> {
    metadata: &'a BTreeMap<String, String>,
    body: &'a str,
}

const VERSION_STR: &'static str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
);
const USAGE: &'static str = "
docdoc

Generate a themed HTML document from Markdown. Supports both CommonMark and GitHub Flavored Markdown.

Usage:
    docdoc [options] [--] <file>
    docdoc -h | --help
    docdoc --version

Options:
    --theme=<theme>         Use a custom theme.
    --template=<template>   Use a specific template in a theme. [default: index.html]
    --gfm                   Use GitHub Flavored Markdown.
    -h, --help              Show this screen.
    --version               Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file: String,
    flag_theme: String,
    flag_template: String,
    flag_gfm: bool,
    flag_version: bool,
}

pub struct FrontmatteredMarkdown<'a> {
    frontmatter: Option<&'a str>,
    content: &'a str,
}

impl<'a> FrontmatteredMarkdown<'a> {
    pub fn parse(text: &'a str) -> FrontmatteredMarkdown<'a> {
        if text.starts_with("---\n") {
            match String::from(&text[4..]).find("---\n") {
                Some(end) => {
                    FrontmatteredMarkdown {
                        frontmatter: Some(&text[..end + 4]),
                        content: &text[end + 8..],
                    }
                },
                None => panic!("Frontmatter never ends!"),
            }
        } else {
            FrontmatteredMarkdown {
                frontmatter: None,
                content: text,
            }
        }
    }
}

pub fn markdown_to_html(markdown: &str, gfm: bool) -> String {
    if gfm {
        let mut options = ComrakOptions::default();

        options.github_pre_lang = true;
        options.ext_strikethrough = true;
        options.ext_table = true;
        options.ext_autolink = true;
        options.ext_tasklist = true;
        options.ext_superscript = true;
        options.ext_header_ids = Some("".to_string());

        comrak_markdown_to_html(markdown, &options)
    } else {
        let mut html = String::with_capacity(markdown.len() * 3 / 2);
        let mut options = Options::empty();
        options.insert(pulldown_cmark::OPTION_ENABLE_TABLES);

        let parser = Parser::new_ext(markdown, options);
        html::push_html(&mut html, parser);

        html
    }
}

pub fn load_document(path: &str) -> String {
    let mut document = String::new();
    let mut file = File::open(path)
        .expect(&format!("\"{}\" not found!", path));

    file.read_to_string(&mut document)
        .expect(&format!("Could not read \"{}\"!", path));

    document
}

pub fn render_document(
    theme_path: &str,
    template: &str,
    document: &DocumentContext,
) -> String {
    let mut context = Context::new();
    context.add("document", &document);

    let tera = compile_templates!(&format!("{}/**/*.html", theme_path));

    match tera.render(template, &context) {
        Ok(html) => html,
        Err(error) => {
            println!("{}", error);
            for err in error.iter().skip(1) {
                println!("{}", err);
            }
            ::std::process::exit(1);
        }
    }
}

pub fn write_document(output_file: &str, assets_path: &str, content: &str) {
    let file_name = output_file.replace(".md", ".html");
    let mut file = File::create(file_name)
        .expect("Could not create output file!");

    file.write_all(content.as_bytes())
        .expect("Could not write to output file!");

    let document_path = Path::new(output_file);
    let document_parent = match document_path.parent() {
        Some(parent) => parent,
        None => panic!("Could not find documents parent directory!")
    };

    let mut options = CopyOptions::new();
    options.overwrite = true;

    match copy(&assets_path, document_parent, &options) {
        Ok(_) => (),
        Err(_) => panic!(
            "An error occured while trying to copy theme assets from \"{}\" to document destination \"{}\"!",
            assets_path,
            document_parent.to_str().unwrap(),
        ),
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        return println!("{}", VERSION_STR);
    }

    let theme_path = if args.flag_theme.is_empty() {
        "/Users/marc/dev/rust/docdoc/docs/themes/default"
    } else {
        &args.flag_theme
    };
    let assets_path = format!("{}/assets", theme_path);
    let document_path: &str = match Path::new(&args.arg_file).to_str() {
        Some(path) => path,
        None => panic!(format!("\"{}\" is not a valid path!", &args.arg_file)),
    };

    let document = load_document(document_path);
    let markdown = FrontmatteredMarkdown::parse(&document);
    let metadata: BTreeMap<String, String> = match markdown.frontmatter {
        Some(frontmatter) => {
            serde_yaml::from_str(frontmatter)
                .expect("Could not parse YAML Frontmatter")
        },
        None => BTreeMap::new(),
    };

    let document_context = DocumentContext {
        metadata: &metadata,
        body: &markdown_to_html(markdown.content, args.flag_gfm),
    };

    let rendered = render_document(
        theme_path,
        &args.flag_template,
        &document_context,
    );

    write_document(
        &document_path.replace(".md", ".html"),
        &assets_path,
        &rendered,
    );
}
