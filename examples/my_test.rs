#![allow(clippy::missing_safety_doc)]
#![allow(clippy::undocumented_unsafe_blocks)]

use pastiche::pastiche_attr;
use lv2::prelude::Plugin;
use lv2::prelude::PortCollection;
use lv2::urid::UriBound;
use std::num::IntErrorKind;
use std::num::ParseIntError;
use std::str::FromStr;

// pastiche::pastiche!("lv2-core-3.0.0", pub, plugin::PluginInstance);
// pastiche::pastiche!("std@1.82", pub, plugin::PluginInstance);

struct DummyPlugin;

impl Plugin for DummyPlugin {
    type Ports = ();
    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(_: &lv2::prelude::PluginInfo, _: &mut Self::InitFeatures) -> Option<Self> {
        Some(DummyPlugin)
    }

    fn run(&mut self, _: &mut Self::Ports, _: &mut Self::AudioFeatures, _: u32) {}
}

unsafe impl UriBound for DummyPlugin {
    const URI: &'static [u8] = c"dummy".to_bytes();
}

fn main() {
    // yay I can use it now
    // let _instance = self::PluginInstance {
    //     instance: DummyPlugin,
    //     connections: (),
    //     init_features: (),
    //     audio_features: (),
    // };

    let _instance2 = MyPluginInstance {
        instance: DummyPlugin,
        connections: (),
        init_features: (),
        audio_features: (),
    };

    // Directly construct a ParseIntError
    let my_parse_int_error = MyParseIntError { kind: IntErrorKind::InvalidDigit };
    let std_parse_int_error: ParseIntError =
        unsafe { std::mem::transmute(my_parse_int_error.clone()) };

    assert_eq!(
        format!("{my_parse_int_error:?}").strip_prefix("My").unwrap(),
        format!("{std_parse_int_error:?}")
    );

    // Rather than failing a parse to acheive it :/
    let parse_int_error_invalid: ParseIntError = i32::from_str("baddbeef").unwrap_err();
    assert_eq!(std_parse_int_error, parse_int_error_invalid);
}

#[pastiche_attr]
// #[pastiche_crate("lv2-core@3.0.0")]
// #[pastiche_path("lv2_core::plugin::PluginInstance")]
#[pastiche_crate = "lv2-core@3.0.0"]
#[pastiche_path = "lv2_core::plugin::PluginInstance"]
pub struct MyPluginInstance {
    pub INHERIT: (),
}

mod pub_super_hack {
    use super::*;

    #[pastiche_attr]
    #[pastiche_crate = "stable@1.82.0"]
    #[pastiche_path = "core::num::error::ParseIntError"]
    pub struct MyParseIntError {
        // body is ignored for now
    }
}
pub use pub_super_hack::*;

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

// #[derive(Copy)]
// fn idk() {}