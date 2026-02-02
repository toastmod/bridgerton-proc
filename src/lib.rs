use std::sync::atomic::AtomicBool;

use syn::{Attribute, Ident, Stmt, Type, parse_quote, token::RArrow};
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
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

    #[cfg(feature = "wasm")]
    let target_arch_is_wasm = true; 

    #[cfg(feature = "python")]
    let target_arch_is_wasm = false; 

    if target_arch_is_wasm {
        println!("Compiling with wasm-bindgen bindings.")
    } else {
        println!("Compiling with pyo3 bindings.")
    }

    let mut input_impl = syn::parse_macro_input!(input as ItemImpl);

    for i in &mut input_impl.items {
        if let ImplItem::Fn(f) = i {

            // Constructor handler
            if f.attrs.iter().any(|a| {
                a.path().is_ident("constructor")
            }) {
                f.attrs.retain(|a| !a.path().is_ident("constructor"));

                if !target_arch_is_wasm {
                    f.attrs.push(parse_quote!(
                        #[new]
                    ));
                }

                if target_arch_is_wasm {
                    f.attrs.push(parse_quote!(
                        #[wasm_bindgen(constructor)]
                    ));
                }
            } 

            // getter handler
            let target_attr = "bindget";
            let mut enable_clone = false;
            if f.attrs.iter().any(|a| {
                if a.path().is_ident(target_attr) {
                    true
                } else if a.path().is_ident("bindget_clone") {
                    enable_clone = true;
                    true
                } else {
                    false
                }

            }) {

                let arg = f.attrs.extract_if(..,|a| a.path().is_ident(target_attr) || a.path().is_ident("bindget_clone")).next().unwrap().parse_args::<Ident>().unwrap();

                if !target_arch_is_wasm {
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
                    
                    if f.block.stmts.is_empty() {
    
                        if enable_clone {
                            f.block = parse_quote!({
                                Ok(
                                    self
                                    .#arg
                                    .clone()
                                )
                            });
                        } else {
                            f.block = parse_quote!({
                                Ok(
                                    self
                                    .#arg
                                )
                            });
                        }
    
                    }

                }

                if target_arch_is_wasm {
                    f.attrs.push(parse_quote!(
                        #[wasm_bindgen(getter = #arg)]
                    ));

                    if f.block.stmts.is_empty() {
    
                        if enable_clone {
                            f.block = parse_quote!({
                                self
                                .#arg
                                .clone()
                            });
                        } else {
                            f.block = parse_quote!({
                                self
                                .#arg
                            });
                        }
    
                    }
                }

                f.sig.ident = format_ident!("get_{}",arg);


            } 

            // setter handler
            let target_attr = "bindset";
            if f.attrs.iter().any(|a| {
                a.path().is_ident(target_attr)
            }) {
                println!("SETTER");
                let arg = f.attrs.extract_if(..,|a| a.path().is_ident(target_attr)).next().unwrap().parse_args::<Ident>().unwrap();

                if !target_arch_is_wasm {
                    f.block = parse_quote!({
                        self.#arg = v;
                    });
                    f.attrs.push(parse_quote!(
                        #[setter]
                    ));
                }

                if target_arch_is_wasm {
                    f.block = parse_quote!({
                        self.#arg = v;
                    });
                    f.attrs.push(parse_quote!(
                        #[wasm_bindgen(setter = #arg)]
                    ));
                }

                f.sig.ident = format_ident!("set_{}",arg);
            }

        }

        
    }

    TokenStream::from(quote!{
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
        #[cfg_attr(not(target_arch = "wasm32"), pymethods)]
        #input_impl
    })

}