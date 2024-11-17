use lv2::prelude::Plugin;
use lv2::prelude::PortCollection;
use lv2::urid::UriBound;

pastiche::pastiche!("lv2-core-3.0.0", pub, plugin::PluginInstance);
// pastiche::pastiche!("std@1.82", pub, plugin::PluginInstance);

struct DummyPlugin;

impl Plugin for DummyPlugin {
    type Ports = ();
    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(_: &lv2::prelude::PluginInfo, _: &mut Self::InitFeatures) -> Option<Self> {
        Some(DummyPlugin)
    }

    fn run(&mut self, _: &mut Self::Ports, _: &mut Self::AudioFeatures, _: u32) {
    }
}

// Safety: is cstr
unsafe impl UriBound for DummyPlugin {
    const URI: &'static [u8] = c"dummy".to_bytes();
}

fn main() {
    // yay I can use it now
    let _instance = self::PluginInstance {
        instance: DummyPlugin,
        connections: (),
        init_features: (),
        audio_features: (),
    };
}

// #[pastiche::pastiche_attr]
// #[pastiche::uh]
// struct MyPuginInstance {
//     pub INHERIT: (),
// }

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