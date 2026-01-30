use syn::{Attribute, Ident, Stmt, Type, parse_quote, token::RArrow};
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
#[cfg(not(target_arch = "wasm32"))]
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

    for i in &mut input_impl.items {
        if let ImplItem::Fn(f) = i {

            // Constructor handler
            if f.attrs.iter().any(|a| {
                a.path().is_ident("constructor")
            }) {
                f.attrs.retain(|a| !a.path().is_ident("constructor"));

                #[cfg(not(target_arch = "wasm32"))]
                f.attrs.push(parse_quote!(
                    #[new]
                ));

                #[cfg(target_arch = "wasm32")]
                f.attrs.push(parse_quote!(
                    #[wasm_bindgen(constructor)]
                ));
            } 

            // getter handler
            let target_attr = "bindget";
            if f.attrs.iter().any(|a| {
                a.path().is_ident(target_attr)
            }) {
                let args = f.attrs.extract_if(..,|a| a.path().is_ident(target_attr)).next().unwrap().parse_args::<Ident>().unwrap();

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let fixed_return: Type = match &f.sig.output {
                        syn::ReturnType::Default => {
    
                            parse_quote! {PyResult<()>}
                        },
                        syn::ReturnType::Type(_, ty) => {
                            let ty = ty.clone();
                            parse_quote!{PyResult<#ty>}
                        },
                    };
    
                    f.sig.output = syn::ReturnType::Type(RArrow::default(), Box::new(fixed_return as Type));
                    f.attrs.push(parse_quote!(
                        #[getter]
                    ));
                }

                #[cfg(target_arch = "wasm32")]
                f.attrs.push(parse_quote!(
                    #[wasm_bindgen(getter = #arg)]
                ));
            } 

            // setter handler
            let target_attr = "bindset";
            if f.attrs.iter().any(|a| {
                a.path().is_ident(target_attr)
            }) {
                let arg = f.attrs.extract_if(..,|a| a.path().is_ident(target_attr)).next().unwrap().parse_args::<Ident>().unwrap();

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let fixed_return: Type = match &f.sig.output {
                        syn::ReturnType::Default => {
    
                            parse_quote! {PyResult<()>}
                        },
                        syn::ReturnType::Type(_, ty) => {
                            let ty = ty.clone();
                            parse_quote!{PyResult<#ty>}
                        },
                    };
    
                    f.sig.output = syn::ReturnType::Type(RArrow::default(), Box::new(fixed_return as Type));
                    f.attrs.push(parse_quote!(
                        #[setter]
                    ));
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let fixed_return: Type = match &f.sig.output {
                        syn::ReturnType::Default => {
    
                            parse_quote! {Result<(), JsValue>}
                        },
                        syn::ReturnType::Type(_, ty) => {
                            let ty = ty.clone();
                            parse_quote!{Result<#ty, JsValue>}
                        },
                    };
    
                    f.sig.output = syn::ReturnType::Type(RArrow::default(), Box::new(fixed_return as Type));
                    f.attrs.push(parse_quote!(
                        #[wasm_bindgen(setter = #arg)]
                    ));
                }
            } 

        }

    }

    TokenStream::from(quote!{
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
        #[cfg_attr(not(target_arch = "wasm32"), pymethods)]
        #input_impl
    })

}