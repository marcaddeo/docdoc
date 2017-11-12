use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use serde_yaml;
use serde_yaml::Mapping;
use markdown::Markdown;
use tera::Context;
use theme::Theme;
use ::errors::*;

#[derive(Serialize, Debug)]
pub struct Document {
    path: PathBuf,
    metadata: Mapping,
    body: String,
}

impl Document {
    pub fn new(path: PathBuf, metadata: Mapping, body: String) -> Document {
        Document {
            path: path,
            metadata: metadata,
            body: body,
        }
    }

    pub fn load(path: &Path) -> Result<Document> {
        let path_string = path.to_str().ok_or("")?.to_string();

        if !path.exists() {
            bail!(ErrorKind::DocumentNotFound(path_string));
        }

        if !path.is_file() {
            bail!(ErrorKind::DocumentNotValid(path_string));
        }

        let mut document = String::new();
        File::open(path)
            .chain_err(|| format!(
                "Failed to open document: '{}'",
                path_string,
            ))?
            .read_to_string(&mut document)
            .chain_err(|| format!(
                "Failed to read document: '{}'",
                path_string,
            ))?;

        let markdown = Markdown::parse(&document)?;
        let metadata = match markdown.frontmatter() {
            Some(frontmatter) => serde_yaml::from_str(frontmatter)?,
            None => Mapping::new(),
        };

        Ok(Document {
            path: path.to_path_buf(),
            metadata: metadata,
            body: String::from(markdown.content()),
        })
    }

    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn get_metadata(&self) -> &Mapping {
        &self.metadata
    }

    pub fn get_body(&self) -> &String {
        &self.body
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }

    pub fn set_metadata(&mut self, metadata: Mapping) {
        self.metadata = metadata;
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }
}

pub fn render_document(
    theme: &Theme,
    template: &str,
    document: &Document
) -> Result<String> {
    // Merge document metadata into theme metadata
    let mut theme_metadata = theme.get_metadata().clone();
    let document_metadata_iter = document.get_metadata().iter();

    for (key, value) in document_metadata_iter {
        // Themes must define all possible variables
        if theme_metadata.contains_key(key) {
            theme_metadata.insert(key.clone(), value.clone());
        }
    }

    let final_document = Document::new(
        document.get_path().to_path_buf(),
        theme_metadata,
        document.get_body().clone()
    );

    let tera = compile_templates!(
        &format!("{}/**/*.html", theme.get_path().to_str().ok_or("")?)
    );

    let mut context = Context::new();
    context.add("document", &final_document);

    Ok(tera.render(template, &context)?)
}

pub fn write_document(document: &Document) -> Result<()> {
    let path_string = document.get_path().to_str().ok_or("")?.to_string();

    create_dir_all(document.get_path().parent().ok_or("")?)?;

    File::create(&document.get_path())
        .chain_err(|| format!(
            "Failed to create document: '{}'",
            path_string
        ))?
        .write_all(document.get_body().as_bytes())
        .chain_err(|| format!(
            "Failed to write document: '{}'",
            path_string
        ))?;

    Ok(())
}
