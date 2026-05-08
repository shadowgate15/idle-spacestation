//! Procedural macro crate providing `#[derive(BitEq)]` and `#[derive(BitHash)]`
//! for the `idle_spacestation_lib` crate.
//!
//! The generated `BitEq` implementation performs field-by-field equality using
//! `crate::game::bit_eq::BitEq`, which compares floating-point values by their
//! raw bit patterns (`f32::to_bits()` / `f64::to_bits()`) and uses standard
//! equality for integral and other hash-like snapshot types. This is the
//! proc-macro support behind `idle_spacestation_lib::game::snapshot::state_equals`:
//! snapshot floats must not be compared with raw `==` because NaN bit patterns
//! and deterministic diffing need bit-precise semantics.
//!
//! See also `idle_spacestation_lib::game::bit_eq::BitEq`, the trait derived for
//! the main crate's snapshot DTOs.

#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive `crate::game::bit_eq::BitEq` for a named-field struct.
///
/// The macro expands to a `bit_eq` implementation that compares every field in
/// declaration order by calling that field's own `BitEq::bit_eq` implementation.
/// In the main crate, this means `f32` and `f64` fields use bit-equality via
/// `to_bits()`, while integers, booleans, strings, options, and vectors compose
/// from the standard implementations in `crate::game::bit_eq`. This is the
/// preferred solution for the project rule that snapshot floats must not be
/// compared with raw `==`.
///
/// # Supported input
///
/// - Named-field structs only.
/// - Every field type must implement `crate::game::bit_eq::BitEq`.
/// - Struct generics and `where` clauses are preserved in the generated impl.
///
/// Tuple structs, unit structs, and enums are rejected at macro expansion time.
/// The derive is intentionally scoped to the DTO shapes used by
/// `idle_spacestation_lib::game::snapshot`.
///
/// # Related attributes
///
/// `#[bit_hash(order = N)]` and `#[bit_hash(sort)]` are consumed by the sibling
/// `#[derive(BitHash)]` macro, not by `BitEq`. Use `order` to make hashing use a
/// stable custom field order, and `sort` to clone, sort, and hash a collection
/// when its logical equality should be order-independent for hash purposes.
///
/// # Examples
///
/// ```
/// pub mod game {
///     pub mod bit_eq {
///         pub trait BitEq {
///             fn bit_eq(&self, other: &Self) -> bool;
///         }
///
///         impl BitEq for f32 {
///             fn bit_eq(&self, other: &Self) -> bool {
///                 self.to_bits() == other.to_bits()
///             }
///         }
///
///         impl BitEq for u32 {
///             fn bit_eq(&self, other: &Self) -> bool {
///                 self == other
///             }
///         }
///
///         impl BitEq for bool {
///             fn bit_eq(&self, other: &Self) -> bool {
///                 self == other
///             }
///         }
///     }
/// }
///
/// use game::bit_eq::BitEq as _;
/// use idle_spacestation_bit_eq_derive::BitEq;
///
/// fn main() {
///     #[derive(BitEq)]
///     struct SnapshotTotals {
///         tick_count: u32,
///         power_available: f32,
///         online: bool,
///     }
///
///     let left = SnapshotTotals {
///         tick_count: 7,
///         power_available: f32::NAN,
///         online: true,
///     };
///     let right = SnapshotTotals {
///         tick_count: 7,
///         power_available: f32::NAN,
///         online: true,
///     };
///
///     assert!(left.bit_eq(&right));
/// }
/// ```
///
/// # Examples
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

/// Derive `crate::game::bit_eq::BitHash` for a named-field struct.
///
/// `BitHash` mirrors `BitEq` semantics for deterministic hashing: floating
/// values hash their bit patterns through their field-level `BitHash` impls,
/// ordinary hash-like fields can opt into `std::hash::Hash`, and annotated
/// collections can be sorted before hashing.
///
/// # Supported input
///
/// - Named-field structs only.
/// - Every field must either implement `crate::game::bit_eq::BitHash`, opt into
///   `#[bit_hash(std)]`, or opt into `#[bit_hash(sort)]` for sortable cloned
///   collections.
/// - Struct generics and `where` clauses are preserved.
///
/// # Field attributes
///
/// - `#[bit_hash(order = N)]` assigns a stable hash order independent of source
///   field order.
/// - `#[bit_hash(sort)]` clones the field, sorts it, and hashes the sorted copy.
/// - `#[bit_hash(std)]` uses `std::hash::Hash::hash` directly for that field.
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

/// Parsed options from a field-level `#[bit_hash(...)]` attribute.
struct BitHashFieldOptions {
    /// Stable ordering key used when emitted hash statements are sorted.
    order: usize,
    /// Whether to clone and sort the field before hashing it.
    sort_before_hash: bool,
    /// Whether to delegate directly to `std::hash::Hash` for the field.
    use_std_hash: bool,
}

impl BitHashFieldOptions {
    /// Reads `#[bit_hash(order = N)]`, `#[bit_hash(sort)]`, and
    /// `#[bit_hash(std)]` from a `syn::Field`, falling back to declaration order.
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
