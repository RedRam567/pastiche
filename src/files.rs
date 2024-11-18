use crate::rust::{all_toolchains2, get_specific_toolchain2, ModuleLocation, Toolchain2};
use crate::standard_library::{all_toolchains, RustChannel, RustVersion};
use crate::vec_into_single;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

/// `.cargo/registry/src/index.crates.io-HASH/`
pub fn get_registry_srcs_path() -> io::Result<PathBuf> {
    let src = home::cargo_home()?.join("registry/src");
    let entries = fs::read_dir(src)?.collect::<Vec<_>>();

    let [Ok(index_crates_io)] = entries.as_slice() else {
        todo!("multiple indexes found: {:?}", entries);
    };

    Ok(index_crates_io.path())
}

/// Find file system path a module is located at.
/// `crate-name-0.6.0`, `[::]mod::mod::mod[::]`
///
/// `path` must end with a module, not an item.
pub fn module_file_system_path(search_dir: &Path, mod_path: &str) -> (PathBuf, ModuleLocation) {
    #[cfg(debug_assertions)]
    {
        let last = match mod_path.rsplit_once("::") {
            Some((_, last)) => last,
            None => mod_path,
        };
        debug_assert!(
            !last.as_bytes()[0].is_ascii_uppercase(),
            "Last element in path must be a module, not an item"
        );
    }

    let path = mod_path.trim_matches(':').replace("::", "/");
    let base_dir = search_dir.join("src");

    // FIXME: doesn't handle multiple nested inline modules at the end
    // Most modules must have folders. Last can be foo.rs or mod foo {}
    // So we can skip everything but last module.
    let mod_folder = base_dir.join(format!("{}/mod.rs", path));
    let mod_file = base_dir.join(format!("{}.rs", path));
    let mut mod_inline = base_dir.join(path);
    mod_inline.pop();

    if mod_folder.exists() {
        (mod_folder, ModuleLocation::Folder)
    } else if mod_file.exists() {
        (mod_file, ModuleLocation::File)
    } else {
        // just assume its inline
        (mod_inline, ModuleLocation::Inline)
    }
}

// kinda outdated now
/// `lv2-0.6.0`, `std@1.82.0`, `core@latest`, `alloc@nightly`
pub fn get_crate_dir(the_crate: &str) -> std::io::Result<PathBuf> {
    match the_crate {
        "std" => todo!(),
        "core" => todo!(),
        name => Ok(get_registry_srcs_path()?.join(name)),
    }
}

#[derive(Clone, Debug)]
pub enum Crate {
    Crate { crate_name: String, version: Option<String> },
    StdLibCrate { crate_name: String, version: Option<String> },
}

impl Crate {
    pub fn file_system_path(&self, triple: Option<String>) -> std::io::Result<PathBuf> {
        match self {
            Crate::Crate { crate_name, version } => {
                let fs_name = format!("{}-{}", crate_name, version.as_ref().unwrap());
                Ok(get_registry_srcs_path()?.join(fs_name))
            }
            Crate::StdLibCrate { crate_name, version } => {
                let std_lib_crate =
                    StdLibCrate::from_str(version.as_ref().expect("must specify rustc toolchain"))
                        .expect("error parsing rustc toolchain");
                let toolchain = Toolchain2::from_std_lib_crate(std_lib_crate, triple);
                let (tc, raw_path) = get_specific_toolchain2(all_toolchains2(), &toolchain)
                    .expect("found multiple or zero matching toolchains");
                let path = raw_path.join("lib/rustlib/src/rust/library").join(crate_name);
                Ok(path)
            }
        }
    }
}

impl FromStr for Crate {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, version) = match s.split_once('@') {
            Some((name, version)) => (name.to_string(), Some(version.to_string())),
            None => (s.to_string(), None),
        };
        let crate_ = match name.as_str() {
            "std" | "core" => Crate::StdLibCrate { crate_name: name, version },
            _ => Crate::Crate { crate_name: name, version },
        };
        dbg!(&crate_);
        Ok(crate_)
    }
}

