use lv2::prelude::Plugin;
use lv2::prelude::PortCollection;
use lv2::urid::UriBound;

pastiche::pastiche!("lv2-core-3.0.0", pub, plugin::PluginInstance);

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
