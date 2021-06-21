extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{braced, parenthesized, parse_macro_input, token, Data, DeriveInput, Fields, Token};

#[cfg_attr(feature = "debug", derive(Debug))]
struct ExpandToArg {
    keyword: Ident,
    name: syn::Lit,
}

impl Parse for ExpandToArg {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let keyword = input.parse()?;
        input.parse::<Token![=]>()?;
        let name = input.parse()?;

        Ok(ExpandToArg { keyword, name })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct TestStruct {
    args_group: Option<TokenStream>,
    expected: syn::LitStr,
}

impl Parse for TestStruct {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let keyword: Ident = input.parse()?;
        if keyword != "test" {
            return Err(syn::Error::new(keyword.span(), "expected `test`"));
        }
        input.parse::<Token![=]>()?;

        let args_group: Option<TokenStream> = if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let content: TokenStream = content.parse()?;

            Some(quote! {
                { #content }
            })
        } else if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let content: TokenStream = content.parse()?;

            Some(quote! {
                ( #content )
            })
        } else {
            None
        };

        input.parse::<Token![=>]>()?;

        let expected = input.parse()?;
        if let syn::Lit::Str(expected) = expected {
            Ok(TestStruct {
                args_group,
                expected,
            })
        } else {
            Err(syn::Error::new(keyword.span(), "expected a string literal"))
        }
    }
}

fn split_first_space_args(val: TokenStream) -> TokenStream {
    quote! {
        {
            let formatted = #val;
            let parts = formatted.splitn(2, " ").collect::<Vec<_>>();

            let mut answer = vec![parts[0].to_string()];
            if let Some(part) = parts.get(1) {
                answer.push(part.to_string());
            }

            answer
        }
    }
}

fn generate_test(
    parsed: TestStruct,
    test_count: usize,
    enum_name: &Ident,
    name: &Ident,
    span: Span,
) -> TokenStream {
    let test_name = format_ident!("TEST_{}_{}", name, test_count);
    let args_group = &parsed.args_group.unwrap_or_default();
    let expected = &parsed.expected;

    quote_spanned! {span=>
        #[test]
        fn #test_name() {
            use Expand;

            let v = #enum_name::#name#args_group;
            println!("{:?} => {}", v, v.expand_cli());
            assert_eq!(v.expand_cli(), #expected);
        }
    }
}

