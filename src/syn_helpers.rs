use crate::rust::RustPath;
use quote::ToTokens;
use syn::{
    AttrStyle, Attribute, Ident, Item, LitStr, Meta, MetaList, MetaNameValue, PathArguments,
    Visibility,
};

pub fn find_item_in_file(file: &syn::File, item_path: RustPath) -> Option<&Item> {
    if !item_path.is_single_item() {
        todo!("inline module")
    }

    let ident = item_path.inner;
    file.items
        .iter()
        .find(|item| item_ident(item).map(|v| v.to_string()).as_ref() == Some(&ident))
}

// TODO: sub visibility
pub fn item_set_visibility(item: &Item, vis: Visibility) -> Item {
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
        // Item::ForeignMod(_) => todo!(), // extern "C" {}
        // Item::Impl(_) => todo!(),
        // Item::Verbatim(_) => todo!(),
        _ => panic!("unsupported item type: {:?}", item.to_token_stream()),
    }
    item
}

pub fn item_set_ident(item: &Item, new_ident: Ident) -> Item {
    use syn::*;

    let mut item = item.clone();
    match &mut item {
        Item::Const(ItemConst { ident, .. })
        | Item::Enum(ItemEnum { ident, .. })
        | Item::ExternCrate(ItemExternCrate { ident, .. })
        | Item::Fn(ItemFn { sig: Signature { ident, .. }, .. })
        | Item::Macro(ItemMacro { ident: Some(ident), .. }) // macro_rules! ident {}
        | Item::Mod(ItemMod { ident, .. })
        | Item::Static(ItemStatic { ident, .. })
        | Item::Struct(ItemStruct { ident, .. })
        | Item::Trait(ItemTrait { ident, .. })
        | Item::TraitAlias(ItemTraitAlias { ident, .. })
        | Item::Type(ItemType { ident, .. })
        | Item::Union(ItemUnion { ident, .. }) => *ident = new_ident,
        // Item::Macro(ItemMacro { mac: Macro {path, ..}, .. }) => None, // path!()
        // Item::ForeignMod(_) => todo!(), // extern "C" {}
        // Item::Impl(_) => todo!(),
        // Item::Use(_) => todo!(), // TODO: hard
        // Item::Verbatim(_) => todo!(),
        _ => (),
    }
    item
}

pub fn item_ident(item: &Item) -> Option<Ident> {
    use syn::*;

    // I wish or patterns and bindings were in the Rust book.
    match item {
        Item::Const(ItemConst { ident, .. })
        | Item::Enum(ItemEnum { ident, .. })
        | Item::ExternCrate(ItemExternCrate { ident, .. })
        | Item::Fn(ItemFn { sig: Signature { ident, .. }, .. })
        | Item::Macro(ItemMacro { ident: Some(ident), .. }) // macro_rules! ident {}
        | Item::Mod(ItemMod { ident, .. })
        | Item::Static(ItemStatic { ident, .. })
        | Item::Struct(ItemStruct { ident, .. })
        | Item::Trait(ItemTrait { ident, .. })
        | Item::TraitAlias(ItemTraitAlias { ident, .. })
        | Item::Type(ItemType { ident, .. })
        | Item::Union(ItemUnion { ident, .. }) => Some(ident.clone()),
        // Item::Macro(ItemMacro { mac: Macro {path, ..}, .. }) => None, // path!()
        // Item::ForeignMod(_) => todo!(), // extern "C" {}
        // Item::Impl(_) => todo!(),
        // Item::Use(_) => todo!(), // TODO: hard
        // Item::Verbatim(_) => todo!(),
        _ => None,
    }
}

// returns None for macro_rules!
pub fn item_visibility(item: &Item) -> Option<Visibility> {
    use syn::*;

    match item {
        Item::Const(ItemConst { vis, .. })
        | Item::Enum(ItemEnum { vis, .. })
        | Item::ExternCrate(ItemExternCrate { vis, .. })
        | Item::Fn(ItemFn { vis, .. })
        | Item::Mod(ItemMod { vis, .. })
        | Item::Static(ItemStatic { vis, .. })
        | Item::Struct(ItemStruct { vis, .. })
        | Item::Trait(ItemTrait { vis, .. })
        | Item::TraitAlias(ItemTraitAlias { vis, .. })
        | Item::Type(ItemType { vis, .. })
        | Item::Union(ItemUnion { vis, .. })
        | Item::Use(ItemUse { vis, .. }) => Some(vis.clone()),
        // | Item::Macro(_)
        // Item::ForeignMod(_) => todo!(), // extern "C" {}
        // Item::Impl(_) => todo!(),
        // Item::Verbatim(_) => todo!(),
        _ => None,
    }
}

