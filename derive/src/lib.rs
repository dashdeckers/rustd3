//! `#[derive(Interpolate)]`, which interpolates a struct field by field so
//! that marks with several animated attributes transition as one value;
//! enabled through `rustd3-core`'s `derive` feature, which re-exports it as
//! `rustd3_core::interpolate::Interpolate` beside the trait. Examples and
//! the shapes it rejects are documented there.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericArgument, Ident, LitStr, PathArguments,
    ReturnType, Type, WherePredicate, parse_quote,
};

/// `#[derive(Interpolate)]`: each field of a struct with named fields
/// interpolates through its own impl, and a field marked
/// `#[interpolate(skip)]` keeps the start value while `t < 1` and takes the
/// end value from `t = 1` on, as d3 jumps values it cannot blend. On a
/// generic struct the impl is bounded by each field type that names a type
/// parameter (`Interpolate`, or `Clone` when skipped) and by `Self: Clone`.
/// `#[interpolate(crate = "path")]` on the struct names the trait as
/// `path::interpolate::Interpolate`, for code that imports the derive by its
/// full path without the trait.
#[proc_macro_derive(Interpolate, attributes(interpolate))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// The derive's body, on proc-macro2 types so unit tests can call it.
fn expand(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let input: DeriveInput = syn::parse2(input)?;
    let mut errors = Vec::new();
    let krate = container(&input.attrs).unwrap_or_else(|e| {
        errors.push(e);
        None
    });
    let named = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => Some(&f.named),
            Fields::Unnamed(_) | Fields::Unit => None,
        },
        Data::Enum(_) | Data::Union(_) => None,
    };
    let Some(named) = named else {
        errors.push(syn::Error::new_spanned(
            &input.ident,
            "Interpolate derives only for structs with named fields",
        ));
        return Err(combined(errors));
    };
    let fields: Vec<_> = named
        .iter()
        .filter_map(|f| {
            skipped(&f.attrs)
                .map_err(|e| errors.push(e))
                .ok()
                .map(|s| (f, s))
        })
        .collect();
    if !errors.is_empty() {
        return Err(combined(errors));
    }

    let path: syn::Path = match krate {
        Some(k) => parse_quote!(#k::interpolate::Interpolate),
        None => parse_quote!(Interpolate),
    };
    let params: Vec<&Ident> = input.generics.type_params().map(|p| &p.ident).collect();
    let mut generics = input.generics.clone();
    let predicates = &mut generics.make_where_clause().predicates;
    for &(field, skip) in &fields {
        let ty = &field.ty;
        if !mentions(ty, &params) {
            continue;
        }
        let bound: WherePredicate = if skip {
            parse_quote!(#ty: ::core::clone::Clone)
        } else {
            parse_quote!(#ty: #path)
        };
        predicates.push(bound);
    }
    // The supertrait needs `Self: Clone`, which the field bounds imply only
    // when each parameter is a whole field's type: `Mark<T>: Interpolate`
    // does not give `T: Clone` to a derived `Clone`, nor does a skipped
    // `PhantomData<T>`.
    if !params.is_empty() {
        predicates.push(parse_quote!(Self: ::core::clone::Clone));
    }
    if !predicates.empty_or_trailing() {
        predicates.push_punct(Default::default());
    }
    let inits = fields.iter().map(|&(field, skip)| {
        let id = &field.ident;
        if skip {
            quote! {
                #id: if t < 1.0 {
                    ::core::clone::Clone::clone(&self.#id)
                } else {
                    ::core::clone::Clone::clone(&to.#id)
                }
            }
        } else {
            quote! { #id: #path::lerp(&self.#id, &to.#id, t) }
        }
    });
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics #path for #name #ty_generics #where_clause {
            fn lerp(&self, to: &Self, t: f64) -> Self {
                Self { #(#inits,)* }
            }
        }
    })
}

/// The container's `crate = "path"`, if given.
fn container(attrs: &[Attribute]) -> Result<Option<syn::Path>, syn::Error> {
    let mut krate = None;
    let mut errors = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("interpolate")) {
        let parsed = attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("crate") {
                return Err(unknown(&meta.path, "crate"));
            }
            let path = meta.value()?.parse::<LitStr>()?.parse::<syn::Path>()?;
            match krate.replace(path) {
                Some(_) => Err(meta.error("duplicate interpolate attribute `crate`")),
                None => Ok(()),
            }
        });
        errors.extend(parsed.err());
    }
    if errors.is_empty() {
        Ok(krate)
    } else {
        Err(combined(errors))
    }
}

