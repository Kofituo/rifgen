use crate::enums::{Delimiters, NewLineState, Types};
use crate::generator_lib::{F_CALLBACK, F_CLASS, F_ENUM};
use crate::text_formatter::StringFormatter;
use crate::TypeCases;
use derive_new::new;
use inflector::Inflector;
use std::iter::Chain;
use std::slice::Iter;

//todo: avoid static mut
pub(crate) static mut TYPE_CASE: TypeCases = TypeCases::Default;

#[derive(Debug, new)]
pub struct ItemInfo {
    ///doc for the method or variant
    pub docs: Vec<String>,
    /// could be an enum variant or a method
    pub signature: String,
    /// item type
    pub is_constructor: bool,
    ///method name to use with only methods
    ///No enums included
    pub method_info: Option<MethodInfo>,
}
#[derive(Debug, new)]

pub struct MethodInfo {
    name: String,
    types_in_method: Vec<String>,
    return_types: Vec<String>,
}

impl MethodInfo {
    pub fn all_types(&self) -> Chain<Iter<'_, String>, Iter<'_, String>> {
        self.types_in_method.iter().chain(self.return_types.iter())
    }
}

impl ItemInfo {
    ///Creates a new `ItemInfo` which is a method
    pub fn new_method(
        signature: String,
        docs: Vec<String>,
        method_name: String,
        is_constructor: bool,
        types_in_method: Vec<String>,
        return_types: Vec<String>,
    ) -> ItemInfo {
        ItemInfo::new(
            docs,
            signature,
            is_constructor,
            Some(MethodInfo::new(method_name, types_in_method, return_types)),
        )
    }

