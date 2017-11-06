extern crate docdoc;
extern crate docopt;
extern crate serde_yaml;
extern crate fs_extra;
#[macro_use]
extern crate serde_derive;

use std::path::{Component, Path, PathBuf};
use docopt::Docopt;
use docdoc::theme::{copy_theme_assets, Theme};
use docdoc::document::{render_document, write_document, Document};
use docdoc::markdown::{markdown_to_html, MarkdownParser};

const VERSION_STR: &'static str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
);
const USAGE: &'static str = "
docdoc

Generate a themed HTML document from Markdown. Supports both CommonMark and GitHub Flavored Markdown.

Usage:
    docdoc [options] [--] <file> <output-dir>
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
    flag_theme: PathBuf,
    flag_template: PathBuf,
    flag_gfm: bool,
    flag_version: bool,
    arg_file: PathBuf,
    arg_output_dir: PathBuf,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        return println!("{}", VERSION_STR);
    }

    let components = args.arg_file.components();
    let mut destination = String::from(args.arg_output_dir.to_str().unwrap());
    let mut skipped_first_directory = false;

    for component in components {
        match component {
            Component::Normal(directory) => {
                if !skipped_first_directory {
                    skipped_first_directory = true;
                    continue;
                }

                destination = format!(
                    "{}/{}",
                    destination,
                    directory.to_str().unwrap()
                );
            },
            _ => (),
        }
    }

    destination = destination.replace(".md", ".html");
    let destination = Path::new(&destination);

    let theme = match Theme::load(&args.flag_theme) {
        Ok(theme) => theme,
        Err(_) => panic!("Failed to load theme!"),
    };

    let mut document = match Document::load(&args.arg_file) {
        Ok(document) => document,
        Err(_) => panic!("Failed to load document!"),
    };

    let parser = if args.flag_gfm {
        MarkdownParser::GithubFlavoredMarkdown
    } else {
        MarkdownParser::CommonMark
    };

    let html_body = markdown_to_html(&document.get_body().clone(), parser);
    document.set_body(html_body);

    let rendered_body = match render_document(
        &theme,
        args.flag_template.to_str().unwrap(),
        &document
    ) {
        Ok(body) => body,
        Err(_) => panic!("Failed to render document!"),
    };

    document.set_body(rendered_body);
    document.set_path(destination);

    match write_document(&document) {
        Ok(_) => (),
        Err(_) => panic!("Failed to write document!"),
    }

    match copy_theme_assets(&theme, destination.parent().unwrap()) {
        Ok(_) => (),
        Err(_) => panic!("Failed to copy theme assets!"),
    }
}
