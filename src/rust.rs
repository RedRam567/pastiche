use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

// TODO: I forgot associeted types and methods exist
/// `::crate::mod::mod::item` or just a single section
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RustPath {
    pub(crate) inner: String,
}

impl RustPath {
    /// Checks if the path is crate, module, or item, with no "directories".
    /// Returns false for absolute paths.
    pub fn is_single_item(&self) -> bool {
        !self.inner.contains("::")
    }

    /// returns the path to the last item and the last item.
    #[expect(unused)]
    pub fn split_last(&self) -> Option<(RustPath, RustPath)> {
        self.inner.rsplit_once("::").map(|(l, r)| (l.into(), r.into()))
    }

    /// Returns the first segment in the path.
    #[expect(unused)]
    pub fn first(&self) -> String {
        let (first, rest) = self.inner.split_once("::").unwrap_or((&self.inner, ""));
        first.to_string()
    }

    /// crate, mod::mod::mod, item
    pub fn parts(&self) -> Option<(RustPath, RustPath, RustPath)> {
        let s = &self.inner;
        let (crate_, s) = s.split_once("::")?;
        let (mods, item) = s.rsplit_once("::")?;
        Some((crate_.into(), mods.into(), item.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<&str> for RustPath {
    fn from(value: &str) -> Self {
        RustPath { inner: value.to_string() }
    }
}

impl FromStr for RustPath {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RustPath { inner: s.to_string() })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ModuleLocation {
    /// `foo/...`
    Folder,
    /// `foo.rs`
    File,
    /// `mod foo {}`
    Inline,
}

#[derive(Clone, Debug)]
pub struct RustToolchain {
    pub channel: String,
    pub version: Option<String>,
    pub date: Option<String>,
    pub triple: Option<String>,
}

impl RustToolchain {
    /// `.../1.68.2-x86_64-unknown-linux-gnu`
    pub(crate) fn from_path(path: &Path) -> Self {
        let s = path.to_str().expect("not utf8 toolchain path");
        // beta, nightly, stable, 1.66.1
        // FIXME: use last part
        let (_path_version, triple) = s.split_once('-').expect("unexpected toolchain path format");

        // I dont see a better way than to just call `rustc --version`
        let rustc = path.join("bin/rustc");
        let output = Command::new(rustc)
            .arg("--version")
            .output()
            .expect("error starting `rustc --version`");
        assert!(output.status.success(), "{:?} error running `rustc --version`", output.status);
        let rustc_version = String::from_utf8(output.stdout).expect("bad version uft8");

        let triple = Some(triple.to_string());
        Self::from_rustc_str(&rustc_version, triple).expect("couldn't parse rustc --version")
    }

    /// Check that `pattern` matches `self`, similar to [`matches!`].
    /// Skips checking any [`None`] fields for `pattern`.
    /// # Panics.
    /// Panics if any fields of `self` are [`None`].
    ///
    /// [match]: https://doc.rust-lang.org/stable/std/keyword.match.html
    #[allow(clippy::unwrap_used, reason = "infallible, nicer formatting")]
    fn matches(&self, pattern: &Self) -> bool {
        let RustToolchain {
            channel,
            version: Some(version),
            date: Some(date),
            triple: Some(triple),
        } = self
        else {
            panic!("self must be some {self:?}")
        };

        // TODO: semantic version check. 1.82.9999 == 1.82
        // skips checking fields if is_none. unwraps are infallible.
        // same as: is_some && unwrap != { return false }
        &pattern.channel == channel
            && (pattern.version.is_none() || pattern.version.as_ref().unwrap() == version)
            && (pattern.date.is_none() || pattern.date.as_ref().unwrap() == date)
            && (pattern.triple.is_none() || pattern.triple.as_ref().unwrap() == triple)
    }

    /// `rustc 1.83.0-nightly (da889684c 2024-09-20)`,
    /// `rustc 1.82.0 (f6e511eec 2024-10-15)`,
    /// `rustc 1.83.0-beta.3 (f41c7ed98 2024-10-31)`
    fn from_rustc_str(s: &str, triple: Option<String>) -> Result<Self, ()> {
        let parts = s.split(' ').collect::<Vec<_>>();
        let &["rustc", version_channel, _hash, date] = parts.as_slice() else { return Err(()) };
        let (version, channel) = match version_channel.split_once('-') {
            Some((v, c)) => (v, Some(c)),
            None => (version_channel, None),
        };
        Ok(Self {
            channel: channel.unwrap_or("stable").to_string(),
            version: Some(version.to_string()),
            date: Some(date.to_string()),
            triple,
        })
    }

    /// - `stable`
    /// - `1.82.0`
    /// - `stable@1.82.0`
    /// - `beta`
    /// - `beta.4@1.82.0`
    /// - `nightly`
    /// - `nightly@1.82.0`
    /// - `nightly@2024-09-20`
    /// - `nightly@1.82.0-2024-09-20`
    pub fn from_pastiche_crate_str(s: &str, triple: Option<String>) -> Result<Self, &'static str> {
        // TODO: use HOST var
        // This works, imma not try to refactor
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

        /// `stable`, `1.82.0`, `invalid`
        fn parse_chan_or_version(s: &str) -> (Option<String>, Option<String>) {
            // Self::try_parse_channel(s).unwrap_or_else(|| Self::try_parse_version(s))
            if let Some(v) = try_parse_channel(s) {
                return (Some(v), None);
            }
            if let Some(v) = try_parse_version(s) {
                return (None, Some(v));
            }
            (None, None)
        }
        /// `1.82.0`, `2024-09-20`, `1.82.0-2024-09-20`, `invalid`
        fn parse_version_andor_date(s: &str) -> (Option<String>, Option<String>) {
            if let Some((version, date)) = s.split_once('-') {
                return (Some(version.to_string()), Some(date.to_string()));
            }
            let version = try_parse_version(s);
            let date = try_parse_date(s);
            (version, date)
        }

        if let Some((channel, version_date)) = s.split_once('@') {
            // chan version date:`nightly@1.82.0-2024-09-20`
            let (version, date) = parse_version_andor_date(version_date);
            if version.is_none() && date.is_none() {
                return Err("Bad version/date. Check after the `@` sign");
            }

            Ok(RustToolchain { channel: channel.to_string(), version, date, triple })
        } else {
            // channel or version: `stable` OR `1.82.0`
            let (channel, version) = parse_chan_or_version(s);
            if channel.is_none() && version.is_none() {
                return Err("Bad channel/version");
            }

            let channel = channel.unwrap_or("stable".to_string());
            Ok(RustToolchain { channel, version, date: None, triple })
        }
    }
}

/// See [`RustToolchain::matches`].
///
/// # Errors
/// Returns zero no toolchains were found, or all matching toolchains.
pub(crate) fn get_specific_toolchain(
    toolchains: Vec<(RustToolchain, PathBuf)>, pattern: &RustToolchain,
) -> Result<(RustToolchain, PathBuf), Vec<(RustToolchain, PathBuf)>> {
    let tcs = toolchains.into_iter().filter(|(tc, _)| tc.matches(pattern)).collect::<Vec<_>>();
    if tcs.len() == 1 {
        Ok(tcs[0].clone())
    } else {
        Err(tcs)
    }
}

/// Get the toolchains in `.../.rustup/toolchains/`
pub(crate) fn all_toolchains() -> Vec<(RustToolchain, PathBuf)> {
    let search_dir = home::rustup_home().expect("failed to find rustup home").join("toolchains");
    let iter = std::fs::read_dir(search_dir).expect("error walking toolchains folder");

    let mut out = Vec::new();
    for entry in iter {
        let path = entry.expect("bad entry").path();
        if !path.is_dir() {
            continue;
        }
        out.push((RustToolchain::from_path(&path), path));
    }

    out
}
