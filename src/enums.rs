use crate::types_structs::{Enum, Struct, Trait};

///Supported types
#[derive(Debug)]
pub enum Types {
    Struct,
    Trait,
    Enum,
}

#[derive(Debug)]
pub enum TypeHolder {
    Struct(Struct),
    Trait(Trait),
    Enum(Enum),
}

impl TypeHolder {
    pub fn generate_interface(&mut self) -> String {
        match self {
            TypeHolder::Trait(ref mut val) => val.generate_interface(),
            TypeHolder::Struct(ref mut val) => val.generate_interface(),
            TypeHolder::Enum(ref mut val) => val.generate_interface(),
        }
    }

    pub fn types(&self) -> Vec<&String> {
        let mut types = Vec::new();
        match self {
            TypeHolder::Struct(val) => {
                val.extras
                    .iter()
                    .filter_map(|it| it.method_info.as_ref())
                    .for_each(|it| types.append(&mut it.all_types().collect::<Vec<&String>>()));
            }
            TypeHolder::Trait(val) => {
                val.extras
                    .iter()
                    .filter_map(|it| it.method_info.as_ref())
                    .for_each(|it| types.append(&mut it.all_types().collect::<Vec<&String>>()));
            }
            _ => {
                unimplemented!()
            }
        }
        types
    }

    pub fn name(&self) -> &str {
        match self {
            TypeHolder::Struct(val) => val.name.as_str(),
            TypeHolder::Trait(val) => val.name.as_str(),
            TypeHolder::Enum(val) => val.name.as_str(),
        }
    }
}

///`Current` refers to just adding a new line\
/// `ShiftRight` refers to adding a new line and then a tab more than the previous line\
/// `ShiftLeft` refers to adding a new line and then a tab less than the previous line
#[derive(Debug)]
pub enum NewLineState {
    Current,
    ShiftRight,
    ShiftLeft,
}

pub enum Delimiters {
    Bracket,
    Parenthesis,
}
