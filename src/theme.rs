use std::fs::File;
use std::io::prelude::*;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use yaml_rust::{YamlLoader, Yaml};
use yaml_rust::emitter::EmitError;
use fs_extra::copy_items; use fs_extra::dir::CopyOptions;
use fs_extra::error::Error as FsError;

#[derive(Debug)]
pub struct Theme {
    name: String,
    path: PathBuf,
    assets: Vec<PathBuf>,
    metadata: Yaml,
}

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
    YamlError(EmitError)
}

impl Theme {
    pub fn load(path: &Path) -> Result<Theme, Error> {
        let path_str = match path.to_str() {
            Some(path) => path,
            None => {
                return Err(Error::InvalidPath);
            }
        };

        if !path.is_dir() {
            return Err(Error::PathError(
                format!("The theme \"{}\" is not a directory!", path_str)
            ));
        }

        if !path.exists() {
            return Err(Error::PathError(
                format!("Theme directory \"{}\" does not exist!", path_str)
            ));
        }

        let theme_file_path = path.join("theme.yml");

        if !theme_file_path.exists() {
            return Err(Error::PathError(
                format!("Theme file \"{}/theme.yml\" does not exist!", path_str)
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
            Err(_) => panic!("lol"),
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

            theme_assets.push(PathBuf::from(asset_path_str));
        }

        let theme_metadata = match doc["metadata"].clone() {
            Yaml::Hash(_) => doc["metadata"].clone(),
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

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn assets(&self) -> Vec<PathBuf> {
        self.assets.clone()
    }

    pub fn metadata(&self) -> Yaml {
        self.metadata.clone()
    }

    pub fn copy_assets(&self, destination: &Path) -> Result<(), Error> {
        let mut options = CopyOptions::new();
        options.overwrite = true;

        match copy_items(&self.assets, &destination, &options) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::FsError(error))
        }
    }

}
