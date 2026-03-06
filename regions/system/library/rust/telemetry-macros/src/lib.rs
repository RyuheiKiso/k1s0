use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, spanned::Spanned, Token};

/// `#[k1s0_trace]` proc-macro attribute for automatic tracing instrumentation.
///
/// Wraps functions with `tracing::instrument` to automatically create spans.
///
/// # Options
/// - `skip(arg1, arg2)`: Skip specified arguments from the span.
/// - `name = "custom.span.name"`: Set a custom span name.
///
/// # Examples
/// ```ignore
/// #[k1s0_trace]
/// async fn get_user(id: UserId) -> Result<User, AppError> { ... }
///
/// #[k1s0_trace(skip(password), name = "auth.login")]
/// async fn login(user: &str, password: &str) -> Result<Token, AuthError> { ... }
/// ```
#[proc_macro_attribute]
pub fn k1s0_trace(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as syn::ItemFn);
    let args = parse_macro_input!(args with Punctuated::<syn::Meta, Token![,]>::parse_terminated);

    let mut skip_args: Vec<syn::Ident> = Vec::new();
    let mut custom_name: Option<syn::LitStr> = None;

    for meta in &args {
        match meta {
            syn::Meta::List(list) if list.path.is_ident("skip") => {
                let result: syn::Result<Punctuated<syn::Ident, Token![,]>> =
                    list.parse_args_with(Punctuated::parse_terminated);
                match result {
                    Ok(idents) => skip_args.extend(idents),
                    Err(e) => return e.to_compile_error().into(),
                }
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("name") => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = &nv.value
                {
                    custom_name = Some(lit.clone());
                } else {
                    return syn::Error::new(nv.value.span(), "expected string literal for `name`")
                        .to_compile_error()
                        .into();
                }
            }
            other => {
                return syn::Error::new(other.span(), "unknown attribute, expected `skip` or `name`")
                    .to_compile_error()
                    .into();
            }
        }
    }

    let name_attr = if let Some(ref name) = custom_name {
        quote! { name = #name, }
    } else {
        quote! {}
    };

    let skip_attr = if !skip_args.is_empty() {
        quote! { skip(#(#skip_args),*), }
    } else {
        quote! {}
    };

    let output = quote! {
        #[::tracing::instrument(#name_attr #skip_attr level = "info")]
        #input_fn
    };

    output.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui_placeholder() {
        // proc-macro crate tests are limited; real validation happens
        // at compile time when the macro is used.
        // See telemetry crate tests for integration tests.
    }
}
