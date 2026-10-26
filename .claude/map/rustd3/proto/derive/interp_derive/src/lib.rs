//! `#[derive(Interpolate)]`: field-wise lerp for structs
//! with named fields; `#[interpolate(skip)]` keeps the
//! start value until t = 1, as d3 does for non-numbers.
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Interpolate, attributes(interpolate))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let (imp, ty, wh) = ast.generics.split_for_impl();
    let Data::Struct(s) = &ast.data else {
        return syn::Error::new_spanned(name, "Interpolate derives only for structs")
            .to_compile_error()
            .into();
    };
    let Fields::Named(f) = &s.fields else {
        return syn::Error::new_spanned(name, "Interpolate needs named fields")
            .to_compile_error()
            .into();
    };
    let fields = f.named.iter().map(|f| {
        let id = &f.ident;
        let skip = f.attrs.iter().any(|a| {
            a.path().is_ident("interpolate")
                && a.parse_args::<syn::Ident>().map(|i| i == "skip").unwrap_or(false)
        });
        if skip {
            quote! { #id: if t < 1.0 { self.#id.clone() } else { to.#id.clone() } }
        } else {
            quote! { #id: Interpolate::lerp(&self.#id, &to.#id, t) }
        }
    });
    quote! {
        impl #imp Interpolate for #name #ty #wh {
            fn lerp(&self, to: &Self, t: f64) -> Self {
                Self { #(#fields,)* }
            }
        }
    }
    .into()
}
