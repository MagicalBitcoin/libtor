extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use syn::{parse_macro_input, DeriveInput, Data, Fields};
use syn::spanned::Spanned;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};

#[proc_macro_derive(Expand, attributes(expand_to))]
pub fn derive_helper_attr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    //println!("{:#?}", input);
    let enum_name = &input.ident;

    let match_body = match input.data {
        Data::Enum(data) => {
            let mut stream = TokenStream::new();
            for variant in data.variants {
                let span = &variant.span();
                let name = &variant.ident;
                let name_string = name.to_string();

                let tokens = if variant.attrs.is_empty() {
                    if let Fields::Named(_) = variant.fields {
                        quote_spanned!{*span=>
                            #enum_name::#name{..} => compile_error!("Named fields require an explicit expansion attribute"),
                        }
                    } else if let Fields::Unnamed(fields) = variant.fields {
                        let fmt_str = (0..fields.unnamed.len()).into_iter().map(|_| "{}").collect::<Vec<&str>>().join(" ");
                        let fmt_str = format!("{{}} \"{}\"", fmt_str); // {cmdName} + Wrap all the params between quotes
                        let fmt_params = (0..fields.unnamed.len()).into_iter().map(|i| Ident::new(&format!("p_{}", i), i.span()));
                        let expand_params = fmt_params.clone();

                        quote_spanned!{*span=>
                            #enum_name::#name(#(#expand_params, )*) => format!(#fmt_str, #name_string, #(#fmt_params, )*),
                        }
                    } else {
                        quote!{
                            #enum_name::#name => #name_string.to_string(),
                        }
                    }
                } else {
                    let attr = &variant.attrs[0];
                    let args: TokenStream = attr.parse_args().unwrap();

                    if let Fields::Named(fields) = variant.fields {
                        let ignore_filter = |field: &&syn::Field| {
                            !field.attrs.iter().any(|a| {
                                a.parse_args::<syn::Ident>().and_then(|ident| Ok(ident == "ignore")).unwrap_or(false)
                            })
                        };

                        let fmt_params = fields.named.iter().filter(ignore_filter).map(|f| {
                            let ident = &f.ident;
                            quote_spanned!{f.span()=> #ident = #ident }
                        });
                        let expand_params = fields.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote_spanned!{f.span()=> #ident }
                        });

                        quote_spanned!{attr.span()=>
                            #enum_name::#name{#(#expand_params,)*} => format!(#args, #(#fmt_params, )*),
                        }
                    } else if let Fields::Unnamed(fields) = variant.fields {
                        let fmt_params = (0..fields.unnamed.len()).into_iter().map(|i| Ident::new(&format!("p_{}", i), i.span()));
                        let expand_params = fmt_params.clone();

                        quote_spanned!{*span=>
                            #enum_name::#name(#(#expand_params, )*) => format!(#args, #(#fmt_params, )*),
                        }
                    } else {
                        quote!{
                            #enum_name::#name => #args.to_string(),
                        }
                    }
                };

                stream.extend(TokenStream::from(tokens));
            }

            stream
        },
        _ => unimplemented!(),
    };
    let match_body = TokenStream::from(match_body);

    let name = input.ident;
    let expanded = quote! {
        impl Expand for #name {
            fn expand(&self) -> String {
                #[allow(unused)]
                match self {
                    #match_body
                }
            }
        }
    };
    
    proc_macro::TokenStream::from(expanded)
}
