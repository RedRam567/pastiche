mod files;
mod rust;
mod standard_library;
mod syn_helpers;

use crate::files::*;
use crate::rust::{ModuleLocation, RustPath};
use crate::syn_helpers::*;
use proc_macro::TokenStream;
use quote::ToTokens;
use std::path::Path;
use std::str::FromStr;
use syn::{Attribute, Item, Meta, MetaList, MetaNameValue, Visibility};

// force public (also fields)
// into raw parts

// ideally it'd just be `crate::prelude::Struct as pub(crate) MyStruct`, idk how for sub vis
/// Copy and force an item in a crate to a certain visibility.
///
/// `pastiche!("search_dir", pub, path::item)`.
#[proc_macro]
pub fn pastiche(tokens: TokenStream) -> TokenStream {
    let s = tokens.to_string();
    let (search_dir, s) = s.split_once(',').unwrap();
    let (vis, s) = s.split_once(',').unwrap();
    let None = s.split_once(',') else { panic!("bruh") };
    let item_path = s;

    let search_dir = search_dir.trim().trim_matches('\"').trim();
    let vis = vis.trim();
    let item_path = item_path.trim();

    // let search_dir: &Path = Path::new(search_dir);
    // let search_dir =
    let search_dir = get_crate_dir(search_dir).unwrap();

    dbg!(&search_dir, vis, item_path);

    let vis = syn::parse_str::<Visibility>(vis).unwrap();
    force_visibility_inner(&search_dir, item_path.into(), vis)
    // let path = tokens.
    // let path = syn::parse::<syn::Path>(tokens).expect("Expected item path");
}

// NOTE: search dir, crate version, vis, sub vis, item, attributes removal, adds and overrides
// define must struct
/// Ideally something like this where we can change vis of anything,
/// add, remove, enforce, enforce not, and dont emit any: attributes, consts, methods
/// :
/// ```rust compile_fail
/// pastiche! {
///     is some_crate@0.3.0::prelude::Struct as MyStruct
///     must #[repr(C)] // must be repr(c)
///     add #[derive(Clone)] // add
///     remove #[derive(Debug)] // remove
///     not #[repr(rust)] // cant have repr(rust)
///     // must be a struct, make it public
///     pub struct MyStruct {
///         // must have a field called `len`, make it public
///         pub len,
///         pub capcity: usize,
///         // make rest of the fields public too
///         pub ..
///     }
///
///     impl MyStruct {
///         const ANSWER: i32 = 42;
///         // has this assoc constant of any visibility, including none (`pub(self)`)
///         pub(..) const VIS: () = ();
///         // make rest assoc constants public
///         pub const ..;
///     }
///
///     impl MyStruct {
///         fn morb();
///         pub fn tweak(..) -> ..;
///         // must have 3 args and return any arity tuple
///         pub(crate) fn frobnicate(_, _, _) -> (..);
///         pub fn speak() {
///             println!("meow");
///         }
///     }
/// }
/// ```
///
/// NOTE: attribute like macro better? yeah
/// ```rust compile_fail
/// pastiche! {
///     "std::num::IntErrorKind@stable,1.82",
///     #[attr_has(repr(C))]
///     #[attr_add(derive(Clone))]
///     #[attr_remove(derive(Debug))]
///     #[attr_inherit]
///     #[attr_add(derive(Copy))]
///     #[attr_move(repr(C))] // move reprc from inherited to down here
///     pub struct MyStruct {
///         pub inherit
///     }
/// }
/// ```
fn force_visibility_inner(search_dir: &Path, item_path: RustPath, vis: Visibility) -> TokenStream {
    let (mod_path, item_name) = item_path.split_last().unwrap();

    // Get the file its in
    let (file_path, mod_location) = module_file_system_path(search_dir, mod_path.as_str());
    if mod_location == ModuleLocation::Inline {
        todo!("inline");
    }

    let file_string = std::fs::read_to_string(file_path).unwrap();
    let file = syn::parse_str::<syn::File>(&file_string).unwrap();

    // Force it public
    let item = find_item_in_file(&file, item_name).unwrap();
    item_set_visibility(item, vis).to_token_stream().into()
}