/// Whether a field carries `#[interpolate(skip)]`.
fn skipped(attrs: &[Attribute]) -> Result<bool, syn::Error> {
    let mut skip = false;
    let mut errors = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("interpolate")) {
        let parsed = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
                Ok(())
            } else {
                Err(unknown(&meta.path, "skip"))
            }
        });
        errors.extend(parsed.err());
    }
    if errors.is_empty() {
        Ok(skip)
    } else {
        Err(combined(errors))
    }
}

/// An error spanned on an attribute key the derive does not know.
fn unknown(key: &syn::Path, expected: &str) -> syn::Error {
    let shown = key.to_token_stream().to_string().replace(' ', "");
    syn::Error::new_spanned(
        key,
        format!("unknown interpolate attribute `{shown}`; expected `{expected}`"),
    )
}

/// One error carrying every message of `errors`, which is not empty.
fn combined(errors: Vec<syn::Error>) -> syn::Error {
    errors
        .into_iter()
        .reduce(|mut all, e| {
            all.combine(e);
            all
        })
        .expect("invariant: called with at least one error")
}

/// Whether `ty` names one of the type parameters `params`, walked by hand
/// (syn's default features lack `visit`). Shapes whose parameters hide in
/// bounds or macro input count as naming one: the bound that adds is then
/// trivially true or false, and a false one fails exactly as the field
/// would without it.
fn mentions(ty: &Type, params: &[&Ident]) -> bool {
    if params.is_empty() {
        return false;
    }
    match ty {
        Type::Path(p) => {
            let head = p.path.leading_colon.is_none()
                && p.path
                    .segments
                    .first()
                    .is_some_and(|s| params.contains(&&s.ident));
            head || p.qself.as_ref().is_some_and(|q| mentions(&q.ty, params))
                || p.path
                    .segments
                    .iter()
                    .any(|s| arguments(&s.arguments, params))
        }
        Type::Reference(r) => mentions(&r.elem, params),
        Type::Slice(s) => mentions(&s.elem, params),
        Type::Array(a) => mentions(&a.elem, params),
        Type::Ptr(p) => mentions(&p.elem, params),
        Type::Paren(p) => mentions(&p.elem, params),
        Type::Group(g) => mentions(&g.elem, params),
        Type::Tuple(t) => any(&t.elems, params),
        Type::Never(_) => false,
        Type::BareFn(_)
        | Type::ImplTrait(_)
        | Type::Infer(_)
        | Type::Macro(_)
        | Type::TraitObject(_)
        | Type::Verbatim(_) => true,
        _ => true,
    }
}

/// Whether a path segment's generic arguments name one of `params`.
fn arguments(args: &PathArguments, params: &[&Ident]) -> bool {
    match args {
        PathArguments::None => false,
        PathArguments::AngleBracketed(a) => a.args.iter().any(|arg| match arg {
            GenericArgument::Type(t) => mentions(t, params),
            GenericArgument::AssocType(a) => mentions(&a.ty, params),
            GenericArgument::Lifetime(_)
            | GenericArgument::Const(_)
            | GenericArgument::AssocConst(_) => false,
            GenericArgument::Constraint(_) => true,
            _ => true,
        }),
        PathArguments::Parenthesized(p) => {
            any(&p.inputs, params)
                || match &p.output {
                    ReturnType::Default => false,
                    ReturnType::Type(_, t) => mentions(t, params),
                }
        }
    }
}

