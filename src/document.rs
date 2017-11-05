use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use serde_yaml;
use serde_yaml::{Mapping, Error as SerdeYamlError};
use markdown::{Markdown, Error as MarkdownError};
use tera::{Context, Error as TeraError};
use theme::Theme;

#[derive(Debug)]
pub enum Error {
    InvalidPath,
    PathError(String),
    IoError(IoError),
    MarkdownError(MarkdownError),
    SerdeYamlError(SerdeYamlError),
    RenderError(TeraError),
}

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

    pub fn load(path: &Path) -> Result<Document, Error> {
        let path_str = match path.to_str() {
            Some(path) => path,
            None => {
                return Err(Error::InvalidPath);
            },
        };

        if !path.exists() {
            return Err(Error::PathError(
                format!("Document `{}` does not exist!", path_str)
            ));
        }

        if !path.is_file() {
            return Err(Error::PathError(
                format!("Document `{}` is not a file!", path_str)
            ));
        }

        let mut document = String::new();
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                return Err(Error::IoError(error));
            },
        };

        match file.read_to_string(&mut document) {
            Ok(_) => (),
            Err(error) => {
                return Err(Error::IoError(error));
            },
        }

        let markdown = match Markdown::parse(&document) {
            Ok(markdown) => markdown,
            Err(error) => {
                return Err(Error::MarkdownError(error));
            },
        };
        let metadata = match markdown.frontmatter() {
            Some(frontmatter) => {
                match serde_yaml::from_str(frontmatter) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        return Err(Error::SerdeYamlError(error));
                    },
                }
            },
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
) -> Result<String, Error> {
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

    let mut context = Context::new();
    context.add("document", &final_document);

    let theme_path_str = match theme.get_path().to_str() {
        Some(path) => path,
        None => {
            return Err(Error::InvalidPath);
        },
    };

    let tera = compile_templates!(&format!("{}/**/*.html", theme_path_str));

    match tera.render(template, &context) {
        Ok(html) => Ok(html),
        Err(error) => {
            return Err(Error::RenderError(error));
        },
    }
}

pub fn write_document(document: &Document) -> Result<(), Error> {
    let document_path_str = match document.get_path().to_str() {
        Some(path) => path,
        None => {
            return Err(Error::InvalidPath);
        },
    };
    let parent_dir = match document.path.parent() {
        Some(dir) => dir,
        None => {
            return Err(Error::PathError(
                format!("`{}` has no parent directory!", document_path_str)
            ));
        },
    };

    match create_dir_all(parent_dir) {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::IoError(error));
        },
    }

    let mut file = match File::create(&document.path) {
        Ok(file) => file,
        Err(error) => {
            return Err(Error::IoError(error));
        },
    };

    match file.write_all(document.get_body().as_bytes()) {
        Ok(_) => Ok(()),
        Err(error) => Err(Error::IoError(error)),
    }
}
