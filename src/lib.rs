use syn::parse_quote;
use proc_macro::TokenStream;
use quote::quote;
#[cfg(not(target_arch = "wasm32"))]
use syn::parse_str;
use syn::{ImplItem, Item, ItemImpl};


#[proc_macro_attribute]
pub fn bindclass(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Item);
    TokenStream::from(quote!{
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
        #[cfg_attr(not(target_arch = "wasm32"), pyclass)]
        #input
    })
}

#[proc_macro_attribute]
pub fn bindimpl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_impl = syn::parse_macro_input!(input as ItemImpl);

    #[cfg(not(target_arch = "wasm32"))]
    for i in &mut input_impl.items {
        if let ImplItem::Fn(f) = i {

            if f.attrs.iter().any(|a| {
                a.path().is_ident("constructor")
            }) {
                f.attrs.retain(|a| !a.path().is_ident("constructor"));
                f.attrs.push(parse_quote!(
                    #[new]
                ));
            } 

        }
    }

    #[cfg(target_arch = "wasm32")]
    for i in &mut input_impl.items {
        if let ImplItem::Fn(f) = i {

            if f.attrs.iter().any(|a| {
                a.path().is_ident("constructor")
            }) {
                f.attrs.retain(|a| !a.path().is_ident("constructor"));
                f.attrs.push(parse_quote!(
                    #[wasm_bindgen(constructor)]
                ));
            } 

        }
    }

    TokenStream::from(quote!{
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
        #[cfg_attr(not(target_arch = "wasm32"), pymethods)]
        #input_impl
    })

}