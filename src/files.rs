use crate::rust::{
    all_toolchains, get_specific_toolchain, ModuleLocation, RustPath, RustToolchain,
};
use std::path::{Path, PathBuf};
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
pub fn module_file_system_path(search_dir: &Path, mod_path: RustPath) -> (PathBuf, ModuleLocation) {
    let mod_path = mod_path.as_str();
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

#[derive(Clone, Debug)]
pub enum Crate {
    Crate { crate_name: String, version: Option<String> },
    StdLibCrate { crate_name: String, toolchain: RustToolchain },
}

impl Crate {
    pub fn crate_name(&self) -> &str {
        let (Self::Crate { crate_name, .. } | Self::StdLibCrate { crate_name, .. }) = self;
        crate_name
    }

    pub fn file_system_path(&self) -> std::io::Result<PathBuf> {
        match self {
            Crate::Crate { crate_name, version } => {
                dbg!("bruh", self);
                let fs_name =
                    format!("{}-{}", crate_name, version.as_ref().expect("TODO: version"));
                Ok(get_registry_srcs_path()?.join(fs_name))
            }
            Crate::StdLibCrate { crate_name, toolchain } => {
                dbg!("ay", self);
                let (_tc, raw_path) = get_specific_toolchain(all_toolchains(), toolchain)
                    .expect("found multiple or zero matching toolchains");
                let path = raw_path.join("lib/rustlib/src/rust/library").join(crate_name);
                Ok(path)
            }
        }
    }

    pub fn from_pastiche_crate_str(
        s: &str, triple: Option<String>, item_path: &RustPath,
    ) -> Result<Self, &'static str> {
        let (crate_name, version_or_toolchain) = match s.split_once('@') {
            Some((name, version)) => (name.to_string(), Some(version.to_string())),
            None => (s.to_string(), None),
        };
        let crate_ = match crate_name.as_str() {
            // TODO: beta.9
            "stable" | "beta" | "nightly" => Crate::StdLibCrate {
                crate_name: item_path.first(),
                toolchain: RustToolchain::from_pastiche_crate_str(s, triple)?,
            },
            _ => Crate::Crate { crate_name, version: version_or_toolchain },
        };
        Ok(crate_)
    }
}