// /// ```rust compile_fail,ignore
// ///     #[pastiche::path(std::num::IntErrorKind, "stable")]
// ///     #[pastiche::attr_has(repr(C))]
// ///     #[pastiche::attr_add(derive(Clone))]
// ///     #[pastiche::attr_remove(derive(Debug))]
// ///     #[pastiche::attr_inherit]
// ///     #[pastiche::attr_add(derive(Copy))]
// ///     #[pastiche::attr_move(repr(C))] // move reprc from inherited to down here
// ///     #[pastiche::use_if_public] // just `use::` if vis matches
// ///     pub struct PubIntErrorKind {
// ///         pub ..
// ///     }
// /// ```
// #[proc_macro_attribute]
// pub fn pastiche_attr(attrs: TokenStream, item: TokenStream) -> TokenStream {
//     dbg!(&attrs);
//     dbg!(&item);
//     let x = syn::parse_macro_input!(item as Item);
//     dbg!(x.to_token_stream());
//     // panic!();
//     item
// }

// #[proc_macro(arg)]
// item ...
#[proc_macro_attribute]
pub fn pastiche_attr(_arg: TokenStream, item: TokenStream) -> TokenStream {
    // let arg = syn::parse_macro_input!(arg as Meta);
    let item = syn::parse_macro_input!(item as Item);

    let mut crate_ = None;
    let mut item_path = None;
    for attr in item_attributes(&item).unwrap_or(&Vec::new()) {
        // dbg!(attr.meta.to_token_stream().to_string());
        match attr.meta {
            Meta::Path(_) => eprintln!("path {}", attr.meta.to_token_stream()),
            Meta::List(_) => eprintln!("list {}", attr.meta.to_token_stream()),
            Meta::NameValue(_) => eprintln!("namevalue {}", attr.meta.to_token_stream()),
        }
        // // match attr.meta
        // dbg!(attr.style);
        let Some(MetaNameValue { path, value, .. }) = attr_meta_name_value(attr.clone()) else {
            continue;
        };
        let tokens = value.to_token_stream().into();
        match syn_path_to_string(path).as_str() {
            "pastiche_crate" => crate_ = Some(tokens_to_string_literal(tokens).expect("crate")),
            "pastiche_path" => item_path = Some(tokens_to_string_literal(tokens).expect("path")),
            _ => (),
        }
    }

    let crate_ = Crate::from_str(crate_.as_ref().expect("expected pastiche_crate"))
        .expect("error parsing crate");
    let item_path = RustPath::from_str(item_path.as_ref().expect("expected pastiche_path"))
        .expect("error parsing path");

    dbg!(&crate_);
    dbg!(&item_path);

    let token_stream = pastiche_inner(crate_, item_path, item, true).to_token_stream();
    token_stream.into()
}

fn pastiche_inner(
    crate_: Crate, item_path: RustPath, item: Item, remove_stablility_attrs: bool,
) -> Item {
    let triple = "x86_64-unknown-linux-gnu".to_string().into(); // FIXME: dont hardcode
    let crate_path = Crate::file_system_path(&crate_, triple).expect("couldn't find crate path");
    let vis = item_visibility(&item).expect("input item must have a visiblity");
    let ident = item_ident(&item).expect("input item must have an ident");
    drop(item);

    let (crate_name, mod_path, item_name) = item_path.parts().expect("path parts");
    // TODO: remove as_str
    let (file_path, mod_location) = module_file_system_path(&crate_path, mod_path.as_str());
    if mod_location == ModuleLocation::Inline {
        todo!("inline module or path does not exist: {file_path:?}")
    }

    // Find the item in the file
    let file_string = std::fs::read_to_string(&file_path).expect("error reading file");
    let file = syn::parse_str::<syn::File>(&file_string).expect("error parsing file");
    let item = find_item_in_file(&file, item_name)
        .unwrap_or_else(|| panic!("item not found in file {:?}", file_path));

    // Process the found item
    let mut item = item_set_ident(&item_set_visibility(item, vis), ident);
    item_remove_stablility_attrs(&mut item);
    item
}

#[track_caller]
pub(crate) fn vec_into_single<T: Clone + std::fmt::Debug>(vec: Vec<T>) -> Result<T, Vec<T>> {
    if vec.len() == 1 {
        Ok(vec[0].clone())
    } else {
        // panic!("Expected a single value. {:?} for {}", vec, std::any::type_name::<T>())
        Err(vec)
    }
}
