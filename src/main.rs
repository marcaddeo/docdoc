extern crate docdoc;
extern crate docopt;
extern crate serde_yaml;
extern crate fs_extra;
#[macro_use]
extern crate serde_derive;

use std::path::Path;
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

// @TODO Maybe consider not putting dist/ in front of the file name. This
// prevents us from keepnig the file hiearchy intact.
//
// But even without it, there are other issues:
// - Clean up is non-trivial since themes define any number of assets that get
//   copied into place. Maybe a `docdoc clean` command? But then it seems like
//   docdoc has to know too much information. Where does it look to delete
//   theme assets? Does it recurse? You'd have to do something like:
//   ```
//   docdoc clean -r --theme=docs/themes/theme_name docs/
//   ```
//
//   Which would remove all of the themes assets from the docs directory,
//   recursively.
// - Similarly, deployment is non-trivial for the same reason. How do you know
//   what files to deploy to the final location? You can glob for **/*.html,
//   but again themes can define any number of assets. Maybe a
//   `docdoc ls-files` command?
//
//   ```
//   docdoc ls-files --theme=docs/themes/theme_name docs/
//
//   docs/document.html
//   docs/assets
//   docs/test/something.html
//   docs/test/assets
//   ```
//
//   This would basically just global for all **/*.html files in a path, and
//   find any associated assets as well based on the themes asset definitions.
//
// Both issues have solutions that seem error prone, and seem to do too much.
// Maybe something simpler is possible.
fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        return println!("{}", VERSION_STR);
    }

    let document_file_name = match Path::new(&args.arg_file).file_name() {
        Some(os_str) => match os_str.to_str() {
            Some(file_name) => file_name,
            None => panic!("Could not determine document file name!"),
        },
        None => panic!("Could not determine document file name!"),
    };
    let destination_document_file_name = document_file_name
        .replace(".md", ".html");
    let destination_document_dir = Path::new(&args.arg_file)
        .with_file_name("dist/");
    let destination_document_path = Path::new(&args.arg_file).with_file_name(
        format!("dist/{}", destination_document_file_name)
    );

    let theme = match Theme::load(Path::new(&args.flag_theme)) {
        Ok(theme) => theme,
        Err(_) => panic!("Failed to load theme!"),
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

    let html_body = markdown_to_html(&document.body().clone(), parser);
    document.set_body(html_body);

    let rendered_body = match render_document(&theme, &args.flag_template, &document) {
        Ok(body) => body,
        Err(_) => panic!("Failed to render document!"),
    };

    document.set_body(rendered_body);
    document.set_path(&destination_document_path);

    match write_document(&document) {
        Ok(_) => (),
        Err(_) => panic!("Failed to write document!"),
    }

    match copy_theme_assets(&theme, &destination_document_dir) {
        Ok(_) => (),
        Err(_) => panic!("Failed to copy theme assets!"),
    }
}
