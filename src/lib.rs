#![feature(drain_filter)]

mod enums;
mod traits;
mod generator_lib;
mod text_formatter;
mod types_structs;

pub extern crate gen_attributes;

use std::path::PathBuf;
use crate::enums::NewLineState;

/// The various type cases to use when generating interface files
#[derive(Copy, Clone)]
pub enum TypeCases {
    /// Various names of methods variants are untouched
    /// This is the default setting
    Default,
    /// Convert all method names to CamelCase
    CamelCase,
    /// Convert all method method names to snake_case
    SnakeCase,
}

/// The builder to use in build.rs file to generate the interface file
pub struct Generator {
    type_case: TypeCases,
    scr_folder: PathBuf,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
