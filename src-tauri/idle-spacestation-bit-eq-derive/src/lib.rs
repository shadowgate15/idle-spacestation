use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(BitEq)]
pub fn derive_bit_eq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let field_comparisons = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let comparisons = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    quote! { self.#field_name.bit_eq(&other.#field_name) }
                });

                quote! { #(#comparisons)&&* }
            }
            _ => panic!("BitEq only supports named struct fields"),
        },
        _ => panic!("BitEq only supports structs"),
    };

    let expanded = quote! {
        impl #impl_generics crate::game::bit_eq::BitEq for #name #ty_generics #where_clause {
            fn bit_eq(&self, other: &Self) -> bool {
                #field_comparisons
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(BitHash, attributes(bit_hash))]
pub fn derive_bit_hash(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let field_hashes = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let mut hashes = fields
                    .named
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        let field_name = &field.ident;
                        let options = BitHashFieldOptions::from_field(index, field);
                        let hash = if options.use_std_hash {
                            quote! { std::hash::Hash::hash(&self.#field_name, state); }
                        } else if options.sort_before_hash {
                            quote! {
                                let mut sorted = self.#field_name.clone();
                                sorted.sort();
                                std::hash::Hash::hash(&sorted, state);
                            }
                        } else {
                            quote! { self.#field_name.bit_hash(state); }
                        };

                        (options.order, hash)
                    })
                    .collect::<Vec<_>>();

                hashes.sort_by_key(|(order, _)| *order);
                let hashes = hashes.into_iter().map(|(_, hash)| hash);

                quote! { #(#hashes)* }
            }
            _ => panic!("BitHash only supports named struct fields"),
        },
        _ => panic!("BitHash only supports structs"),
    };

    let expanded = quote! {
        impl #impl_generics crate::game::bit_eq::BitHash for #name #ty_generics #where_clause {
            fn bit_hash<H: std::hash::Hasher>(&self, state: &mut H) {
                #field_hashes
            }
        }
    };

    expanded.into()
}

struct BitHashFieldOptions {
    order: usize,
    sort_before_hash: bool,
    use_std_hash: bool,
}

impl BitHashFieldOptions {
    fn from_field(default_order: usize, field: &syn::Field) -> Self {
        let mut options = Self {
            order: default_order,
            sort_before_hash: false,
            use_std_hash: false,
        };

        for attr in &field.attrs {
            if !attr.path().is_ident("bit_hash") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("order") {
                    let value = meta.value()?;
                    let order: syn::LitInt = value.parse()?;
                    options.order = order.base10_parse()?;
                    Ok(())
                } else if meta.path.is_ident("sort") {
                    options.sort_before_hash = true;
                    Ok(())
                } else if meta.path.is_ident("std") {
                    options.use_std_hash = true;
                    Ok(())
                } else {
                    Err(meta.error("unsupported bit_hash option"))
                }
            })
            .expect("invalid bit_hash attribute");
        }

        options
    }
}
