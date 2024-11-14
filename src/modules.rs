use quote::ToTokens;
use std::any::Any;
use std::path::Path;
use syn::Item;

// pub enum Unit {
//     Path(PathBuf),
//     Item(String),
// }

// fn find_item(file: &str)
/// lv2, prelude::PluginInstance
fn many(search_file: &Path, item_path: &str) {
    // given start path, try find foo.rs or foo/mod.rs
    assert!(!item_path.ends_with("::") && !item_path.is_empty());

    let (path, last) = match item_path.rsplit_once("::") {
        Some(v) => v,
        None => panic!(),
    };

    let contents = std::fs::read_to_string(search_file).unwrap();
    let file: syn::File = syn::parse_str(&contents).unwrap();
    for item in file.items {}
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