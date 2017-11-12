use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use serde_yaml;
use serde_yaml::Mapping;
use yaml_rust::{YamlLoader, YamlEmitter, Yaml};
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use ::errors::*;

#[derive(Serialize, Debug)]
pub struct Theme {
    name: String,
    path: PathBuf,
    assets: Vec<PathBuf>,
    metadata: Mapping,
}

impl Theme {
    pub fn load(path: &Path) -> Result<Theme> {
        let path_string = path.to_str().ok_or("")?.to_string();

        if !path.exists() {
            bail!(ErrorKind::ThemeNotFound(path_string));
        }

        if !path.is_dir() {
            bail!(ErrorKind::ThemeNotValid(path_string));
        }

        let theme_file_path = path.join("theme.yml");
        let theme_file_path_string = theme_file_path
            .to_str()
            .ok_or("")?
            .to_string();

        if !theme_file_path.exists() {
            bail!(ErrorKind::ThemeFileMissing(path_string));
        }

        let mut document = String::new();
        File::open(&theme_file_path)
            .chain_err(|| format!(
                "Failed to open theme file: '{}'",
                theme_file_path_string,
            ))?
            .read_to_string(&mut document)
            .chain_err(|| format!(
                "Failed to read theme file: '{}'",
                theme_file_path_string,
            ))?;

        let doc = &YamlLoader::load_from_str(&document)?[0];

        let theme_name = match doc["name"].clone() {
            Yaml::String(name) => name,
            Yaml::BadValue => bail!(ErrorKind::ThemeNameMissing),
            _ => bail!(ErrorKind::ThemeNameNotValid),
        };

        let mut theme_assets: Vec<PathBuf> = Vec::new();
        let theme_assets_array = match doc["assets"].clone() {
            Yaml::Array(assets) => assets,
            Yaml::BadValue => bail!(ErrorKind::ThemeAssetsMissing),
            _ => bail!(ErrorKind::ThemeAssetsNotValid),
        };

        for asset_path in theme_assets_array {
            let asset_path_str = match asset_path {
                Yaml::String(path) => path,
                _ => bail!(ErrorKind::ThemeAssetsNotValid),
            };

            theme_assets.push(PathBuf::from(
                format!("{}/{}", path_string, asset_path_str)
            ));
        }

        let theme_metadata: Mapping = match doc["metadata"].clone() {
            Yaml::Hash(_) => {
                let mut metadata_str = String::new();
                {
                    let mut emitter = YamlEmitter::new(&mut metadata_str);
                    emitter.dump(&doc["metadata"]).unwrap(); // @TODO
                }

                serde_yaml::from_str(&metadata_str)?
            },
            Yaml::BadValue => bail!(ErrorKind::ThemeMetadataMissing),
            _ => bail!(ErrorKind::ThemeMetadataNotValid),
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
) -> Result<()> {
    create_dir_all(&destination)?;

    let mut options = CopyOptions::new();
    options.overwrite = true;

    copy_items(theme.get_assets(), &destination, &options)?;

    Ok(())
}
