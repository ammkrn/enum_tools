#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unreachable_code)]
extern crate proc_macro;
use proc_macro::TokenStream;

use std::collections::HashSet;
use proc_macro2::Ident as Ident2;
use quote::{ quote, format_ident };
use syn::{ parse_macro_input, 
           parse_quote, 
           parse::Parse,
           parse::ParseStream,
           parse::Result,
           ItemFn, 
           punctuated::Punctuated,
           Stmt };

mod helpers;
mod field_sets;
// The behavior you want out of the box is that
// for fields shared by all enum variants, 
// getters are type (&A) -> &B
// for fields not shared by all enum variants,
// getters are of type (&A) -> Option<&B>


#[proc_macro_derive(Get)]
pub fn derive_get(input : TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as syn::DeriveInput);
    // we don't need any visibility/generic stuff right now
    let enum_ident = derive_input.ident;

    let all_enum_variants : Punctuated<syn::Variant, _> = match derive_input.data {
        syn::Data::Enum(data_enum) => data_enum.variants,
        _ => panic!("Not an enum!")
    };

    let getters = crate::helpers::mk_common_field_get(&enum_ident, &all_enum_variants);

    TokenStream::from(quote! {
        #(#getters)*
    })
}

#[proc_macro_derive(IterUnique)]
pub fn unique_iter(input : TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as syn::DeriveInput);
    let enum_ident = derive_input.ident;
    let all_enum_variants : Punctuated<syn::Variant, _> = match derive_input.data {
        syn::Data::Enum(data_enum) => data_enum.variants,
        _ => panic!("Not an enum!")
    };

    let item = crate::helpers::map_variants_for_unique_iter(&enum_ident, all_enum_variants);

    TokenStream::from(quote! {
        #item
    })

}


#[proc_macro_derive(GetMut)]
pub fn derive_get_mut(input : TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as syn::DeriveInput);
    let enum_ident = derive_input.ident;
    
    let all_enum_variants : Punctuated<syn::Variant, _> = match derive_input.data {
        syn::Data::Enum(data_enum) => data_enum.variants,
        _ => panic!("Not an enum!")
    };
    let getters = crate::helpers::mk_common_field_get_mut(&enum_ident, &all_enum_variants);

    //let enum_generics = derive_input.generics;
    TokenStream::from(quote! {
        #(#getters)*
    })
}

#[proc_macro_derive(Discrim)]
pub fn derive_discrim(input : TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as syn::DeriveInput);
    let enum_ident = derive_input.ident;

    let all_enum_variants : Punctuated<syn::Variant, _> = match derive_input.data {
        syn::Data::Enum(data_enum) => data_enum.variants,
        _ => panic!("Not an enum in `Discrim`!")
    };

    let discrims = crate::helpers::mk_discrims(&enum_ident, &all_enum_variants);
    

    //let enum_generics = derive_input.generics;
    TokenStream::from(quote! {
        #(#discrims)*
    })

}
