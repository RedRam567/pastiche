use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn get_registry_srcs_path() -> io::Result<PathBuf> {
    let src = home::cargo_home()?.join("registry/src");
    let mut entries = fs::read_dir(src)?;

    // "index.crates.io-<HASH>"
    let (Some(Ok(index_crates_io)), None) = (entries.next(), entries.next()) else {
        // must only have 1 dir else idk what to do
        let into = io::ErrorKind::Other.into();
        return Err(into);
    };

    Ok(index_crates_io.path())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum WalkStatus {
    Ok,
    /// Just doesn't exist or the module is inline.
    NotInFileSystem,
    /// Found `foo.rs` and `foo/mod.rs`
    Duplicate,
}

// most mods must have folders. last can be foo.rs or mod foo {}
// lv2-0.6.0, [::]crate::mod::mod::mod::item
pub(crate) fn walk(search_dir: &Path, item_path: &str) -> (PathBuf, WalkStatus) {
    let item_path = item_path.trim_matches(':');
    let mut working_path = search_dir.to_path_buf();
    let mut iter = item_path.split("::").peekable();

    while let Some(crate_mod) = iter.next() {
        let is_last = iter.peek().is_none();
    }
    todo!()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleLocation {
    /// `foo/...`
    Folder,
    /// `foo.rs`
    File,
    /// `mod foo {}`
    Inline,
}

/// `crate-name-0.6.0`, `[::]mod::mod::mod[::]`
///
/// `path` must end with a module, not an item.
pub fn module_file_path(search_dir: &Path, mod_path: &str) -> (PathBuf, ModuleLocation) {
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

    dbg!(&mod_folder, &mod_file);

    if mod_folder.exists() {
        (mod_folder, ModuleLocation::Folder)
    } else if mod_file.exists() {
        (mod_file, ModuleLocation::File)
    } else {
        // just assume its inline
        (mod_inline, ModuleLocation::Inline)
    }
}