/// Only returns `None` for [`Item::Verbatim`]
pub fn item_attributes(item: &Item) -> Option<&Vec<Attribute>> {
    use syn::*;

    match item {
        Item::Const(ItemConst { attrs, .. })
        | Item::Enum(ItemEnum { attrs, .. })
        | Item::ExternCrate(ItemExternCrate { attrs, .. })
        | Item::Fn(ItemFn { attrs, .. })
        | Item::Mod(ItemMod { attrs, .. })
        | Item::Static(ItemStatic { attrs, .. })
        | Item::Struct(ItemStruct { attrs, .. })
        | Item::Trait(ItemTrait { attrs, .. })
        | Item::TraitAlias(ItemTraitAlias { attrs, .. })
        | Item::Type(ItemType { attrs, .. })
        | Item::Union(ItemUnion { attrs, .. })
        | Item::Use(ItemUse { attrs, .. })
        | Item::Macro(ItemMacro { attrs, .. })
        | Item::ForeignMod(ItemForeignMod { attrs, .. })
        | Item::Impl(ItemImpl { attrs, .. }) => Some(attrs),
        _ => None,
        // Item::Verbatim(_) => todo!(),
    }
}

/// Only returns `None` for [`Item::Verbatim`]
pub fn item_attributes_mut(item: &mut Item) -> Option<&mut Vec<Attribute>> {
    use syn::*;

    match item {
        Item::Const(ItemConst { attrs, .. })
        | Item::Enum(ItemEnum { attrs, .. })
        | Item::ExternCrate(ItemExternCrate { attrs, .. })
        | Item::Fn(ItemFn { attrs, .. })
        | Item::Mod(ItemMod { attrs, .. })
        | Item::Static(ItemStatic { attrs, .. })
        | Item::Struct(ItemStruct { attrs, .. })
        | Item::Trait(ItemTrait { attrs, .. })
        | Item::TraitAlias(ItemTraitAlias { attrs, .. })
        | Item::Type(ItemType { attrs, .. })
        | Item::Union(ItemUnion { attrs, .. })
        | Item::Use(ItemUse { attrs, .. })
        | Item::Macro(ItemMacro { attrs, .. })
        | Item::ForeignMod(ItemForeignMod { attrs, .. })
        | Item::Impl(ItemImpl { attrs, .. }) => Some(attrs),
        _ => None,
        // Item::Verbatim(_) => todo!(),
    }
}

/// #[derive(Clone, Copy, "blah")]
#[expect(unused)]
pub fn attr_meta_list(attr: Attribute) -> Option<MetaList> {
    match attr {
        Attribute { style: AttrStyle::Inner(..), meta: Meta::List(v), .. } => Some(v),
        _ => None,
    }
}

/// #[attr = "blah"]
/// #[attr = 2 + 2]
pub fn attr_meta_name_value(attr: Attribute) -> Option<MetaNameValue> {
    match attr {
        Attribute { style: AttrStyle::Outer, meta: Meta::NameValue(v), .. } => Some(v),
        _ => None,
    }
}

pub fn syn_path_to_string(path: syn::Path) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    if path.leading_colon.is_some() {
        _ = write!(out, "::");
    }
    for segment in path.segments {
        match segment.arguments {
            PathArguments::None => _ = write!(out, "{}::", segment.ident),
            PathArguments::AngleBracketed(_) => todo!(),
            PathArguments::Parenthesized(_) => todo!(),
        }
    }
    out.trim_end_matches("::").to_string()
}

pub fn tokens_to_string_literal(tokens: proc_macro::TokenStream) -> syn::Result<String> {
    let lit = syn::parse::<LitStr>(tokens)?;
    let raw_string = lit.to_token_stream().to_string();
    Ok(raw_string
        .strip_prefix('"')
        .unwrap_or_else(|| unreachable!())
        .strip_suffix('"')
        .unwrap_or_else(|| unreachable!())
        .to_string())
}

#[expect(clippy::match_like_matches_macro)]
pub fn item_remove_stablility_attrs(item: &mut Item) {
    let Some(attrs) = item_attributes_mut(item) else { return };
    attrs.retain(|attr| match attr.path().to_token_stream().to_string().as_str() {
        "stable" | "unstable" => false,
        _ => true,
    });
}
