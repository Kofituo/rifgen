use itertools::MultiUnzip;
use quote::format_ident;
use syn::ItemImpl;

pub fn generate_impl_block(item: &syn::ItemStruct) -> ItemImpl {
    let name = item.clone().ident;
    let vis = item.clone().vis;
    let fields = match item.clone().fields {
        syn::Fields::Named(fields) => fields.named.into_iter().collect::<Vec<_>>(),
        _ => unreachable!(),
    };

    let (f_setter, f_getter, f_ident, f_vis, f_ty): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
        fields
            .iter()
            .cloned()
            .filter_map(|f| {
                if let Some(ident) = f.ident {
                    Some((
                        format_ident!("set_{}", ident),
                        format_ident!("get_{}", ident),
                        ident,
                        f.vis,
                        f.ty,
                    ))
                } else {
                    None
                }
            })
            .multiunzip();
    let impl_block = quote::quote! {
         impl #name {
            #[generate_interface(constructor)]
            #vis fn new(
                #(#f_ident: #f_ty),*
            ) -> #name {
                #name {
                    #(#f_ident),*
                }
            }
            #(
                #[generate_interface]
                #f_vis fn #f_setter(&mut self, #f_ident: #f_ty) {
                    self.#f_ident = #f_ident;
                }

                #[generate_interface]
                #f_vis fn #f_getter(&self) -> &#f_ty {
                    &self.#f_ident
                }
            )*
        }
    };

    syn::parse2(impl_block).unwrap()
}
