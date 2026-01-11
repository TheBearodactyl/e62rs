use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{
        FnArg, GenericParam, Generics, ImplItem, Item, Meta, ReturnType, Type, TypePath,
        parse_macro_input, punctuated::Punctuated, token::Comma,
    },
};

#[proc_macro_attribute]
/// allow the usage of doc comments on function/method parameters
pub fn argdoc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);

    match input {
        Item::Fn(item_fn) => TokenStream::from(process_function(
            &item_fn.attrs,
            &item_fn.vis,
            &item_fn.sig,
            &item_fn.block,
        )),

        Item::Impl(mut item_impl) => {
            for impl_item in &mut item_impl.items {
                if let ImplItem::Fn(method) = impl_item {
                    let doc_sections = build_doc_sections(
                        &method.attrs,
                        &method.sig.generics,
                        &method.sig.inputs,
                        &method.sig.output,
                    );

                    let new_inputs = clean_param_attrs(&method.sig.inputs);
                    let filtered_attrs = filter_attrs(&method.attrs);
                    let new_generics = clean_generic_attrs(&method.sig.generics);

                    method.sig.inputs = new_inputs;
                    method.sig.generics = new_generics;
                    method.attrs = filtered_attrs;

                    if !doc_sections.is_empty() {
                        let doc_attr: syn::Attribute = syn::parse_quote! {
                            #[doc = #doc_sections]
                        };

                        method.attrs.insert(0, doc_attr);
                    }
                }
            }

            TokenStream::from(quote! { #item_impl })
        }

        Item::Trait(mut item_trait) => {
            for trait_item in &mut item_trait.items {
                if let syn::TraitItem::Fn(method) = trait_item {
                    let doc_sections = build_doc_sections(
                        &method.attrs,
                        &method.sig.generics,
                        &method.sig.inputs,
                        &method.sig.output,
                    );

                    let new_inputs = clean_param_attrs(&method.sig.inputs);
                    let filtered_attrs = filter_attrs(&method.attrs);
                    let new_generics = clean_generic_attrs(&method.sig.generics);

                    method.sig.inputs = new_inputs;
                    method.sig.generics = new_generics;
                    method.attrs = filtered_attrs;

                    if !doc_sections.is_empty() {
                        let doc_attr: syn::Attribute = syn::parse_quote! {
                            #[doc = #doc_sections]
                        };

                        method.attrs.insert(0, doc_attr);
                    }
                }
            }

            TokenStream::from(quote! { #item_trait })
        }

        _ => {
            let error = syn::Error::new_spanned(
                &input,
                "argdoc can only be applied to functions, impl blocks, or traits",
            );

            TokenStream::from(error.to_compile_error())
        }
    }
}

fn process_function(
    attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    sig: &syn::Signature,
    block: &syn::Block,
) -> proc_macro2::TokenStream {
    let doc_sections = build_doc_sections(attrs, &sig.generics, &sig.inputs, &sig.output);
    let new_inputs = clean_param_attrs(&sig.inputs);
    let filtered_attrs = filter_attrs(attrs);

    let mut new_sig = sig.clone();
    new_sig.inputs = new_inputs;

    quote! {
        #(#filtered_attrs)*
        #[doc = #doc_sections]
        #vis #new_sig {
            #block
        }
    }
}

fn build_doc_sections(
    attrs: &[syn::Attribute],
    generics: &Generics,
    inputs: &Punctuated<FnArg, Comma>,
    output: &ReturnType,
) -> String {
    let mut doc_sections = String::new();
    let param_docs = extract_param_docs(inputs);
    let error_docs = extract_error_docs(attrs);
    let generic_docs = extract_generic_docs(generics);
    let panic_docs = extract_panic_docs(attrs);
    let ret_result = matches!(output, ReturnType::Type(_, ty) if is_result_type(ty));

    if !generic_docs.is_empty() {
        doc_sections.push_str(&format!(
            "\n\n# Type Parameters\n\n{}",
            generic_docs.join("\n")
        ));
    }

    if !param_docs.is_empty() {
        doc_sections.push_str(&format!("\n\n# Parameters\n\n{}", param_docs.join("\n")));
    }

    if ret_result && !error_docs.is_empty() {
        doc_sections.push_str(&format!(
            "\n\n# Errors\n\nReturns an error if:\n{}",
            error_docs.join("\n")
        ));
    }

    if !panic_docs.is_empty() {
        doc_sections.push_str(&format!("\n\n# Panics\n\n{}", panic_docs.join("\n")));
    }

    doc_sections
}

fn extract_generic_docs(generics: &Generics) -> Vec<String> {
    let mut generic_docs = Vec::new();

    for param in &generics.params {
        match param {
            GenericParam::Type(type_param) => {
                for attr in &type_param.attrs {
                    if attr.path().is_ident("doc")
                        && let Meta::NameValue(meta) = &attr.meta
                        && let syn::Expr::Lit(expr_lit) = &meta.value
                        && let syn::Lit::Str(lit_str) = &expr_lit.lit
                    {
                        generic_docs.push(format!("* `{}` -{}", type_param.ident, lit_str.value()));
                    }
                }
            }

            GenericParam::Lifetime(lifetime_param) => {
                for attr in &lifetime_param.attrs {
                    if attr.path().is_ident("doc")
                        && let Meta::NameValue(meta) = &attr.meta
                        && let syn::Expr::Lit(expr_lit) = &meta.value
                        && let syn::Lit::Str(lit_str) = &expr_lit.lit
                    {
                        generic_docs.push(format!(
                            "* `{}` -{}",
                            lifetime_param.lifetime,
                            lit_str.value()
                        ));
                    }
                }
            }

            GenericParam::Const(const_param) => {
                for attr in &const_param.attrs {
                    if attr.path().is_ident("doc")
                        && let Meta::NameValue(meta) = &attr.meta
                        && let syn::Expr::Lit(expr_lit) = &meta.value
                        && let syn::Lit::Str(lit_str) = &expr_lit.lit
                    {
                        generic_docs.push(format!(
                            "* `{}` -{}",
                            const_param.ident,
                            lit_str.value()
                        ));
                    }
                }
            }
        }
    }

    generic_docs
}

fn extract_param_docs(inputs: &Punctuated<FnArg, Comma>) -> Vec<String> {
    let mut param_docs = Vec::new();

    for arg in inputs {
        if let FnArg::Typed(pat_type) = arg {
            for attr in &pat_type.attrs {
                if attr.path().is_ident("doc")
                    && let Meta::NameValue(meta) = &attr.meta
                    && let syn::Expr::Lit(expr_lit) = &meta.value
                    && let syn::Lit::Str(lit_str) = &expr_lit.lit
                {
                    let param_name = extract_param_name(&pat_type.pat);
                    param_docs.push(format!("* `{}` - {}", param_name, lit_str.value().trim()));
                }
            }
        }
    }

    param_docs
}

fn extract_error_docs(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut err_docs = Vec::new();

    for attr in attrs {
        if (attr.path().is_ident("error") || attr.path().is_ident("err"))
            && let Meta::NameValue(meta) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &meta.value
            && let syn::Lit::Str(lit_str) = &expr_lit.lit
        {
            err_docs.push(format!("* {}", lit_str.value()));
        }
    }

    err_docs
}

fn extract_panic_docs(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut panic_docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("panics")
            && let Meta::NameValue(meta) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &meta.value
            && let syn::Lit::Str(lit_str) = &expr_lit.lit
        {
            panic_docs.push(format!("* {}", lit_str.value()));
        }
    }

    panic_docs
}

fn clean_param_attrs(inputs: &Punctuated<FnArg, Comma>) -> Punctuated<FnArg, Comma> {
    inputs
        .iter()
        .map(|arg| {
            if let FnArg::Typed(mut pat_type) = arg.clone() {
                pat_type.attrs.clear();
                FnArg::Typed(pat_type)
            } else {
                arg.clone()
            }
        })
        .collect()
}

fn clean_generic_attrs(generics: &Generics) -> Generics {
    let mut new_generics = generics.clone();

    for param in &mut new_generics.params {
        match param {
            GenericParam::Type(type_param) => {
                type_param.attrs.clear();
            }
            GenericParam::Lifetime(lifetime_param) => {
                lifetime_param.attrs.clear();
            }
            GenericParam::Const(const_param) => {
                const_param.attrs.clear();
            }
        }
    }

    new_generics
}

fn filter_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("error") && !attr.path().is_ident("panic"))
        .cloned()
        .collect()
}

fn extract_param_name(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
        syn::Pat::Wild(_) => "_".to_string(),
        _ => "param".to_string(),
    }
}

fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
    {
        return segment.ident == "Result";
    }

    false
}
