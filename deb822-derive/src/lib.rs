extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use syn::{Data, Type, TypePath};

fn is_option(ty: &syn::Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

// Generate `from_paragraph`, ``to_paragraph`` methods for the annotated struct, i.e.:
//
// ```rust
// #[derive(Deb822)]
// struct X {
//    a: i32,
//    b: i32,
//    c: Option<String>,
//    d: Vec<String>,
//    e: bool,
// }
// ```
//
// will generate:
//
// ```rust
//
// impl FromDeb822Paragraph for X {
//     fn from_paragraph(para: &deb822_lossless::Paragraph) -> Result<Self, String> {
//     Ok(Self {
//         a: para.get("a").ok_or_else(|| "missing field: a")?.parse().map_err(|e| format!("parsing field a: {}", e))?,
//         b: para.get("b").ok_or_else(|| "missing field: b")?.parse().map_err(|e| format!("parsing field b: {}", e))?,
//         c: para.get("c").map(|v| v.parse().map_err(|e| format!("parsing field c: {}", e))).transpose()?,
//         d: para.get("d").ok_or_else(|| "missing field: d")?.split_whitespace().map(|s| s.to_string()).collect(),
//         e: para.get("e").ok_or_else(|| "missing field: e")?.parse().map_err(|e| format!("parsing field e: {}", e))?,
//     })
// }
//
// impl ToDeb822Paragraph for X {
//     fn to_paragraph(&self) -> deb822_lossless::Paragraph {
//         let mut fields = Vec::new();
//         fields.insert("a", &self.a.to_string());
//         fields.insert("b", &self.b.to_string());
//         if let Some(v) = &self.c {
//             fields.insert("c", &v.to_string());
//         }
//         fields.insert("d", &self.d.join(" "));
//         fields.insert("e", &self.e.to_string());
//         deb822_lossless::Paragraph::from(fields)
//     }
//
//     fn update_paragraph(&self, para: &mut deb822_lossless::Paragraph) {
//         para.insert("a", &self.a.to_string());
//         para.insert("b", &self.b.to_string());
//         if let Some(v) = &self.c {
//             para.insert("c", &v.to_string());
//         } else {
//             para.remove("c");
//         }
//         para.insert("d", &self.d.join(" "));
//         para.insert("e", &self.e.to_string());
//     }
// }
// ```

#[proc_macro_derive(Deb822)]
pub fn derive_deb822(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let s = if let syn::Data::Struct(s) = &input.data {
        s
    } else {
        panic!("Deb822 can only be derived for structs")
    };

    let from_fields = s.fields.iter().map(|f| {
            let ident = &f.ident;
            let key = ident.as_ref().unwrap().to_string();
            // Check if the field is optional or not
            let ty = &f.ty;
            let is_option = is_option(ty);
            if is_option {
                // Allow the field to be missing
                quote! {
                    #ident: para.get(#key).map(|v| v.parse().map_err(|e| format!("parsing field {}: {}", #key, e))).transpose()?
                }
            } else {
                // The field is required
                quote! {
                    #ident: para.get(#key).ok_or_else(|| format!("missing field: {}", #key))?.parse().map_err(|e| format!("parsing field {}: {}", #key, e))?
                }
            }
        }).collect::<Vec<_>>();

    let to_fields = s.fields.iter().map(|f| {
        let ident = &f.ident;
        let key = ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        let is_option = is_option(ty);

        if is_option {
            quote! {
                if let Some(v) = &self.#ident {
                    fields.push((#key.to_string(), v.to_string()));
                }
            }
        } else {
            quote! {
                fields.push((#key.to_string(), self.#ident.to_string()));
            }
        }
    }).collect::<Vec<_>>();

    let update_fields = s.fields.iter().map(|f| {
        let ident = &f.ident;
        let key = ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        let is_option = is_option(ty);

        if is_option {
            quote! {
                if let Some(v) = &self.#ident {
                    para.insert(#key, &v.to_string());
                } else {
                    para.remove(#key);
                }
            }
        } else {
            quote! {
                para.insert(#key, &self.#ident.to_string());
            }
        }
    }).collect::<Vec<_>>();

    let gen = quote! {
        impl deb822_lossless::convert::FromDeb822Paragraph for #name {
            fn from_paragraph(para: &deb822_lossless::Paragraph) -> Result<Self, String> {
                Ok(Self {
                    #(#from_fields,)*
                })
            }
        }

        impl deb822_lossless::convert::ToDeb822Paragraph for #name {
            fn to_paragraph(&self) -> deb822_lossless::Paragraph {
                let mut fields = Vec::new();
                #(#to_fields)*
                deb822_lossless::Paragraph::from(fields)
            }

            fn update_paragraph(&self, para: &mut deb822_lossless::Paragraph) {
                #(#update_fields)*
            }
        }
    };
    gen.into()
}
