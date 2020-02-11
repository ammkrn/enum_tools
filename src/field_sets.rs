use std::collections::HashSet;
use quote::format_ident;
use syn::{ Ident, 
           Variant, 
           parse_quote, 
           punctuated::Punctuated, 
           token::Comma, 
           Field };


pub fn fields_inter(variants : &Punctuated<Variant, Comma>) -> HashSet<Field> {
    let mut named_fields = 
        variants
        .iter()
        .filter_map(|v| match &v.fields {
            syn::Fields::Named(fields_named) => Some(fields_named.named.clone().into_iter().collect::<HashSet<Field>>()),
            _ => None
        }).collect::<Vec<HashSet<Field>>>();

    let mut acc_set = named_fields.pop().unwrap_or_else(|| HashSet::new());

    for s in named_fields.iter() {
        let intersection = acc_set.intersection(s).cloned().collect::<HashSet<Field>>();
        acc_set = intersection
    }

    acc_set
}

pub fn fields_union(variants : &Punctuated<Variant, Comma>) -> HashSet<Field> {
    let mut union = HashSet::new();
    for elem in variants
               .iter()
               .filter_map(|v| match &v.fields {
                   syn::Fields::Named(fields_named) => Some(fields_named.named.clone()),
                   _ => None
               }).flat_map(|named| named.into_iter()) {
                   union.insert(elem);
               }
    union
}


pub fn fields_diff(variants : &Punctuated<Variant, Comma>) -> HashSet<Field> {
    let union = fields_union(variants);
    let inter = fields_inter(variants);

    union.difference(&inter).cloned().collect::<HashSet<Field>>()
}