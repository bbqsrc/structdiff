extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Diff)]
pub fn structdiff_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = structdiff_macro::derive(input);

    match output {
        Ok(tokens) => tokens.into(),
        Err(err) => {
            TokenStream::from(syn::Error::new(err.span(), err.to_string()).to_compile_error())
        }
    }
}
