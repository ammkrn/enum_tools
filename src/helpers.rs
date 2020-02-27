use crate::field_sets::{ fields_union, fields_inter, fields_diff };
use std::collections::HashSet;
use syn::{ Ident, Variant, parse_quote, punctuated::Punctuated, token::Comma, Field };
use quote::format_ident;

pub fn fold1<A>(mut iter : impl Iterator<Item = A>, f : impl FnMut(A, A) -> A) -> Option<A> {
    iter.next().map(move |x| iter.fold(x, f))
}



pub fn mk_common_field_get(enum_ident : &Ident, variants : &Punctuated<Variant, Comma>) -> Vec<syn::ItemImpl> {
    let s = fields_inter(variants);
    let union = fields_union(variants);
    let inter = fields_inter(variants);
    let diff = fields_diff(variants);

    let mut getters : Vec<syn::ItemImpl> = Vec::new();
    let variant_idents = variants.iter().map(|v| v.ident.clone()).collect::<Vec<Ident>>();




    for f in inter.iter() {
        let getter = field_to_getter(enum_ident, &variant_idents, f);
        getters.push(getter)
    }


    getters
}



pub fn mk_common_field_get_mut(enum_ident : &Ident, variants : &Punctuated<Variant, Comma>) -> Vec<syn::ItemImpl> {
    let s = fields_inter(variants);
    let union = fields_union(variants);
    let inter = fields_inter(variants);
    let diff = fields_diff(variants);

    let mut getters : Vec<syn::ItemImpl> = Vec::new();
    let variant_idents = variants.iter().map(|v| v.ident.clone()).collect::<Vec<Ident>>();

    for f in inter.iter() {
        let mut_getter = field_to_mut_getter(enum_ident, &variant_idents, f);
        getters.push(mut_getter);
    }

    getters
}


 

pub fn mk_match_arm_one(enum_ident : &Ident, variant_ident : &Ident, field_name : &Ident) -> syn::Arm {
    let enum_match_path : syn::Path = parse_quote!(#enum_ident::#variant_ident);
    let arm : syn::Arm = parse_quote! {
        #enum_match_path { #field_name, .. } => #field_name
    };
    arm
}
pub fn field_to_getter(enum_ident : &Ident, variant_idents : &Vec<Ident>, field : &Field) -> syn::ItemImpl {
    let field_name = field.ident.as_ref().expect("No field ident ; field_to_getter").clone();
    let getter_name = format_ident!("get_{}", &field_name);
    let return_type = &field.ty;
    let match_arms = variant_idents.iter().map(|v_id| mk_match_arm_one(enum_ident, v_id, &field_name))
                    .collect::<Punctuated<syn::Arm, syn::token::Comma>>();

    let impl_fn : syn::ItemImpl = parse_quote! {
        impl #enum_ident {
            pub fn #getter_name(&self) -> &#return_type {
                match self {
                    #match_arms
                }
            }
        }
    };
    impl_fn
}

pub fn field_to_mut_getter(enum_ident : &Ident, variant_idents : &Vec<Ident>, field : &Field) -> syn::ItemImpl {
    let field_name = field.ident.as_ref().expect("No field ident ; field_to_getter").clone();
    let getter_name = format_ident!("get_mut_{}", &field_name);
    let return_type = &field.ty;
    let match_arms = variant_idents.iter().map(|v_id| mk_match_arm_one(enum_ident, v_id, &field_name))
                    .collect::<Punctuated<syn::Arm, syn::token::Comma>>();

    let impl_fn : syn::ItemImpl = parse_quote! {
        impl #enum_ident {
            pub fn #getter_name(&mut self) -> &mut #return_type {
                match self {
                    #match_arms
                }
            }
        }
    };
    impl_fn
}

pub struct FieldsSummary {
    common_fields : HashSet<Field>,
    // other stuff is going to be specific to the enum variant
    // since you need to tell the implementation which ones have non-common
    // field X and which don't.
}



// From derive_input, get list of variants
pub fn map_variants_for_unique_iter(enum_ident : &Ident, vs : Punctuated<Variant, syn::token::Comma>) -> syn::ItemImpl {
    let fields_inter = fields_inter(&vs);
    let all_uniques = vs.iter().map(|v| {
        this_variant_unique(v, &fields_inter)
    }).collect::<Vec<Vec<Field>>>();


    let mut all_arms = Punctuated::<syn::Arm, syn::token::Comma>::new();
    for (v, uniques) in vs.iter().zip(all_uniques.iter()) {

        all_arms.push(variant_to_arm(enum_ident, v, uniques));
    }

    let impl_ : syn::ItemImpl = parse_quote! {
        impl #enum_ident {
            pub fn iter_uniques(&self) -> Vec<ItemIdx> {
                match self {
                   #all_arms
                }
            }
        }
    };
    impl_

}

