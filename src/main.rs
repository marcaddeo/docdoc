extern crate docdoc;
extern crate docopt;
extern crate serde_yaml;
extern crate fs_extra;
#[macro_use]
extern crate tera;
#[macro_use]
extern crate serde_derive;

use std::path::Path;
use tera::Context;
use docopt::Docopt;
use docdoc::theme::Theme;
use docdoc::document::Document;
use docdoc::markdown::{MarkdownParser, markdown_to_html};

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
    --theme=<theme>         Use a custom theme. [default: /usr/local/share/docodoc/themes/default]
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

pub fn render_document(
    theme: &Theme,
    template: &str,
    document: &Document
) -> String {
    let mut context = Context::new();
    context.add("document", &document);

    let tera = compile_templates!(&format!("{}/**/*.html", theme.path().to_str().unwrap()));

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

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        return println!("{}", VERSION_STR);
    }

    let theme = match Theme::load(Path::new(&args.flag_theme)) {
        Ok(theme) => theme,
        Err(error) => {
            println!("Failed to load theme!");
            println!("{:?}", error);
            ::std::process::exit(1);
        }
    };

    let mut document = match Document::load(Path::new(&args.arg_file)) {
        Ok(document) => document,
        Err(_) => panic!("Failed to load document!"),
    };

    let parser = if args.flag_gfm {
        MarkdownParser::GithubFlavoredMarkdown
    } else {
        MarkdownParser::CommonMark
    };

    document = document.with_body(markdown_to_html(document.body(), parser));
    document = document.with_body(render_document(&theme, &args.flag_template, &document));

    let file_name = document.path().file_name().unwrap().to_str().unwrap();
    let destination = document.path().with_file_name(format!("dist/{}", file_name.replace(".md", ".html")));

    // @TODO figure out the borrow issue here
    let final_document = document.with_path(&destination);

    final_document.write().unwrap();
    theme.copy_assets(&final_document.path().parent().unwrap()).unwrap();
}
