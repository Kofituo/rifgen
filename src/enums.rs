///Supported types
#[derive(Debug)]
pub enum Types {
    Struct,
    Trait,
    Enum,
}

/*#[derive(Debug)]
enum TypeHolder {
    Struct(Struct),
    Trait(Trait),
    Enum(Enum),
}*/
/*
#[derive(Debug)]
enum ItemType {
    Ordinary,
    Constructor,
}
*/
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

enum NewLineProperty {
    ShiftLeft,
    NoShift,
    ShiftRight,
}
