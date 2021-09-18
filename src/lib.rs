#![feature(drain_filter)]

mod enums;
mod generator_lib;
mod maps;
mod text_formatter;
mod traits;
mod types_structs;

pub extern crate gen_attributes_interface_generator;
use crate::generator_lib::FileGenerator;
use std::path::PathBuf;

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

impl Generator {
    /// Creates a new generator instance
    ///
    /// `scr_folder` refers to the starting folder where it is recursively walked
    ///through to find other files
    pub fn new(type_case: TypeCases, scr_folder: PathBuf) -> Generator {
        Generator {
            type_case,
            scr_folder,
        }
    }

    ///`interface_file_path` refers to the path of the output folder
    /// If it exists, it would be overwritten
    pub fn generate_interface(&self, interface_file_path: &PathBuf) {
        FileGenerator::new(
            self.type_case,
            interface_file_path.into(),
            self.scr_folder.to_path_buf(),
        )
        .build();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Generator, TypeCases};

    #[test]
    fn it_works() {
        let test_folder =
            "C:/Users/taimoor/IdeaProjects/proto/rust_interface_file_generator/src/tests";
        let test_file = "C:/Users/taimoor/IdeaProjects/proto/rust_interface_file_generator/src/tests/testfile_new.txt";
        Generator::new(TypeCases::SnakeCase, test_folder.parse().unwrap())
            .generate_interface(&(test_file.parse().unwrap()));
        //panic!()
    }
}
