extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn generate_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    let mut is_func = false;

    match item {
        syn::Item::Fn(ref fun) => {
            //general function
            is_func = true;
            //generics not supported
            let gene = &fun.sig.generics;
            assert!(gene.gt_token.is_none(), "Generics not yet supported");
            assert!(gene.lt_token.is_none(), "Generics not yet supported")
        }
        syn::Item::Enum(_) => {}
        syn::Item::Trait(_) => {}
        syn::Item::Struct(_) => panic!(
            "Annotate methods of this struct instead. \
        To use enable doc comments on this struct use #[generate_interface_doc] macro instead."
        ),
        _ => panic!("unsuppoted type"),
    }
    let attr = attr.to_string();
    if !attr.is_empty() {
        assert_eq!(
            attr, "constructor",
            "only constructor attributes are supported for now"
        );
        if !is_func {
            panic!("call constructor on function")
        }
    }
    let y = quote::quote! {
        #item
    };
    y.into()
}

#[proc_macro_attribute]
pub fn generate_interface_doc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    match item {
        syn::Item::Struct(_) => {}
        _ => panic!("Use this macro on only struct`"),
    }
    assert!(attr.is_empty(), "No attributes allowed yet");
    let fin = quote::quote! {
        #item
    };
    fin.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        todo!()
    }
}
