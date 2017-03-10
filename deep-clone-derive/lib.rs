extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(DeepClone)]
pub fn deep_clone(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_deep_clone(&ast);
    gen.parse().unwrap()
}

fn impl_deep_clone(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;

    let borrowed_lifetime_params = ast.generics.lifetimes.iter().map(|alpha| quote! { #alpha });
    let borrowed_type_params = ast.generics.ty_params.iter().map(|ty| quote! { #ty });
    let borrowed_params = borrowed_lifetime_params.chain(borrowed_type_params).collect::<Vec<_>>();
    let borrowed = if borrowed_params.is_empty() {
        quote! { }
    } else {
        quote! { < #(#borrowed_params),* > }
    };
    
    let type_constraints = ast.generics.ty_params.iter().map(|ty| quote! { #ty: DeepClone });
    let where_clause_predicates = ast.generics.where_clause.predicates.iter().map(|pred| quote! { #pred });
    let where_clause_items = type_constraints.chain(where_clause_predicates).collect::<Vec<_>>();
    let where_clause = if where_clause_items.is_empty() {
        quote! { }
    } else {
        quote! { where #(#where_clause_items),* }
    };

    let owned_lifetime_params = ast.generics.lifetimes.iter().map(|_| quote! { 'static });
    let owned_type_params = ast.generics.ty_params.iter().map(|ty| quote! { #ty::DeepCloned });
    let owned_params = owned_lifetime_params.chain(owned_type_params).collect::<Vec<_>>();
    let owned = if owned_params.is_empty() {
        quote! { }
    } else {
        quote! { < #(#owned_params),* > }
    };

    let deep_clone = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref body)) => {
            let fields = body.iter()
                .filter_map(|field| field.ident.as_ref())
                .map(|ident| quote! { #ident: self.#ident.deep_clone() })
                .collect::<Vec<_>>();
            quote! { #name { #(#fields),* } }
        },
        syn::Body::Struct(syn::VariantData::Tuple(ref body)) => {
            let fields = (0..body.len())
                .map(syn::Ident::from)
                .map(|index| quote! { self.#index.deep_clone() })
                .collect::<Vec<_>>();
            quote! { #name ( #(#fields),* ) }
        },
        syn::Body::Struct(syn::VariantData::Unit) => {
            quote! { #name }
        },
        syn::Body::Enum(ref body) => {
            let cases = body.iter()
                .map(|case| {
                    let unqualified_ident = &case.ident;
                    let ident = quote! { #name::#unqualified_ident };
                    match case.data {
                        syn::VariantData::Struct(ref body) => {
                            let idents = body.iter()
                                .filter_map(|field| field.ident.as_ref())
                                .collect::<Vec<_>>();;
                            let cloned = idents.iter()
                                .map(|ident| quote! { #ident: #ident.deep_clone() })
                                .collect::<Vec<_>>();
                            quote! { #ident { #(ref #idents),* } => #ident { #(#cloned),* } }
                        },
                        syn::VariantData::Tuple(ref body) => {
                            let idents = (0..body.len())
                                .map(|index| syn::Ident::from(format!("x{}", index)))
                                .collect::<Vec<_>>();
                            let cloned = idents.iter()
                                .map(|ident| quote! { #ident.deep_clone() })
                                .collect::<Vec<_>>();
                            quote! { #ident ( #(ref #idents),* ) => #ident ( #(#cloned),* ) }
                        },
                        syn::VariantData::Unit => {
                            quote! { #ident => #ident }
                        },
                    }
                })
                .collect::<Vec<_>>();
            quote! { match *self { #(#cases),* } }
        },
    };
    
    quote! {
        impl #borrowed DeepClone for #name #borrowed #where_clause {
            type DeepCloned = #name #owned;
            fn deep_clone(&self) -> #name #owned { #deep_clone }
        }
    }
}
