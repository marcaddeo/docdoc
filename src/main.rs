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
use comrak::{markdown_to_html, ComrakOptions};

#[derive(Serialize)]
pub struct DocdocContext {
    body: String,
    metadata: BTreeMap<String, String>,
}

const USAGE: &'static str = "
docdoc

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
}

pub fn parse_frontmatter(text: &str) -> (Option<&str>, &str) {
    return match text.starts_with("---\n") {
        true => {
            let slice_after_marker = &text[4..];
            match slice_after_marker.find("---\n") {
                Some(end) => (Some(&text[..end + 4]), &text[end + 2 * 4..]),
                None => (None, text),
            }
        },
        false => (None, text),
    }
}

pub fn load_document(file: &str, gfm: bool) -> DocdocContext {
    let mut document = String::new();
    let mut file_handle = File::open(file)
        .expect("File not found!");

    file_handle.read_to_string(&mut document)
        .expect("Could not read file!");

    let (frontmatter, document) = parse_frontmatter(&document);
    let metadata: BTreeMap<String, String> = match frontmatter {
        Some(matter) => {
            serde_yaml::from_str(matter).unwrap()
        },
        None => BTreeMap::new(),
    };

    let mut html = String::with_capacity(document.len() * 3 / 2);

    if gfm {
        let mut options = ComrakOptions::default();

        options.github_pre_lang = true;
        options.ext_strikethrough = true;
        options.ext_table = true;
        options.ext_autolink = true;
        options.ext_tasklist = true;
        options.ext_superscript = true;
        options.ext_header_ids = Some("".to_string());

        html = markdown_to_html(document, &options);
    } else {
        let mut options = Options::empty();
        options.insert(pulldown_cmark::OPTION_ENABLE_TABLES);

        let parser = Parser::new_ext(&document, options);
        html::push_html(&mut html, parser);
    }

    DocdocContext {
        body: html,
        metadata: metadata,
    }
}

pub fn render_document(
    theme_path: &str,
    template: &str,
    docdoc: &DocdocContext
) -> String {
    let mut context = Context::new();
    context.add("docdoc", &docdoc);

    compile_templates!(&format!("{}/**/*.html", theme_path))
        .render(template, &context)
        .unwrap()
}

pub fn write_document(output_file: &str, assets_path: &str, content: &str) {
    let file_name = output_file.replace(".md", ".html");
    let mut file = File::create(file_name)
        .expect("Could not create output file!");

    file.write_all(content.as_bytes())
        .expect("Could not write to output file!");

    let document_path = Path::new(output_file);
    let mut options = CopyOptions::new();
    options.overwrite = true;

    copy(&assets_path, document_path.parent().unwrap(), &options)
        .unwrap();
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let theme_path = match args.flag_theme.is_empty() {
        true => "/Users/marc/dev/rust/docdoc/docs/themes/default",
        false => &args.flag_theme,
    };
    let assets_path = format!("{}/assets", theme_path);
    let document_path = Path::new(&args.arg_file);


    let docdoc = load_document(document_path.to_str().unwrap(), args.flag_gfm);
    let rendered = render_document(theme_path, &args.flag_template, &docdoc);

    write_document(
        &document_path.to_str().unwrap().replace(".md", ".html"),
        &assets_path,
        &rendered,
    );
}
