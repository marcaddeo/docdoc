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
use docdoc::errors::*;

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

    if let Err(error) = run(&args) {
        print_error_chain(&error, true);
        ::std::process::exit(1);
    }
}

fn print_error_chain(error: &Error, backtrace: bool) {
    use std::io::Write;
    let stderr = &mut ::std::io::stderr();
    let error_message = "Error writing to stderr";

    writeln!(stderr, "Error: {}", error).expect(error_message);

    for error in error.iter().skip(1) {
        writeln!(stderr, "Caused by: {}", error).expect(error_message);
    }

    if backtrace {
        if let Some(backtrace) = error.backtrace() {
            writeln!(stderr, "Backtrace: {:?}", backtrace)
                .expect(error_message);
        }

    }
}

fn run(args: &Args) -> Result<()> {
    if args.flag_version {
        println!("{}", VERSION_STR);

        return Ok(());
    }

    let components = args.arg_file.components();
    let mut destination = String::from(args.arg_output_dir.to_str().ok_or("")?);
    let mut skipped_first_directory = false;

    for component in components {
        if let Component::Normal(directory) = component {
            if !skipped_first_directory {
                skipped_first_directory = true;
                continue;
            }

            destination = format!(
                "{}/{}",
                destination,
                directory.to_str().ok_or("")?
            );
        }
    }

    destination = destination.replace(".md", ".html");
    let destination = Path::new(&destination);

    let theme = Theme::load(&args.flag_theme)?;
    let mut document = Document::load(&args.arg_file)?;

    let parser = if args.flag_gfm {
        MarkdownParser::GithubFlavoredMarkdown
    } else {
        MarkdownParser::CommonMark
    };

    let html_body = markdown_to_html(&document.get_body().clone(), &parser);
    document.set_body(html_body);

    let rendered_body = render_document(
        &theme,
        args.flag_template.to_str().ok_or("")?,
        &document
    )?;

    document.set_body(rendered_body);
    document.set_path(destination);

    write_document(&document)?;
    copy_theme_assets(&theme, destination.parent().ok_or("")?)?;

    Ok(())
}
