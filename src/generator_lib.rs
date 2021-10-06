use crate::enums::{TypeHolder, Types};
use crate::types_structs::{Enum, ItemInfo, Struct, Trait, TYPE_CASE};
use crate::{Language, TypeCases};
use derive_new::new;
use std::collections::{HashMap, VecDeque};
use std::fs::{DirEntry, File};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use syn::__private::ToTokens;
use syn::{PathArguments, ReturnType, Type};
use std::time::Instant;

//constants

pub const F_CLASS: &str = "foreign_class!";
pub const F_CALLBACK: &str = "foreign_callback!";
pub const F_ENUM: &str = "foreign_enum!";

//errors
const UNABLE_TO_READ: &str = "Unable to read file: ";

//helper macros
#[derive(new, Debug)]
struct AttrCheck {
    is_attribute: bool,
    is_constructor: bool,
}

macro_rules! has_gen_attr {
    ($expr:expr) => {
        has_gen_attr!($expr, false)
    };
    ($expr:expr,$check_for_constructor:expr) => {{
        let mut is_attribute = false;
        let mut is_constructor = false;
        $expr.attrs.iter().any(|it| {
            is_attribute = it
                .path
                .segments
                .iter()
                .any(|it| it.ident == "generate_interface");
            if is_attribute && $check_for_constructor {
                is_constructor = it.tokens.to_string().contains("constructor");
            }
            is_attribute
        });
        AttrCheck::new(is_attribute, is_constructor)
    }};
}

