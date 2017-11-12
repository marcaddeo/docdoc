#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate fs_extra;
extern crate serde_yaml;
extern crate yaml_rust;
extern crate pulldown_cmark;
extern crate comrak;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tera;
extern crate docopt;

pub mod errors;
pub mod theme;
pub mod markdown;
pub mod document;
