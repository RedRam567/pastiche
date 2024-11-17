mod files;
mod rust;
mod standard_library;
mod syn_helpers;

use crate::files::*;
use crate::rust::{ModuleLocation, RustPath};
use crate::syn_helpers::*;
use proc_macro::TokenStream;
use quote::ToTokens;
use std::path::{Path, PathBuf};
use syn::{Item, Visibility};

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

// // fn pastiche_attr_inner(search_dir: &Path, item_path: RustPath, item: Item) -> Item {
// //     // let attrs = item.a
// // }

/// `lv2-0.6.0`, `std@1.82.0`, `core@latest`, `alloc@nightly`
fn get_crate_dir(the_crate: &str) -> std::io::Result<PathBuf> {
    match the_crate {
        "std" => todo!(),
        "core" => todo!(),
        name => Ok(get_registry_srcs_path()?.join(name)),
    }
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
    let (mod_path, item_name) = item_path.split().unwrap();

    // Get the file its in
    let (file_path, mod_location) = module_file_system_path(search_dir, mod_path.as_str());
    if mod_location == ModuleLocation::Inline {
        todo!("inline");
    }

    let file_string = std::fs::read_to_string(file_path).unwrap();
    let file = syn::parse_str::<syn::File>(&file_string).unwrap();

    // Force it public
    let item = find_item_in_file(&file, item_name).unwrap();
    item_force_visibility(item, vis).to_token_stream().into()
}

fn find_item_in_file(file: &syn::File, item_path: RustPath) -> Option<&Item> {
    if !item_path.is_single_item() {
        todo!("inline module")
    }

    let ident = item_path.inner;
    file.items.iter().find(|item| item_ident(item).as_ref() == Some(&ident))
}
