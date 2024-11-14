mod files;
mod modules;

use files::*;
use proc_macro::TokenStream;

use std::path::PathBuf;

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
    // ok, all we gotta do is `module_file_path`
    None
}

/// lv2-0.6.0
fn get_crate_file(the_crate: &str) -> std::io::Result<PathBuf> {
    match the_crate {
        "std" => todo!(),
        "core" => todo!(),
        name => Ok(get_registry_srcs_path()?.join(name)),
    }
}

#[proc_macro]
pub fn get_span(item: TokenStream) -> TokenStream {
    let x = item.clone().into_iter().next().unwrap();
    // dbg!(&x);
    let span = format!("{:?}", x.span());
    format!("\"{span} <{item}>\"").parse().unwrap()
}