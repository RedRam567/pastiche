mod files;
mod rust;
mod syn_helpers;

use crate::files::*;
use crate::rust::{ModuleLocation, RustPath};
use crate::syn_helpers::*;
use proc_macro::TokenStream;
use quote::ToTokens;
use std::str::FromStr;
use syn::{Item, MetaNameValue};

#[proc_macro_attribute]
pub fn pastiche_attr(_arg: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as Item);

    let mut crate_ = None;
    let mut item_path = None;
    for attr in item_attributes(&item).unwrap_or(&Vec::new()) {
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
    assert_eq!(
        crate_.crate_name().replace('-', "_"),
        crate_name.as_str(),
        "crate name in path must currently be the same as the crate"
    );
    // TODO: remove as_str
    let (file_path, mod_location) = module_file_system_path(&crate_path, mod_path);
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
    if remove_stablility_attrs {
        item_remove_stablility_attrs(&mut item);
    }
    item
}