pub struct StdLibCrate {
    pub channel: String,
    pub version: Option<String>,
    pub date: Option<String>,
}

impl StdLibCrate {
    fn try_parse_channel(s: &str) -> Option<String> {
        match s {
            "stable" | "beta" | "nightly" => Some(s.to_string()),
            _ => None,
        }
    }

    /// `1.2.3`
    fn try_parse_version(s: &str) -> Option<String> {
        if !s.contains('.') {
            return None;
        }
        if s.split('.').all(|num| num.parse::<i64>().is_ok()) {
            Some(s.to_string())
        } else {
            None
        }
    }

    /// `1-2-3`
    fn try_parse_date(s: &str) -> Option<String> {
        if !s.contains('-') {
            return None;
        }
        if s.split('-').all(|num| num.parse::<i64>().is_ok()) {
            Some(s.to_string())
        } else {
            None
        }
    }

    // TODO: remove RustVersion in favor of this
    fn into_rust_version(self, stable_rust_version: String) -> RustVersion {
        // rustc 1.83.0-nightly (da889684c 2024-09-20),
        // rustc 1.83.0-beta.3 (f41c7ed98 2024-10-31)
        // rustc 1.82.0 (f6e511eec 2024-10-15),
        // rustc 1.65.0 (897e37553 2022-11-02),
        let StdLibCrate { channel, version, date } = self;
        // let
        // format!("rustc {} (SYNTHETIC {})")
        todo!()
    }
}

impl FromStr for StdLibCrate {
    type Err = &'static str;

    /// - stable, beta, nightly
    /// - 1.82.0
    /// - 1.83.0-nightly
    /// - 1.83.0-beta
    /// - nightly@2024-09-20
    ///
    /// - 1.82.0-nightly@2024-09-20
    ///
    /// - stable
    /// - 1.82.0
    /// - stable@1.82.0
    /// - beta
    /// - beta.4@1.82.0
    /// - nightly
    /// - nightly@1.82.0
    /// - nightly@2024-09-20
    /// - nightly@1.82.0-2024-09-20
    ///
    /// -> `rustc 1.83.0-nightly (da889684c 2024-09-20)`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /// `stable`, `1.82.0`, `invalid`
        fn parse_chan_version(s: &str) -> (Option<String>, Option<String>) {
            // Self::try_parse_channel(s).unwrap_or_else(|| Self::try_parse_version(s))
            if let Some(v) = StdLibCrate::try_parse_channel(s) {
                return (Some(v), None);
            }
            if let Some(v) = StdLibCrate::try_parse_version(s) {
                return (None, Some(v));
            }
            (None, None)
        }
        /// `1.82.0`, `2024-09-20`, `1.82.0-2024-09-20`, `invalid`
        fn parse_version_date(s: &str) -> (Option<String>, Option<String>) {
            if let Some((version, date)) = s.split_once('-') {
                return (Some(version.to_string()), Some(date.to_string()));
            }
            let version = StdLibCrate::try_parse_version(s);
            let date = StdLibCrate::try_parse_date(s);
            (version, date)
        }
        if let Some((channel, version_date)) = s.split_once('@') {
            // `nightly@1.82.0-2024-09-20`
            let channel = channel.to_string();
            let (version, date) = parse_version_date(version_date);
            if version.is_none() && date.is_none() {
                return Err("Bad version/date. Check after the `@` sign");
            }

            Ok(StdLibCrate { channel, version, date })
        } else {
            // `stable` or `1.82.0`
            let (channel, version) = parse_chan_version(s);
            if channel.is_none() && version.is_none() {
                return Err("Bad channel/version");
            }

            let channel = channel.unwrap_or("stable".to_string());
            Ok(StdLibCrate { channel, version, date: None })
        }
    }
}