    ///Creates a new `ItemInfo` which is an enum
    pub fn new_enum(signature: String, docs: Vec<String>) -> ItemInfo {
        ItemInfo::new(docs, signature, false, None)
    }
}
///Convenient macro to add the doc comments
#[macro_export]
#[doc(hidden)]
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
#[doc(hidden)]
macro_rules! gen_structs {
    ($($name:ident),*) => {
        $(
            #[derive(Debug,new)]
            pub struct $name {
                /// Name of struct, Trait or Enum
                pub name: String,
                pub type_:Types,
                /// the doc string of this type
                pub docs: Vec<String>,
                /// the methods or variants with this type
                pub extras: Vec<ItemInfo>,
            }

            impl $name {
                pub fn generate_interface(&mut self) -> String {
                    let mut formatter = StringFormatter::new(String::with_capacity(1024), 0);
                    match self.type_ {
                        Types::Struct => self.format_struct(&mut formatter),
                        Types::Trait => self.format_trait(&mut formatter),
                        Types::Enum => {self.format_enum(&mut formatter)}
                    }
                    formatter.close_all_delimiters();
                    formatter.string_container
                }

                fn format_struct(&mut self, formatter: &mut StringFormatter) {
                    //Case where the struct has constructors
                    let constructors = {
                         let mut result = vec![];
                         let mut i = 0;
                         while i < self.extras.len() {
                             if self.extras[i].is_constructor {
                                 let val = self.extras.remove(i);
                                 result.push(val)
                             } else {
                                 i += 1;
                             }
                         }
                         result
                     };
                    let any_is_constructor = !constructors.is_empty();
                    formatter.add_text_delimiter_then_line(
                        vec![F_CLASS],
                        Delimiters::Parenthesis,
                        NewLineState::ShiftRight,
                    );
                    //Add the doc comment associated with this struct
                    add_doc!(self, formatter);
                    formatter.add_text_delimiter_then_line(
                            vec!["class ", &self.name],
                            Delimiters::Bracket,
                            NewLineState::ShiftRight,
                    );

                    if any_is_constructor {
                        formatter.add_text_and_colon(vec!["self_type ",&self.name]);
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
                                TypeCases::CamelCase => (&extra.method_info).as_ref().unwrap().name.to_camel_case(),
                                TypeCases::SnakeCase => (&extra.method_info).as_ref().unwrap().name.to_snake_case(),
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
                    //println!("trait called");
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
                    formatter.add_text_and_colon(vec!["self_type ",&self.name]);
                    for extra in &self.extras {
                        add_doc!(extra, formatter);
                        let mut name = extra.method_info.as_ref().unwrap().name.to_string();
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

                fn format_enum(&mut self, formatter: &mut StringFormatter) {
                    formatter.add_text_delimiter_then_line(
                        vec![F_ENUM],
                        Delimiters::Parenthesis,
                        NewLineState::ShiftRight,
                    );
                    add_doc!(self, formatter);
                    formatter.add_text_delimiter_then_line(
                        vec!["enum ", &self.name],
                        Delimiters::Bracket,
                        NewLineState::ShiftRight,
                    );
                    for extra in &self.extras {
                        add_doc!(extra, formatter);
                        formatter.add_text_and_comma(vec![
                            &extra.signature,
                            " = ",
                            &self.name,
                            "::",
                            &extra.signature,
                        ])
                    }
                }
            }
        )*
    };
}

//Create structs to hold data for various types
gen_structs!(Struct, Enum, Trait);

//Prototype since ide doesn't provide code analysis for macros
// and it's quite difficult programming without code analysis
// just remove the backslash to use
/*#[derive(Debug, new)]
struct TraitTest {
    /// The name of the method or variant as it is in the source code
    name: String,
    type_: Types,
    /// the doc string of this type
    docs: Vec<String>,
    /// the methods or variants with this type
    extras: Vec<ItemInfo>,
}

impl TraitTest {
    ///Convert the info held by this struct
    pub fn generate_interface(&mut self) -> String {
        let mut formatter = StringFormatter::new(String::with_capacity(1024), 0);
        match self.type_ {
            Types::Struct => self.format_struct(&mut formatter),
            Types::Trait => self.format_trait(&mut formatter),
            Types::Enum => self.format_enum(&mut formatter),
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

        formatter.add_text_delimiter_then_line(
            vec!["class ", &self.name],
            Delimiters::Bracket,
            NewLineState::ShiftRight,
        );
        println!("current {}",formatter.string_container);
        if any_is_constructor {
            formatter.add_text_and_colon(vec!["self_type ",&self.name]);
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
                    TypeCases::CamelCase => {
                        (&extra.method_info).as_ref().unwrap().name.to_camel_case()
                    }
                    TypeCases::SnakeCase => {
                        (&extra.method_info).as_ref().unwrap().name.to_snake_case()
                    }
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
        formatter.add_text_and_colon(vec!["self_type ",&self.name]);
        for extra in &self.extras {
            add_doc!(extra, formatter);
            let mut name = extra.method_info.as_ref().unwrap().name.to_string();
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

    fn format_enum(&mut self, formatter: &mut StringFormatter) {
        formatter.add_text_delimiter_then_line(
            vec![F_ENUM],
            Delimiters::Parenthesis,
            NewLineState::ShiftRight,
        );
        add_doc!(self, formatter);
        formatter.add_text_delimiter_then_line(
            vec!["enum ", &self.name],
            Delimiters::Bracket,
            NewLineState::ShiftRight,
        );
        for extra in &self.extras {
            add_doc!(extra, formatter);
            formatter.add_text_and_comma(vec![
                &extra.signature,
                " = ",
                &self.name,
                "::",
                &extra.signature,
            ])
        }
    }
}
*//*
todo!()
#[cfg(test)]
mod tests {
    //use crate::enums::Types;
    use crate::types_structs::{ItemInfo};

    #[test]
    fn test_struct() {
        let items = vec![
            ItemInfo::new_method(
                "new(kkkk: i64, k: Trial) -> Kofi".to_string(),
                vec!["Construct".into()],
                String::from("new"),
                true,
                vec!["i64".into(), "Trial".into()],
                vec![],
            ),
            ItemInfo::new_method(
                "trial(s : String)->i8".into(),
                Vec::new(),
                "trial".into(),
                false,
                vec!["String".into()],
                vec![],
            ),
        ];
        /*let mut result = TraitTest::new(
            "MyClass".into(),
            Types::Struct,
            vec!["My class test".into()],
            items,
        );
        println!("{}", result.generate_interface());*/
    }

    #[test]
    fn test_trait() {
        let items = vec![
            ItemInfo::new_method(
                "trial(s : String)->i8".into(),
                Vec::new(),
                "trial".into(),
                false,
                vec![],
                vec![],
            ),
            ItemInfo::new_method(
                "kofi(s : String)->usize".into(),
                Vec::new(),
                "kofi".into(),true,
                vec![],
                vec![],
            ),
        ];
        /*let mut result = TraitTest::new(
            "MyCallback".into(),
            Types::Trait,
            vec!["My callback test".into()],
            items,
        );
        println!("{}", result.generate_interface());*/
        panic!()
    }

    #[test]
    fn test_enum() {
        //TODO
    }
}
*/
