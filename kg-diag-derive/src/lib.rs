#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;

use kg_diag::*;

use proc_macro2::Span;
use std::convert::TryFrom;


decl_derive!([Detail, attributes(diag)] => detail_derive);

struct DiagAttr {
    code: u32,
    severity: Severity,
}

fn detail_derive(mut st: synstructure::Structure) -> proc_macro2::TokenStream {
    let mut code_offset: u32 = 0;
    let mut severity = Severity::Failure;

    let container_attr = find_nested_attr(&st.ast().attrs, "diag");
    if let Some(params) = container_attr {
        for p in params {
            match p {
                syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, lit: syn::Lit::Int(ref i), .. })) if ident == "code_offset" => {
                    code_offset = i.value() as u32;
                }
                syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, lit: syn::Lit::Str(ref s), .. })) if ident == "severity" => {
                    match Severity::try_from(s.value().as_ref()) {
                        Ok(s) => severity = s,
                        Err(value) => panic!(format!("invalid default severity \"{}\" for type {}", value, st.ast().ident)),
                    }
                }
                syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, lit: syn::Lit::Char(ref c), .. })) if ident == "severity" => {
                    match Severity::try_from(c.value()) {
                        Ok(s) => severity = s,
                        Err(value) => panic!(format!("invalid default severity '{}' for type {}", value, st.ast().ident)),
                    }
                }
                _ => {
                    panic!(format!("invalid diag(...) attribute for type {}", st.ast().ident));
                }
            }
        }
    }

    let mut attrs = Vec::with_capacity(st.variants().len());
    let mut code = code_offset + 1;

    for ref mut v in st.variants_mut() {
        v.filter(|_| false);

        let mut a = DiagAttr {
            code,
            severity,
        };

        let vattr = find_nested_attr(v.ast().attrs, "diag");
        if let Some(params) = vattr {
            for p in params {
                match p {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, lit: syn::Lit::Int(ref i), .. })) if ident == "code" => {
                        a.code = code_offset + i.value() as u32;
                    }
                    syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, lit: syn::Lit::Str(ref s), .. })) if ident == "severity" => {
                        match Severity::try_from(s.value().as_ref()) {
                            Ok(s) => a.severity = s,
                            Err(value) => panic!(format!("invalid severity \"{}\" for variant {}", value, v.ast().ident)),
                        }
                    }
                    syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, lit: syn::Lit::Char(ref c), .. })) if ident == "severity" => {
                        match Severity::try_from(c.value()) {
                            Ok(s) => a.severity = s,
                            Err(value) => panic!(format!("invalid severity '{}' for variant {}", value, v.ast().ident)),
                        }
                    }
                    _ => {
                        panic!(format!("invalid diag(...) attribute for variant {}", v.ast().ident));
                    }
                }
            }
        }

        if a.code > code {
            code = a.code + 1;
        } else {
            code += 1;
        }

        attrs.push(a);
    }

    for a in attrs.iter() {
        for b in attrs.iter() {
            if a as *const _ == b as *const _ {
                continue;
            }
            if a.code == b.code {
                panic!(format!("duplicated code {} in type {}", a.code, st.ast().ident));
            }
        }
    }

    let mut attrs_it = attrs.iter();
    let severity_body = st.each_variant(|_v| {
        let a = attrs_it.next().unwrap();
        let severity = syn::parse_str::<syn::Path>(&format!("kg_diag::Severity::{:?}", a.severity)).unwrap();
        quote! { #severity }
    });

    let mut attrs_it = attrs.iter();
    let code_body = st.each_variant(|_v| {
        let a = attrs_it.next().unwrap();
        let code = a.code;
        quote! { #code }
    });

    let p = st.gen_impl(quote! {
        extern crate kg_diag;

        gen impl kg_diag::Detail for @Self {
            fn severity(&self) -> Severity {
                match *self {
                    #severity_body
                }
            }

            fn code(&self) -> u32 {
                match *self {
                    #code_body
                }
            }
        }
    });

    p
}


fn find_nested_attr(attrs: &[syn::Attribute], id: &str) -> Option<Vec<syn::NestedMeta>> {
    let doc_path: syn::Path = syn::Ident::new("doc", Span::call_site()).into();

    let mut a = None;
    for attr in attrs {
        if attr.path != doc_path && attr.style == syn::AttrStyle::Outer {
            let meta = {
                let m = attr.interpret_meta();
                if let Some(syn::Meta::List(syn::MetaList { ident, nested, .. })) = m {
                    if ident == id {
                        Some(nested.into_iter().collect())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            if a.is_some() && meta.is_some() {
                panic!(format!("multiple {}(...) attributes found", id))
            } else {
                a = meta;
            }
        }
    }
    a
}