/// Whether any of `types` names one of `params`.
fn any<P>(types: &Punctuated<Type, P>, params: &[&Ident]) -> bool {
    types.iter().any(|t| mentions(t, params))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `tokens` as a string without whitespace.
    fn squeezed(tokens: impl ToTokens) -> String {
        tokens
            .to_token_stream()
            .to_string()
            .split_whitespace()
            .collect()
    }

    /// The messages of the error `expand` returns for `input`.
    fn messages(input: TokenStream) -> Vec<String> {
        let error = expand(input).expect_err("the input is rejected");
        error.into_iter().map(|e| e.to_string()).collect()
    }

    #[test]
    fn mark_expands_as_the_specification_writes() {
        let input = quote! {
            struct Mark<T> { v: T, w: f64, #[interpolate(skip)] label: String }
        };
        let expected = quote! {
            impl<T> Interpolate for Mark<T>
            where
                T: Interpolate,
                Self: ::core::clone::Clone,
            {
                fn lerp(&self, to: &Self, t: f64) -> Self {
                    Self {
                        v: Interpolate::lerp(&self.v, &to.v, t),
                        w: Interpolate::lerp(&self.w, &to.w, t),
                        label: if t < 1.0 {
                            ::core::clone::Clone::clone(&self.label)
                        } else {
                            ::core::clone::Clone::clone(&to.label)
                        },
                    }
                }
            }
        };
        assert_eq!(squeezed(expand(input).unwrap()), squeezed(expected));
    }

    #[test]
    fn skipped_generic_fields_are_bounded_by_clone() {
        let input = quote! {
            struct S<'a, T, L, const N: usize> where T: Copy {
                v: [T; N], w: &'a f64, #[interpolate(skip)] label: Vec<L>
            }
        };
        let output = squeezed(expand(input).unwrap());
        assert!(output.contains(
            "whereT:Copy,[T;N]:Interpolate,Vec<L>:::core::clone::Clone,Self:::core::clone::Clone,{"
        ));
    }

    #[test]
    fn the_crate_attribute_names_the_trait_by_its_path() {
        let input = quote! {
            #[interpolate(crate = "::rustd3")]
            struct S<T> { x: T }
        };
        let output = squeezed(expand(input).unwrap());
        assert!(output.starts_with(
            "impl<T>::rustd3::interpolate::InterpolateforS<T>whereT:::rustd3::interpolate::Interpolate,Self:"
        ));
    }

    #[test]
    fn unknown_keys_are_named_and_every_error_is_reported() {
        let input = quote! {
            #[interpolate(krate = "x")]
            struct S { #[interpolate(skp)] a: f64, #[interpolate(skip)] b: String, #[interpolate(crate = "y")] c: f64 }
        };
        assert_eq!(
            messages(input),
            [
                "unknown interpolate attribute `krate`; expected `crate`",
                "unknown interpolate attribute `skp`; expected `skip`",
                "unknown interpolate attribute `crate`; expected `skip`",
            ]
        );
    }

    #[test]
    fn enums_unions_tuple_and_unit_structs_are_rejected() {
        let shapes = [
            quote! { enum E { A } },
            quote! { union U { a: f64 } },
            quote! { struct P(f64, f64); },
            quote! { struct Q; },
        ];
        for input in shapes {
            assert_eq!(
                messages(input),
                ["Interpolate derives only for structs with named fields"]
            );
        }
    }

    #[test]
    fn a_duplicate_crate_attribute_is_an_error() {
        let input = quote! {
            #[interpolate(crate = "a", crate = "b")]
            struct S { x: f64 }
        };
        assert_eq!(messages(input), ["duplicate interpolate attribute `crate`"]);
    }
}