//to use with Structs only
macro_rules! has_doc_gen_attr {
    ($expr:expr) => {
        $expr.attrs.iter().any(|it| {
            it.path
                .segments
                .iter()
                .any(|it| &it.ident.to_string() == "generate_interface_doc")
        })
    };
}
macro_rules! get_doc {
    ($expr:expr) => {
        $expr
            .attrs
            .iter()
            .filter_map(|it| {
                //segments usually contain an item but just in case
                let doc = it
                    .path
                    .segments
                    .iter()
                    .map(|it| it.ident.to_string())
                    .filter(|it| it == "doc")
                    .next();
                //
                if let Some(_) = doc {
                    Some(it.to_token_stream().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
    };
}

macro_rules! correct_string {
    ($string:expr) => {{
        let sign = $string.rfind("<");
        if let Some(sign) = sign {
            //means it's something like Box<Kofi>
            // find the other >
            let other = $string.find(">").unwrap();
            let new_string = &$string[sign + 1..other];
            new_string.trim().to_string()
        } else {
            $string.trim().into()
        }
    }};
}

macro_rules! types_in_method {
    ($expr:expr) => {{
        let result = (&$expr)
            .sig
            .inputs
            .iter()
            .filter_map(|it| {
                match it {
                    syn::FnArg::Receiver(_) => None, //something like (&self)
                    syn::FnArg::Typed(typ) => {
                        let _type = typ.ty.to_token_stream().to_string().replace(" dyn ", "");
                        ////println!("type {}", _type);
                        Some(correct_string!(_type))
                    }
                }
            })
            .collect::<Vec<_>>();
        result
    }};
}

macro_rules! return_types {
    ($expr:expr) => {{
        let mut return_types: Vec<String> = Vec::new();
        match &$expr.sig.output {
            ReturnType::Type(_, val) => match &**val {
                Type::Path(val) => {
                    //let result = vec![];
                    val.path.segments.iter().for_each(|it| {
                        return_types.push((&it.ident).to_string());
                        match &it.arguments {
                            PathArguments::AngleBracketed(val) => {
                                val.args.iter().for_each(|it| {
                                    let str = it.to_token_stream().to_string();
                                    return_types.push(str);
                                });
                            }
                            _ => {}
                        }
                    });
                }
                _ => {}
            },
            _ => {}
        }
        return_types
    }};
}

macro_rules! function_signature {
    ($expr:expr) => {{
        {
            let signature = &$expr.sig;
            let mut iter = signature.to_token_stream().into_iter();
            //it could start with maybe unsafe fn or fn
            iter.find(|it| it.to_string() == "fn");
            let mut result = String::with_capacity(32);
            while let Some(val) = iter.next() {
                let val = val.to_string();
                if val == "'" {
                    //lifetimes cause problems
                    result.push_str(&val); // push the lifetime param
                    result.push_str(&iter.next().unwrap().to_string());
                    result.push(' ');
                } else {
                    result.push_str(&val)
                }
            }
            result
        }
    }};
}

/// First all enums would be placed at the start of the file to make things simpler
///
/// so now to the traits and structs
///
/// assuming we have struct A and struct B
/// struct A has a method which depends on struct B but not vice versa
/// struct A is first added
/// so now it's time to add struct B
/// struct B should be placed in front of struct A in vec deque
struct ItemsHolder {
    list: HashMap<Rc<String>, TypeHolder>,
    enums_list: Vec<Enum>,
    final_list: VecDeque<Rc<String>>,
}

impl ItemsHolder {
    fn new(capacity: usize) -> ItemsHolder {
        ItemsHolder {
            list: HashMap::with_capacity(capacity),
            enums_list: Vec::new(),
            final_list: VecDeque::with_capacity(capacity),
        }
    }
    /*fn ensure_new(&self, name: Rc<String>) {
        assert!(
            !self.list.contains_key(&name),
            "A struct with a similar name already exists. struct {}",
            &name
        );
    }*/

    fn add_items(&mut self, name: Rc<String>, item: TypeHolder) {
        self.list.insert(name, item);
    }

    fn sort_items(&mut self) {
        /* So now it's a 2 way something
        Given 3 types: North, South, East
        If North has a method that depends on East, but North is added first, when East is added, it should
        be placed before North and not after it
         */
        //supposed We have F, N, M, O, T
        // F is added first but F has a method which depends on N
        //N has a method which depends on O
        // So in effect the list should be [O, N, F, ...] even though F was added first
        if self.list.is_empty() {
            eprintln!("Annotate methods and enums to use module rust_interface_file_generator");
            return;
        }
        self.list.keys().next().unwrap().to_string();
        let mut values = self
            .list
            .keys()
            .map(|it| (it.clone(), None))
            .collect::<HashMap<Rc<String>, Option<()>>>();

        //TODO optimise it
        fn analyse_item(
            item: &TypeHolder,
            values: &mut HashMap<Rc<String>, Option<()>>,
            map: &HashMap<Rc<String>, TypeHolder>,
            out: &mut VecDeque<Rc<String>>,
        ) {
            let types = item.types();
            //println!("types {:?} name {}", types, item.name());
            let item_name = item.name().to_string();
            values.remove(&item_name);
            let item_name = Rc::new(item_name);
            out.iter().position(|it| it == &item_name).and_then(|it| {
                //println!("and then {}", item_name);
                Some(out.remove(it))
            });
            out.push_front(item_name);
            for _type in types {
                //println!("type {}", _type);
                if let Some(val) = map.get(_type) {
                    if val.name() == item.name() {
                        //Since classes with constructors have `self` as type
                        continue;
                    }
                    //println!("val {} item {}", val.name(), item.name());
                    //println!("for ilist {:?}", out);
                    analyse_item(val, values, map, out);
                }
            }
            //values.remove(&item.name().to_string());
            ////println!("list {:?}", out);
            //panic!();
        }

        //let mut times = 0;
        while !values.is_empty() {
            /*println!(
                "while {:?} start {}",
                values.keys().collect::<Vec<&Rc<String>>>(),
                values.iter().next().unwrap().0.to_string()
            );*/
            analyse_item(
                self.list
                    .get(&values.iter().next().unwrap().0.to_string())
                    .unwrap(),
                &mut values,
                &self.list,
                &mut self.final_list,
            );
            //println!("panic {:?}", values.keys().collect::<Vec<&Rc<String>>>());
            /*times += 1;
            if times == 5 {
                panic!()
            }*/
            //panic!()
        }
        ////println!("out {}",)
    }

    fn add_enum(&mut self, data: Enum) {
        //self.list.insert(Rc::new(data.name.to_string()),TypeHolder::Enum(data));
        self.enums_list.push(data)
    }

    fn generate_interface(mut self, language: Language, out_file: &PathBuf) {
        //println!("final {:?}", self.final_list);
        let mut file = File::create(out_file).expect("Unable to write to disk");
        file.write(b"//Automatically generated by rifgen\nuse crate::*;\n")
            .unwrap();
        if matches!(language, Language::Java) {
            file.write_all(b"use jni_sys::*;\n").unwrap();
        }
        //first add enums since enums "can't" depend on other data structures
        self.sort_items();
        for mut enums in self.enums_list {
            file.write_all(enums.generate_interface().as_ref())
                .expect("Unable to write to disk");
        }

        /*assert_eq!(
            self.final_list,
            vec![
                Rc::new("OTH".into()),
                Rc::new("Kofi".into()),
                Rc::new("Noth".into()),
                Rc::new("Finalise".into()),
                Rc::new("ImDone".into())
            ]
        );*/
        //println!("tested");
        for name in self.final_list {
            file.write_all(
                self.list
                    .get_mut(&*name)
                    .unwrap()
                    .generate_interface()
                    .as_ref(),
            )
            .expect("Unable to write to disk");
        }
    }
}
// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &PathBuf, cb: &mut dyn FnMut(&std::fs::DirEntry)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

pub struct FileGenerator {
    interface_file_path: PathBuf,
    starting_point: PathBuf,
}

impl FileGenerator {
    pub fn new(
        type_case: TypeCases,
        interface_file_path: PathBuf,
        starting_point: PathBuf,
    ) -> FileGenerator {
        unsafe { TYPE_CASE = type_case }
        FileGenerator {
            interface_file_path,
            starting_point,
        }
    }

    pub fn build(&self, language: Language) {
        let start = Instant::now();
        //the closure to be applied to each file
        let mut file_data: HashMap<Rc<String>, TypeHolder> = HashMap::new();
        let mut closure = |file: &DirEntry| {
            let file_path = file.path();
            let file_contents = std::fs::read_to_string(&file_path).expect(&format!(
                "{}{}",
                UNABLE_TO_READ,
                file_path.to_str().unwrap()
            ));
            let compiled_file = syn::parse_file(&file_contents).expect("Invalid rust file");

            for item in &compiled_file.items {
                //
                match item {
                    syn::Item::Struct(item) => {
                        //check if it has the doc attribute
                        if has_doc_gen_attr!(item) {
                            let name = Rc::new((&item).ident.to_string());
                            //assert!(!file_data.contains_key(&name.clone()));
                            //the impl block may come (ie if it's in a different file) before the struct definition
                            if let Some(val) = file_data.get_mut(&name) {
                                match val {
                                    TypeHolder::Struct(val) => {
                                        val.docs.append(&mut get_doc!(item));
                                    }
                                    _ => {
                                        panic!("Expected {} to be a struct", name)
                                    }
                                }
                            } else {
                                file_data.insert(
                                    name.clone(),
                                    TypeHolder::Struct(Struct::new(
                                        name.to_string(),
                                        Types::Struct,
                                        get_doc!(item),
                                        vec![],
                                    )),
                                );
                            }
                        }
                    }
                    syn::Item::Fn(val) => {
                        // function not in impl block
                        let name = val.sig.ident.to_string();
                        if has_gen_attr!(val).is_attribute {
                            panic!(
                                "interface functions should be declared in impl blocks.\nName of function {}",
                                name
                            )
                        }
                    }
                    syn::Item::Impl(val) => {
                        //TODO let it work with enums
                        FileGenerator::impl_data(&mut file_data, val);
                    }
                    syn::Item::Enum(val) => {
                        if has_gen_attr!(val).is_attribute {
                            let name = Rc::new((&val).ident.to_string());
                            assert!(
                                !file_data.contains_key(&name),
                                "Multiple definitions of {}",
                                &name
                            ); // make sure no other struct has the same name
                            let variants = val
                                .variants
                                .iter()
                                .map(|it| ItemInfo::new_enum(it.ident.to_string(), get_doc!(it)))
                                .collect();

                            file_data.insert(
                                name.clone(),
                                TypeHolder::Enum(Enum::new(
                                    name.to_string(),
                                    Types::Enum,
                                    get_doc!(val),
                                    variants,
                                )),
                            );
                        }
                    }
                    syn::Item::Trait(val) => {
                        if !has_gen_attr!(val).is_attribute {
                            continue;
                        }
                        //println!("trait");
                        let name = Rc::new((&val).ident.to_string());
                        let mut trait_data: Trait = Trait::new(
                            name.to_string(),
                            Types::Trait,
                            get_doc!(val),
                            Vec::with_capacity(val.items.len()),
                        );
                        for item in &val.items {
                            if let syn::TraitItem::Method(method) = item {
                                let method_name = (&method).sig.ident.to_string();
                                trait_data.extras.push(ItemInfo::new_method(
                                    function_signature!(method),
                                    get_doc!(method),
                                    method_name,
                                    false,
                                    types_in_method!(method),
                                    return_types!(method),
                                ));
                            }
                        }
                        assert!(
                            !file_data.contains_key(&name),
                            "Multiple definitions of {}",
                            &name
                        ); // make sure no other struct has the same name
                        file_data.insert(name.clone(), TypeHolder::Trait(trait_data));
                    }
                    _ => {
                        //todo
                    }
                }
            }
        };
        visit_dirs(&self.starting_point, &mut closure).expect("Unable to read directory");
        //create interface file
        let mut holder = ItemsHolder::new(file_data.len());
        //file_data.iter().for_each(|it| println!("it {:?}", it));
        for (name, type_holder) in file_data {
            match type_holder {
                TypeHolder::Struct(_) | TypeHolder::Trait(_) => {
                    holder.add_items(name, type_holder);
                }
                TypeHolder::Enum(val) => holder.add_enum(val),
            }
        }
        holder.generate_interface(language, &self.interface_file_path);
        println!("Total Time Taken To Generate File {:?}", start.elapsed());
    }

    fn impl_data(map: &mut HashMap<Rc<String>, TypeHolder>, item: &syn::ItemImpl) {
        let self_type = &*item.self_ty;
        if let syn::Type::Path(type_path) = self_type {
            let name = type_path
                .path
                .segments
                .iter()
                .next()
                .and_then(|it| Some(it.ident.to_string()));
            if let Some(name) = name {
                //name of struct or enum
                for item in item.items.iter() {
                    match item {
                        syn::ImplItem::Method(method) => {
                            let method_info: AttrCheck = has_gen_attr!(method, true);
                            //not supporting enums for now
                            if !method_info.is_attribute {
                                continue;
                            }
                            let method_name = (&method).sig.ident.to_string();
                            let data = map.get_mut(&name);
                            let item_info = ItemInfo::new_method(
                                function_signature!(method),
                                get_doc!(method),
                                method_name,
                                method_info.is_constructor,
                                types_in_method!(method),
                                return_types!(method),
                            );
                            if let Some(data) = data {
                                match data {
                                    TypeHolder::Struct(val) => {
                                        val.extras.push(item_info);
                                    }
                                    _ => {
                                        unimplemented!("Impl function may only be used for structs")
                                    }
                                }
                            } else {
                                //impl block came before struct definition (properly due to the order in which the files
                                // were read)
                                //we're assuming the impl method is for a struct
                                //if it's for an enum, it would crash in the enum function
                                let data = Struct::new(
                                    name.to_string(),
                                    Types::Struct,
                                    vec![],
                                    vec![item_info],
                                );
                                map.insert(Rc::new(name.clone()), TypeHolder::Struct(data));
                            }
                        }
                        _ => {}
                    }
                }
            }
        };
    }
}

/*
todo!()
#[cfg(test)]
mod tests {
    use crate::generator_lib::AttrCheck;
    use syn::__private::ToTokens;
    use syn::{Item, PathArguments, ReturnType, Type};

    #[test]
    fn test_syn() {
        let item = syn::parse_str::<syn::Item>(
            "
#[generate_interface_doc]
#[derive(Debug)]
///struct us good
///
struct Kofi {
    kkkk: i64,
}
",
        )
        .unwrap();
        ////println!("{:#?}", item);
        let item = syn::parse_str::<syn::Item>(
            "#[generate_interface(here)]
    ///nice docs
    fn not() {
        unimplemented!()
    }",
        )
        .unwrap();
        //println!("{:#?}", item);

        if let syn::Item::Fn(val) = item {
            let y = val
                .attrs
                .iter()
                .map(|it| it.tokens.to_string())
                //.filter(|it| it.contains("constructor"))
                .collect::<Vec<String>>();
            val.attrs.iter().any(|it| {
                let is_attribute = it
                    .path
                    .segments
                    .iter()
                    .any(|it| it.ident == "generate_interface");
                //println!("any {}", is_attribute);
                is_attribute
            });
            for attribute in val.attrs.iter() {
                let is_attribute = attribute
                    .path
                    .segments
                    .iter()
                    .any(|it| it.ident == "generate_interface");
                //println!("is {}", is_attribute);
            }
            //println!("vec {:?} has {:?}", y, has_gen_attr!(val, true));
        }
        panic!()
    }

    #[test]
    fn test_types_in_method() {
        let item = syn::parse_str::<Item>("fn others(h: Box<Box<i64,FOK>>, j: OTH) {}").unwrap();
        let item1 = syn::parse_str::<Item>(
            "unsafe fn new(kkkk: i64, k: Trial) -> HashMap<i64,Kofi> {
        Kofi { kkkk }
    }",
        )
        .unwrap();
        let item2 = syn::parse_str::<Item>(
            "unsafe fn new(kkkk: i64, k: Trial) -> Kofi {
        Kofi { kkkk }
    }",
        )
        .unwrap();
        if let syn::Item::Fn(val) = item1 {
            let vecs = types_in_method!(val);
            ////println!("types in {:?} {:?}", vecs, return_types!(val));
            match val.sig.output {
                ReturnType::Default => {}
                ReturnType::Type(_, val) => match &*val {
                    Type::Path(val) => {
                        //let result = vec![];
                        val.path.segments.iter().for_each(|it| {
                            //println!("ident {}", &it.ident);
                            match &it.arguments {
                                PathArguments::None => {}
                                PathArguments::AngleBracketed(val) => {
                                    let str = val.args.to_token_stream().to_string();
                                    val.args.iter().for_each(|it| {
                                        let str = it.to_token_stream().to_string();
                                        //println!("str {}", str);
                                    });
                                }
                                PathArguments::Parenthesized(_) => {}
                            }
                        });
                    }
                    _ => {}
                },
            }
        }
    }
}
*/