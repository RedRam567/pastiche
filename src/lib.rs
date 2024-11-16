mod files;

use files::*;
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

// find
/// std, f32::next_up
/// lv2, prelude::PluginInstance
fn get_definition(in_crate: &str, path: &str) -> Option<String> {
    // find lv2 file
    // find prelude module file
    // find PluginInstance
    // its in lv2_cor
    // ...
    // let x = <std>::f32::next_up;
    // let x: <&[(); 0]> = todo!();
    // std::slice::
    None
}

/// lv2-0.6.0
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
fn force_visibility_inner(search_dir: &Path, item_path: RustPath, vis: Visibility) -> TokenStream {
    let (mod_path, item_name) = item_path.split().unwrap();

    // Get the file its in
    let (file_path, mod_location) = module_file_path(search_dir, mod_path.as_str());
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

// TODO: I forgot associeted types and methods exist
/// `::crate::mod::mod::item` or just a single section
#[derive(Clone, Debug, PartialEq, Eq)]
struct RustPath {
    inner: String,
}

impl RustPath {
    /// Checks if the path is crate, module, or item, with no "directories".
    /// Returns false for absolute paths.
    fn is_single_item(&self) -> bool {
        !self.inner.contains("::")
    }

    /// returns the path to the last item and the last item.
    fn split(&self) -> Option<(RustPath, RustPath)> {
        self.inner.rsplit_once("::").map(|(l, r)| (l.into(), r.into()))
    }

    fn last_item(&self) -> RustPath {
        let Some((_, last)) = self.inner.rsplit_once("::") else { return self.clone() };
        RustPath { inner: last.to_string() }
    }

    fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<&str> for RustPath {
    fn from(value: &str) -> Self {
        RustPath { inner: value.to_string() }
    }
}

fn item_ident(item: &Item) -> Option<String> {
    use syn::*;

    // I wish or patterns and bindings were in the Rust book.
    match item {
        Item::Const(ItemConst { ident, .. })
        | Item::Enum(ItemEnum { ident, .. })
        | Item::ExternCrate(ItemExternCrate { ident, .. })
        | Item::Fn(ItemFn { sig: Signature { ident, .. }, .. })
        | Item::Macro(ItemMacro { ident: Some(ident), .. })
        | Item::Mod(ItemMod { ident, .. })
        | Item::Static(ItemStatic { ident, .. })
        | Item::Struct(ItemStruct { ident, .. })
        | Item::Trait(ItemTrait { ident, .. })
        | Item::TraitAlias(ItemTraitAlias { ident, .. })
        | Item::Type(ItemType { ident, .. })
        | Item::Union(ItemUnion { ident, .. }) => Some(ident.to_string()),
        // Item::ForeignMod(_) => todo!(), // extern "C" {}
        // Item::Impl(_) => todo!(),
        // Item::Use(_) => todo!(), // TODO: hard
        // Item::Verbatim(_) => todo!(),
        _ => None,
    }
}

// TODO: sub visibility
fn item_force_visibility(item: &Item, vis: Visibility) -> Item {
    use syn::*;

    let mut item = item.clone();
    match &mut item {
        Item::Const(item) => item.vis = vis,
        Item::Enum(item) => item.vis = vis,
        // `pub extern crate` is even weirder than `extern crate`
        Item::ExternCrate(item) => item.vis = vis,
        Item::Fn(item) => item.vis = vis,
        Item::Mod(item) => item.vis = vis,
        Item::Static(item) => item.vis = vis,
        Item::Struct(item) => item.vis = vis,
        Item::Trait(item) => item.vis = vis,
        Item::TraitAlias(item) => item.vis = vis,
        Item::Type(item) => item.vis = vis,
        Item::Union(item) => item.vis = vis,
        Item::Use(item) => item.vis = vis,

        // No visibility
        // Item::Macro(item) => item.vis = vis,
        // Item::ForeignMod(_) => todo!(),
        // Item::Impl(_) => todo!(),
        // Item::Verbatim(_) => todo!(),
        _ => panic!("unsupported item type: {:?}", item.to_token_stream()),
    }
    item
}
