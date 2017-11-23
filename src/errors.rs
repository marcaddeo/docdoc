error_chain! {
    foreign_links {
        SerdeYamlError(::serde_yaml::Error);
        YamlScanError(::yaml_rust::ScanError);
        YamlEmitError(::yaml_rust::EmitError);
        TeraError(::tera::Error);
        IoError(::std::io::Error);
        FsExtraError(::fs_extra::error::Error);
        DocoptError(::docopt::Error);
    }

    errors {
        DocumentNotValid(path: String) {
            description("Document not valid"),
            display("Document not valid: '{}'", path),

        }

        DocumentNotFound(path: String) {
            description("Document not found"),
            display("Document not found: '{}'", path),
        }

        ThemeNotValid(path: String) {
            description("Theme not valid"),
            display("Theme not valid: '{}'", path),

        }

        ThemeNotFound(path: String) {
            description("Theme not found"),
            display("Theme not found: '{}'", path),
        }

        ThemeFileMissing(path: String) {
            description("Theme file missing"),
            display("Theme '{}' missing 'theme.yml' file", path),
        }

        ThemeNameMissing {
            description("Theme name missing"),
        }

        ThemeNameNotValid {
            description("Theme name not valid"),
        }

        ThemeAssetsMissing {
            description("Theme assets missing"),
        }

        ThemeAssetsNotValid {
            description("Theme assets is not valid"),
        }

        ThemeMetadataMissing {
            description("Theme metadata missing"),
        }

        ThemeMetadataNotValid {
            description("Theme metadata not valid"),
        }

        NeverEndingFrontmatter {
            description("Frontmatter never ends"),
        }
    }
}
