use crate::enums::{Delimiters, NewLineState, Types};
use crate::generator_lib::{F_CALLBACK, F_CLASS};
use crate::text_formatter::StringFormatter;
use crate::TypeCases;
use derive_new::new;
use inflector::Inflector;

static mut TYPE_CASE: TypeCases = TypeCases::Default;

#[derive(Debug, new)]
struct ItemInfo {
    ///doc for the method or variant
    docs: Vec<String>,
    /// could be an enum variant or a method
    signature: String,
    /// item type
    is_constructor: bool,
    ///method name to use with only methods
    ///No enums included
    method_name: Option<String>,
}

impl ItemInfo {
    ///Creates a new `ItemInfo` which is a constructor
    fn new_constructor(signature: String, docs: Vec<String>, method_name: String) -> ItemInfo {
        ItemInfo::new(docs, signature, true, Some(method_name))
    }

    ///Creates a new `ItemInfo` which is a method but not a constructor
    fn new_method(signature: String, docs: Vec<String>, method_name: String) -> ItemInfo {
        ItemInfo::new(docs, signature, false, Some(method_name))
    }

    ///Creates a new `ItemInfo` which is an enum
    fn new_enum(signature: String, docs: Vec<String>) -> ItemInfo {
        ItemInfo::new(docs, signature, false, None)
    }
}
///Convenient macro to add the doc comments
#[macro_export]
macro_rules! add_doc {
    ($expr:expr,$formatter:expr) => {{
        $expr
            .docs
            .iter()
            .for_each(|it| $formatter.add_text_and_then_line(vec![it], NewLineState::Current));
    }};
}

///This macro is to generate the structs which would be used to hold data for various item types
/// ie Struct, Enum, Trait
#[macro_export]
macro_rules! gen_structs {
    ($($name:ident),*) => {
        $(
            #[derive(Debug,new)]
            struct $name {
                /// Name of struct, Trait or Enum
                name: String,
                type_:Types,
                /// the doc string of this type
                docs: Vec<String>,
                /// the methods or variants with this type
                extras: Vec<ItemInfo>,
            }
        )*
    };
}

//Prototype since ide doesn't provide code analysis for macros
// and it's quite difficult programming without code analysis
#[derive(Debug, new)]
struct Trait {
    name: String,
    type_: Types,
    /// the doc string of this type
    docs: Vec<String>,
    /// the methods or variants with this type
    extras: Vec<ItemInfo>,
}

impl Trait {
    ///Convert the info held by this struct
    pub fn generate_interface(&mut self) -> String {
        let mut formatter = StringFormatter::new(String::with_capacity(1024), 0);
        match self.type_ {
            Types::Struct => self.format_struct(&mut formatter),
            Types::Trait => self.format_trait(&mut formatter),
            Types::Enum => {}
        }
        formatter.close_all_delimiters();
        formatter.string_container
    }

    fn format_struct(&mut self, formatter: &mut StringFormatter) {
        //Case where the struct has constructors
        let constructors = self
            .extras
            .drain_filter(|it| it.is_constructor)
            .collect::<Vec<ItemInfo>>();
        let any_is_constructor = !constructors.is_empty();
        formatter.add_text_delimiter_then_line(
            vec![F_CLASS],
            Delimiters::Parenthesis,
            NewLineState::ShiftRight,
        );
        //Add the doc comment associated with this struct
        add_doc!(self, formatter);

        if any_is_constructor {
            formatter.add_text_delimiter_then_line(
                vec!["class ", &self.name],
                Delimiters::Bracket,
                NewLineState::ShiftRight,
            );
            for constructor in constructors {
                //add doc comment
                formatter.add_text_and_then_line(
                    constructor.docs.iter().map(|it| it.as_str()).collect(),
                    NewLineState::Current,
                );
                formatter.add_text_and_colon(vec![
                    "constructor ",
                    &self.name,
                    "::",
                    &constructor.signature,
                ])
            }
        }

        for extra in &self.extras {
            add_doc!(&extra, formatter);
            let alias = unsafe {
                match TYPE_CASE {
                    TypeCases::Default => String::new(),
                    TypeCases::CamelCase => (&extra.method_name).as_ref().unwrap().to_camel_case(),
                    TypeCases::SnakeCase => (&extra.method_name).as_ref().unwrap().to_snake_case(),
                }
            };
            let alias = if alias.is_empty() {
                alias
            } else {
                format!("; alias {}", alias)
            };
            formatter.add_text_and_colon(vec!["fn ", &self.name, "::", &extra.signature, &alias])
        }
    }

    fn format_trait(&mut self, formatter: &mut StringFormatter) {
        formatter.add_text_delimiter_then_line(
            vec![F_CALLBACK],
            Delimiters::Parenthesis,
            NewLineState::ShiftRight,
        );
        add_doc!(self, formatter);
        formatter.add_text_delimiter_then_line(
            vec!["callback ", &self.name],
            Delimiters::Bracket,
            NewLineState::ShiftRight,
        );
        for extra in &self.extras {
            add_doc!(extra, formatter);
            let mut name = extra.method_name.as_ref().unwrap().to_string();
            name = unsafe {
                match TYPE_CASE {
                    TypeCases::Default => name,
                    TypeCases::CamelCase => (&name).to_camel_case(),
                    TypeCases::SnakeCase => (&name).to_snake_case(),
                }
            };
            formatter.add_text_and_colon(vec![&name, " = ", &self.name, "::", &extra.signature])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::Types;
    use crate::types_structs::{ItemInfo, Trait};

    #[test]
    fn test_struct() {
        let items = vec![
            ItemInfo::new_constructor(
                "new(kkkk: i64, k: Trial) -> Kofi".to_string(),
                vec!["Construct".into()],
                String::from("new"),
            ),
            ItemInfo::new_method("trial(s : String)->i8".into(), Vec::new(), "trial".into()),
        ];
        let mut result = Trait::new(
            "MyClass".into(),
            Types::Struct,
            vec!["My class test".into()],
            items,
        );
        println!("{}", result.generate_interface());
    }

    #[test]
    fn test_trait(){
        let items = vec![
            ItemInfo::new_method("trial(s : String)->i8".into(), Vec::new(), "trial".into()),
            ItemInfo::new_method("kofi(s : String)->usize".into(), Vec::new(), "kofi".into()),
        ];
        let mut result = Trait::new(
            "MyCallback".into(),
            Types::Trait,
            vec!["My callback test".into()],
            items,
        );
        println!("{}", result.generate_interface());
        panic!()
    }
}