#[proc_macro_derive(Expand, attributes(expand_to))]
pub fn derive_helper_attr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let (match_body, test_funcs) = match input.data {
        Data::Enum(data) => {
            let mut stream = TokenStream::new();
            let mut test_stream = TokenStream::new();

            for variant in data.variants {
                let span = &variant.span();
                let name = &variant.ident;
                let mut name_string = name.to_string();
                let mut test_count = 0;
                let mut implemented_with = false;

                let mut fmt_attr = None;

                for attr in &variant.attrs {
                    if attr.path.get_ident() != Some(&format_ident!("expand_to")) {
                        continue;
                    }

                    if attr.parse_args::<syn::Lit>().is_ok() {
                        fmt_attr = Some(attr);
                    } else if let Ok(arg) = attr.parse_args::<ExpandToArg>() {
                        if arg.keyword == "rename" {
                            if let syn::Lit::Str(lit_str) = arg.name {
                                name_string = lit_str.value();
                            } else {
                                let tokens = quote_spanned! {*span=>
                                    #enum_name::#name{..} => compile_error!("`rename` must be followed by a string literal, eg #[expand_to(rename = \"example\")]"),
                                };
                                stream.extend(tokens);
                            }
                        } else if arg.keyword == "with" {
                            let tokens = if let syn::Lit::Str(lit_str) = arg.name {
                                let ident = Ident::new(&lit_str.value(), *span);

                                let matcher = match variant.fields {
                                    Fields::Unnamed(_) => quote! {
                                        #enum_name::#name(..)
                                    },
                                    Fields::Named(_) => quote! {
                                        #enum_name::#name{..}
                                    },
                                    Fields::Unit => quote! {
                                        #enum_name::#name()
                                    },
                                };

                                quote_spanned! {*span=>
                                    #matcher => #ident(self),
                                }
                            } else {
                                quote_spanned! {*span=>
                                    #enum_name::#name{..} => compile_error!("`with` must be followed by a string literal, eg #[expand_to(with = \"my_custom_function\")]"),
                                }
                            };

                            stream.extend(tokens);
                            implemented_with = true;
                        }
                    } else {
                        // TODO: add those example as doc attributes
                        if let Ok(parsed) = attr.parse_args::<TestStruct>() {
                            test_stream
                                .extend(generate_test(parsed, test_count, enum_name, name, *span));
                            test_count += 1;
                        }
                    }
                }

                if implemented_with {
                    continue;
                }

                let ignore_filter = |field: &&syn::Field| {
                    !field.attrs.iter().any(|a| {
                        a.parse_args::<syn::Ident>()
                            .and_then(|ident| Ok(ident == "ignore"))
                            .unwrap_or(false)
                    })
                };
                let tokens = match (variant.fields, fmt_attr) {
                    (Fields::Named(_), None) => {
                        quote_spanned! {*span=>
                            #enum_name::#name{..} => compile_error!("Named fields require an explicit expansion attribute"),
                        }
                    }
                    (Fields::Named(fields), Some(attr)) => {
                        let args: TokenStream = attr.parse_args().unwrap();

                        let fmt_params = fields.named.iter().filter(ignore_filter).map(|f| {
                            let ident = &f.ident;
                            quote_spanned! {f.span()=> #ident = #ident }
                        });
                        let expand_params = fields.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote_spanned! {f.span()=> #ident }
                        });

                        let fmt_str_quoted = quote! { format!(#args, #(#fmt_params, )*) };
                        let content = split_first_space_args(fmt_str_quoted);
                        quote_spanned! {attr.span()=>
                            #enum_name::#name{#(#expand_params, )*} => {
                                #content
                            },
                        }
                    }
                    (Fields::Unnamed(fields), attr) => {
                        let expand_params = (0..fields.unnamed.len())
                            .map(|i| Ident::new(&format!("p_{}", i), i.span()));
                        let fmt_params = (0..fields.unnamed.len())
                            .filter(|i| ignore_filter(&&fields.unnamed[*i]))
                            .map(|i| Ident::new(&format!("p_{}", i), i.span()));

                        if let Some(attr) = attr {
                            let args: TokenStream = attr.parse_args().unwrap();
                            let fmt_str_quoted = quote! { format!(#args, #(#fmt_params, )*) };
                            let content = split_first_space_args(fmt_str_quoted);
                            quote_spanned! {*span=>
                                #enum_name::#name(#(#expand_params, )*) => {
                                    #content
                                },
                            }
                        } else {
                            let fmt_str = (0..fields.unnamed.len())
                                .map(|_| "{}")
                                .collect::<Vec<&str>>()
                                .join(" ");
                            quote_spanned! {*span=>
                                #enum_name::#name(#(#expand_params, )*) => vec![#name_string.to_string(), format!(#fmt_str, #(#fmt_params, )*)],
                            }
                        }
                    }
                    (Fields::Unit, None) => quote! {
                        #enum_name::#name => vec![#name_string.to_string()],
                    },
                    (Fields::Unit, Some(attr)) => {
                        let args: TokenStream = attr.parse_args().unwrap();
                        let args_str_quoted = quote! { #args.to_string() };
                        let content = split_first_space_args(args_str_quoted);
                        quote! {
                            #enum_name::#name => #content,
                        }
                    }
                };

                stream.extend(tokens);
            }

            (stream, test_stream)
        }
        _ => unimplemented!(),
    };

    let test_mod_name = Ident::new(
        &format!("_GENERATED_TESTS_FOR_{}", enum_name),
        enum_name.span(),
    );

    let name = input.ident;
    let expanded = quote! {
        impl Expand for #name {
            fn expand(&self) -> Vec<String> {
                #[allow(unused)]
                #[allow(clippy::useless_format)]
                match self {
                    #match_body
                }
            }
        }

        #[cfg(test)]
        #[allow(non_snake_case)]
        mod #test_mod_name {
            use super::*;

            #test_funcs
        }
    };

    proc_macro::TokenStream::from(expanded)
}
