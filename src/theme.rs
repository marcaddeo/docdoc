use std::fs::{create_dir_all, read_dir, File};
use std::io::prelude::*;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use serde_yaml;
use serde_yaml::{Mapping, Error as SerdeYamlError};
use yaml_rust::{YamlLoader, YamlEmitter, Yaml};
use yaml_rust::emitter::EmitError;
use yaml_rust::scanner::ScanError;
use fs_extra::copy_items;
use fs_extra::dir::remove;
use fs_extra::dir::CopyOptions;
use fs_extra::error::Error as FsError;

#[derive(Debug)]
pub enum Error {
    InvalidPath,
    ThemeNameMissing,
    InvalidThemeName,
    ThemeAssetsMissing,
    InvalidThemeAssets,
    ThemeMetadataMissing,
    InvalidThemeMetadata,
    PathError(String),
    FsError(FsError),
    IoError(IoError),
    YamlError(EmitError),
    YamlScanError(ScanError),
    YamlSerializerError(SerdeYamlError),
}

#[derive(Serialize, Debug)]
pub struct Theme {
    name: String,
    path: PathBuf,
    assets: Vec<PathBuf>,
    metadata: Mapping,
}

impl Theme {
    pub fn load(path: &Path) -> Result<Theme, Error> {
        let path_str = match path.to_str() {
            Some(path) => path,
            None => {
                return Err(Error::InvalidPath);
            },
        };

        if !path.exists() {
            return Err(Error::PathError(
                format!("Theme directory `{}` does not exist!", path_str)
            ));
        }

        if !path.is_dir() {
            return Err(Error::PathError(
                format!("The theme `{}` is not a directory!", path_str)
            ));
        }
        let theme_file_path = path.join("theme.yml");

        if !theme_file_path.exists() {
            return Err(Error::PathError(
                format!("Theme file `{}/theme.yml` does not exist!", path_str)
            ));
        }

        let mut document = String::new();
        let mut file = match File::open(&theme_file_path) {
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

        let docs = match YamlLoader::load_from_str(&document) {
            Ok(yaml_docs) => yaml_docs,
            Err(error) => {
                return Err(Error::YamlScanError(error));
            },
        };
        let doc = &docs[0];


        let theme_name = match doc["name"].clone() {
            Yaml::String(name) => name,
            Yaml::BadValue  => {
                return Err(Error::ThemeNameMissing);
            },
            _ => {
                return Err(Error::InvalidThemeName);
            },
        };

        let mut theme_assets: Vec<PathBuf> = Vec::new();
        let theme_assets_array = match doc["assets"].clone() {
            Yaml::Array(assets) => assets,
            Yaml::BadValue => {
                return Err(Error::ThemeAssetsMissing);
            },
            _ => {
                return Err(Error::InvalidThemeAssets);
            },
        };

        for asset_path in theme_assets_array {
            let asset_path_str = match asset_path {
                Yaml::String(path) => path,
                _ => {
                    return Err(Error::InvalidThemeAssets);
                },
            };

            theme_assets.push(PathBuf::from(
                format!("{}/{}", path_str, asset_path_str)
            ));
        }

        let theme_metadata: Mapping = match doc["metadata"].clone() {
            Yaml::Hash(_) => {
                let mut metadata_str = String::new();
                {
                    let mut emitter = YamlEmitter::new(&mut metadata_str);
                    emitter.dump(&doc["metadata"]).unwrap();
                }

                match serde_yaml::from_str(&metadata_str) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        return Err(Error::YamlSerializerError(error));
                    },
                }
            },
            Yaml::BadValue => {
                return Err(Error::ThemeMetadataMissing);
            },
            _ => {
                return Err(Error::InvalidThemeMetadata);
            },
        };

        Ok(Theme {
            name: theme_name,
            path: path.to_path_buf(),
            assets: theme_assets,
            metadata: theme_metadata,
        })
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn get_assets(&self) -> &Vec<PathBuf> {
        &self.assets
    }

    pub fn get_metadata(&self) -> &Mapping {
        &self.metadata
    }
}

pub fn copy_theme_assets(
    theme: &Theme,
    destination: &Path
) -> Result<(), Error> {
    match create_dir_all(&destination) {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::IoError(error));
        },
    }

    let mut options = CopyOptions::new();
    options.overwrite = true;

    match copy_items(&theme.get_assets(), &destination, &options) {
        Ok(_) => Ok(()),
        Err(error) => Err(Error::FsError(error))
    }
}

pub fn remove_theme_assets(theme: &Theme, directory: &Path, dry_run: bool) -> Result<(), Error> {
    let dir_str = directory.to_str().unwrap();
    let assets = theme.get_assets();

    for asset in assets.iter() {
        if dry_run {
            println!("Would remove {}/{}", dir_str, asset.to_str().unwrap());
        } else {
            match remove(format!("{}/{}", dir_str, asset.to_str().unwrap())) {
                Ok(_) => (),
                Err(error) => {
                    return Err(Error::FsError(error));
                },
            }
        }

    }

    Ok(())
}

pub fn remove_theme_assets_recursive(theme: &Theme, directory: &Path, dry_run: bool) -> Result<(), Error> {
    let paths = read_dir(directory.to_str().unwrap()).unwrap();

    for entry in paths {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            remove_theme_assets(theme, &entry.path(), dry_run)?;
        }
    }

    Ok(())
}
