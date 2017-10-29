use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use serde_yaml;
use serde_yaml::{Mapping, Error as SerdeYamlError};
use markdown::{Markdown, Error as MarkdownError};

#[derive(Debug)]
pub enum Error {
    InvalidPath,
    PathError(String),
    IoError(IoError),
    MarkdownError(MarkdownError),
    SerdeYamlError(SerdeYamlError),
}

#[derive(Serialize, Debug)]
pub struct Document {
    path: PathBuf,
    metadata: Mapping,
    body: String,
}

impl Document {
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

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn metadata(&self) -> &Mapping {
        &self.metadata
    }

    pub fn body(&self) -> &String {
        &self.body
    }

    pub fn with_path(&self, path: &Path) -> Document {
        Document {
            path: path.to_path_buf(),
            metadata: self.metadata.clone(),
            body: self.body.clone(),
        }
    }

    pub fn with_metadata(&self, metadata: Mapping) -> Document {
        Document {
            path: self.path.clone(),
            metadata: metadata,
            body: self.body.clone(),
        }
    }

    pub fn with_body(&self, body: String) -> Document {
        Document {
            path: self.path.clone(),
            metadata: self.metadata.clone(),
            body: body,
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        let parent_dir = match self.path.parent() {
            Some(dir) => dir,
            None => {
                return Err(Error::PathError(
                    format!("`{}` has no parent directory!", &self.path.to_str().unwrap())
                ));
            },
        };

        match create_dir_all(parent_dir) {
            Ok(_) => (),
            Err(error) => {
                return Err(Error::IoError(error));
            },
        }

        let mut file = match File::create(&self.path) {
            Ok(file) => file,
            Err(error) => {
                return Err(Error::IoError(error));
            },
        };

        match file.write_all(self.body.as_bytes()) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::IoError(error)),
        }
    }
}
