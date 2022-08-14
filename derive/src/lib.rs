use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_derive(DynStruct)]
pub fn derive_dyn_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let output = match expand(input) {
        Ok(output) => output,
        Err(e) => e.to_compile_error(),
    };

    output.into()
}

macro_rules! err {
    ($span:expr, $($fmt:tt)+) => {
        syn::Error::new_spanned($span, format_args!($($fmt)+))
    }
}

fn expand(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(struc) => {
            check_repr(&input)?;

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            let (sized_fields, dynamic_field) = collect_fields(struc)?;

            let single = syn::Ident::new(
                &format!("{}_DynStruct_Single", input.ident),
                input.ident.span(),
            );

            let phantom_field;
            let phantom_init;
            if input.generics.lt_token.is_some() {
                let variables = input.generics.params.iter().map(|param| match param {
                    syn::GenericParam::Type(ty) => {
                        let ident = &ty.ident;
                        quote! { #ident }
                    }
                    syn::GenericParam::Lifetime(life) => {
                        let lifetime = &life.lifetime;
                        quote! { &#lifetime () }
                    }
                    syn::GenericParam::Const(constant) => {
                        let ident = &constant.ident;
                        quote! { [(); #ident] }
                    }
                });

                phantom_field = quote! {
                    __DynStruct_phantom: std::marker::PhantomData<(#(#variables,)*)>
                };
                phantom_init = quote! { __DynStruct_phantom: std::marker::PhantomData };
            } else {
                phantom_field = quote! {};
                phantom_init = quote! {};
            };

            let single_definition;
            let single_init;
            let single_idents: Vec<syn::Ident>;
            if matches!(struc.fields, syn::Fields::Named(_)) {
                single_definition = quote! {
                    #[repr(C)]
                    struct #single #impl_generics #where_clause {
                        #(#sized_fields,)*
                        #phantom_field
                    }
                };
                single_idents = sized_fields
                    .iter()
                    .map(|field| field.ident.clone().unwrap())
                    .collect();
                single_init = quote! { #single { #(#single_idents,)* #phantom_init } };
            } else {
                single_definition = quote! {
                    #[repr(C)]
                    struct #single #impl_generics ( #(#sized_fields,)* #phantom_init ) #where_clause;
                };
                single_idents = sized_fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| syn::Ident::new(&format!("_{}", i), span(field)))
                    .collect();
                single_init = quote! { #single ( #(#single_idents,)*, #phantom_init ) };
            };

            let sized_parameters = sized_fields.iter().enumerate().map(|(i, field)| {
                let name = &single_idents[i];
                let ty = &field.ty;
                quote! { #name: #ty }
            });

            let dynamic_type = match &dynamic_field.ty {
                syn::Type::Slice(inner) => inner.elem.as_ref(),
                _ => {
                    return Err(err!(
                        dynamic_field.ty,
                        "the last field needs to be a slice `[T]`"
                    ))
                }
            };
            let dynamic_name = dynamic_field
                .ident
                .clone()
                .unwrap_or_else(|| syn::Ident::new("tail", span(dynamic_type)));

            let struct_ident = &input.ident;
            Ok(quote! {
                impl #impl_generics #struct_ident #type_generics #where_clause {
                    pub fn new<I>(#(#sized_parameters,)* #dynamic_name: I) -> Box<Self>
                        where I: std::iter::IntoIterator<Item = #dynamic_type>,
                              <I as std::iter::IntoIterator>::IntoIter: std::iter::ExactSizeIterator
                    {
                        #single_definition

                        let header: #single #type_generics = #single_init;

                        let dyn_struct = dyn_struct::DynStruct::new(header, #dynamic_name);
                        let ptr = std::boxed::Box::into_raw(dyn_struct);
                        unsafe { std::boxed::Box::from_raw(ptr as *mut Self) }
                    }
                }
            })
        }
        _ => Err(err!(
            &input.ident,
            "`DynStruct` can only be derived for structs"
        )),
    }
}

fn span<T: syn::spanned::Spanned>(value: &T) -> proc_macro2::Span {
    value.span()
}

fn collect_fields(struc: &syn::DataStruct) -> syn::Result<(Vec<syn::Field>, syn::Field)> {
    let mut fields = struc.fields.clone();

    for field in fields.iter_mut() {
        field.attrs.clear();
        field.vis = syn::Visibility::Inherited;
    }

    let dynamic = match &mut fields {
        syn::Fields::Named(fields) => fields.named.pop(),
        syn::Fields::Unnamed(fields) => fields.unnamed.pop(),
        syn::Fields::Unit => None,
    };

    let dynamic =
        dynamic.ok_or_else(|| err!(&struc.fields, "cannot derive `DynStruct` for empty struct"))?;

    Ok((fields.into_iter().collect(), dynamic.into_value()))
}

fn check_repr(input: &syn::DeriveInput) -> syn::Result<()> {
    if input.attrs.iter().any(|attr| is_repr_c(attr)) {
        Ok(())
    } else {
        Err(err!(
            &input.ident,
            "`DynStruct` can only be derived for structs with `#[repr(C)]`"
        ))
    }
}

fn is_repr_c(attr: &syn::Attribute) -> bool {
    match attr.path.get_ident() {
        Some(ident) if ident == "repr" => {}
        _ => return false,
    }

    match find_ident(attr.tokens.clone()) {
        Some(ident) if ident == "C" => true,
        _ => false,
    }
}

fn find_ident(tokens: TokenStream) -> Option<syn::Ident> {
    tokens.into_iter().find_map(|tree| match tree {
        quote::__private::TokenTree::Group(group) => find_ident(group.stream()),
        quote::__private::TokenTree::Ident(ident) => Some(ident.clone()),
        _ => None,
    })
}