pub fn variant_to_arm(enum_ident : &Ident, v : &Variant, uniques : &Vec<Field>) -> syn::Arm {
    let variant_ident = &v.ident;
    let match_arm_path : syn::Path = parse_quote!(#enum_ident::#variant_ident);

    // If a variant has no unique fields, needs to be handled specially
    // or the macro will panic.
    if uniques.is_empty() {
        let arm : syn::Arm = parse_quote! {
            #match_arm_path { .. } => { Vec::new() }
        };
        return arm
    }

    let field_idents : Punctuated<syn::Ident, syn::token::Comma> = uniques.iter().map(|u| {
        let field_id = u.ident.clone().unwrap();
        field_id
    }).collect();

    let push_stmts : Punctuated<syn::Stmt, syn::token::Semi> = uniques.into_iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let stmt_ : syn::Stmt = parse_quote! {
            buf.push(#field_ident.clone());
        };
        stmt_
    }).collect();
    let arm : syn::Arm = parse_quote! {
        #match_arm_path { #field_idents, .. } => {
            let mut buf = Vec::new();
            #push_stmts
            buf
        }
    };
    arm
}

// Get the list of unique fields for a particular Variant
pub fn this_variant_unique(variant : &Variant, inter : &HashSet<Field>) -> Vec<Field> {
    let mut buf = Vec::new();
    match &variant.fields {
        syn::Fields::Named(fields_named) => {
            for field in fields_named.named.iter() {
                if !(inter.contains(field)) {
                    buf.push(field.clone())
                }
            }
        },
        _ => panic!("this_variant_unique not named fields")
        //syn::Fields::UnNamed(fields_unnamed)
    }

    buf
}


pub fn mk_discrims(enum_ident : &Ident, variants : &syn::punctuated::Punctuated<Variant, syn::token::Comma>) -> Vec<syn::ItemImpl> {
    let mut acc = Vec::with_capacity(variants.len());
    for variant in variants.into_iter() {
        match variant.fields {
            syn::Fields::Named(..) => {
                acc.push(mk_discrim_one_named(enum_ident, &variant.ident))
            },
            syn::Fields::Unnamed(..) => {
                acc.push(mk_discrim_one_unnamed(enum_ident, &variant.ident))

            },
            syn::Fields::Unit => {
                acc.push(mk_discrim_one_unit(enum_ident, &variant.ident))
            }
        }
    }

    acc
}

pub fn mk_discrim_one_named(enum_ident : &Ident, variant_ident : &Ident) -> syn::ItemImpl {
    let match_path : syn::Path = parse_quote!(#enum_ident::#variant_ident);
    let method_name = format_ident!("is_{}", snake_case_name(variant_ident));
    let _x : syn::ItemImpl = parse_quote! {
        
        impl #enum_ident {
            pub fn #method_name(&self) -> bool {
                match self {
                    #match_path { .. } => true,
                    _ => false
                }
            }
        }
    };
    _x
}
pub fn mk_discrim_one_unnamed(enum_ident : &Ident, variant_ident : &Ident) -> syn::ItemImpl {
    let match_path : syn::Path = parse_quote!(#enum_ident::#variant_ident);
    let method_name = format_ident!("is_{}", snake_case_name(variant_ident));
    let _x : syn::ItemImpl = parse_quote! {
        impl #enum_ident {
            pub fn #method_name(&self) -> bool {
                match self {
                    #match_path(..) => true,
                    _ => false
                }
            }
        }
    };
    _x
}
pub fn mk_discrim_one_unit(enum_ident : &Ident, variant_ident : &Ident) -> syn::ItemImpl {
    let match_path : syn::Path = parse_quote!(#enum_ident::#variant_ident);
    let method_name = format_ident!("is_{}", snake_case_name(variant_ident));
    let _m : syn::ImplItem = parse_quote! {
        pub fn #method_name(&self) -> bool {
            match self {
                #match_path => true,
                _ => false
            }
        }
    };
    let _x : syn::ItemImpl = parse_quote! {
        impl #enum_ident {
            #_m
        }
    };
    _x
}


pub fn snake_case_name(ident : &syn::Ident) -> syn::Ident {
    let mut pred_upper_case : bool = false;
    let mut acc = String::new();
    let ident_string = ident.to_string();

    for c in ident_string.chars() {
        let pred_lowercase = acc.chars().last().map(|pred| pred.is_lowercase()).unwrap_or(false);
        let next_uppercase = !(c.is_lowercase());
        if pred_lowercase && next_uppercase {
            acc.push('_')
        }
        acc.push(c.to_ascii_lowercase());

    }

    syn::Ident::new(acc.as_str(), ident.span())
}
