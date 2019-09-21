use heck::CamelCase;
use log::{debug, error, warn};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use syn::punctuated::Punctuated;

fn gen_changeset_ident(ty: &syn::Ident) -> syn::Ident {
    let v = format!("{}_Changeset", ty.to_string()).to_camel_case();
    syn::Ident::new(&v, proc_macro2::Span::call_site())
}

fn gen_changeset_path(ty: &syn::Path) -> syn::Path {
    let mut path = ty.to_owned();
    let mut last_segment = path.segments.last_mut().unwrap();

    let s = last_segment.ident.to_string();
    let a = last_segment.arguments.clone();

    let v = format!("{}_Changeset", &s).to_camel_case();
    last_segment.ident = syn::Ident::new(&v, proc_macro2::Span::call_site());
    path
}

fn gen_changes(field: &syn::Field) -> TokenStream {
    let field_name = &field.ident;

    quote! {
        changes.#field_name = self.#field_name.changeset(&other.#field_name);
    }
}

fn gen_impl_diff(ty: &syn::Ident, fields: &Punctuated<syn::Field, syn::Token![,]>) -> TokenStream {
    let change_items = fields.iter().map(gen_changes);
    let changeset_ident = gen_changeset_ident(&ty);

    quote! {
        impl structdiff::Diff for #ty {
            type Changeset = #changeset_ident;
            type Action = ();

            fn changeset(&self, other: &Self) -> structdiff::Field<Self, Self::Changeset, Self::Action>
            where
                Self: Sized
            {
                if self == other {
                    return structdiff::Field::None
                }

                let mut changes = Self::Changeset::default();

                #(#change_items)*

                structdiff::Field::Changes(changes)
            }
        }
    }
}

fn first_generic_from_type_path(ty: &syn::Type) -> Option<syn::Type> {
    let path = match ty {
        syn::Type::Path(path) => &path.path,
        _ => return None,
    };

    let last = path.segments.last()?;
    match &last.arguments {
        syn::PathArguments::AngleBracketed(args) => args.args.iter().find_map(|x| match x {
            syn::GenericArgument::Type(ty) => Some(ty.clone()),
            _ => None,
        }),
        _ => None,
    }
}

fn gen_changeset_struct(
    ty: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
) -> Result<TokenStream, syn::Error> {
    let ty_name = gen_changeset_ident(&ty);

    let mappings = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            let ty_changeset = match &ty {
                syn::Type::Path(path) => gen_changeset_path(&path.path),
                _ => {
                    return Err(syn::Error::new_spanned(
                        field,
                        "Only path types are supported",
                    ));
                }
            };

            let ty_action = if ty_changeset
                .segments
                .last()
                .as_ref()
                .unwrap()
                .ident
                .to_string()
                .starts_with("Vec")
            {
                let ty = first_generic_from_type_path(ty);
                quote! { VecAction<#ty> }
            } else {
                quote! { () }
            };

            Ok(quote! { #ident : structdiff::Field<#ty, #ty_changeset, #ty_action> })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #[automatically_derived]
        #[derive(Debug, Default)]
        pub struct #ty_name {
            #(#mappings),*
        }
    })
}

pub fn derive(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let struct_ = match &input.data {
        syn::Data::Struct(v) => v,
        syn::Data::Enum(_) => {
            return Err(syn::Error::new_spanned(input, "Enums not supported"));
        }
        syn::Data::Union(_) => {
            return Err(syn::Error::new_spanned(input, "Unions not supported"));
        }
    };

    let fields = match &struct_.fields {
        syn::Fields::Named(fields) => &fields.named,
        syn::Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Unnamed fields not supported",
            ));
        }
        syn::Fields::Unit => {
            return Err(syn::Error::new_spanned(
                input,
                "Unsized struct not supported",
            ));
        }
    };

    let diff_impl = gen_impl_diff(&input.ident, &fields);
    let changeset_struct = gen_changeset_struct(&input.ident, &fields)?;

    let output = quote! {
        #[automatically_derived]
        use structdiff::types::*;

        #changeset_struct
        #diff_impl
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_tokens_eq::assert_tokens_eq;

    #[test]
    fn basic() {
        let quoted = quote! {
            pub struct Time {
                pub secs: Result<u64, String>,
                pub subsec_nanos: Option<u32>,
            }
        };
        let input: DeriveInput = syn::parse2(quoted).unwrap();
        let x = derive(input).unwrap();
        println!("{}", quote! { #x });

        assert_tokens_eq!(quote! {}, &x)
    }
}
