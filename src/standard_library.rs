use semver::Version;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::process::Command;

/// See [`Toolchain::matches`].
///
/// # Errors
/// Returns zero no toolchains were found, or all matching toolchains.
pub(crate) fn get_specific_toolchain(
    version: &str, triple: &str,
) -> Result<Toolchain, Vec<Toolchain>> {
    let tcs = all_toolchains()
        .into_iter()
        .filter(|tc| tc.matches(version, triple))
        .collect::<Vec<_>>();
    if tcs.len() == 1 {
        Ok(tcs[0].clone())
    } else {
        Err(tcs)
    }
}

/// Get the toolchains in `.../.rustup/toolchains/`
pub(crate) fn all_toolchains() -> Vec<Toolchain> {
    let search_dir = home::rustup_home().expect("failed to find rustup home").join("toolchains");
    let iter = std::fs::read_dir(search_dir).expect("error walking toolchains folder");

    let mut out = Vec::new();
    for entry in iter {
        let path = entry.expect("bad entry").path();
        if !path.is_dir() {
            continue;
        }
        out.push(Toolchain::from_path(&path));
    }

    out
}

// TODO: remove path
/// `1.68.2-x86_64-unknown-linux-gnu`,
/// `nightly-x86_64-unknown-linux-gnu`
#[derive(Clone, Debug)]
pub(crate) struct Toolchain {
    /// `rustc 1.83.0-nightly (da889684c 2024-09-20)`,
    /// `rustc 1.82.0 (f6e511eec 2024-10-15)`,
    /// `rustc 1.83.0-beta.3 (f41c7ed98 2024-10-31)`
    pub(crate) version: RustVersion,
    /// `x86_64-unknown-linux-gnu`
    pub(crate) triple: String,
    /// `.../1.68.2-x86_64-unknown-linux-gnu`
    pub(crate) path: PathBuf,
}

impl Toolchain {
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
        let version = String::from_utf8(output.stdout).expect("bad version uft8");

        Self {
            version: RustVersion { inner: version },
            triple: triple.to_owned(),
            path: path.to_owned(),
        }
    }

    /// match channel, date if exists, triple
    /// `nightly[-2024-02-05]`, `unknown-linux-gnu`
    ///
    /// See also [`RustVersion`]
    pub(crate) fn matches(&self, version: &str, triple: &str) -> bool {
        let (version, date) = version.split_once('-').unwrap_or((version, "SYNTH-SYNTH-SYNTH"));

        let synthetic = RustVersion { inner: format!("rustc {} (SYNTHETIC {})", version, date) };
        let (ver_ord, chan_ord, date_ord) = self.version.detailed_cmp(&synthetic);

        // weird returns
        if ver_ord.is_ne() {
            return false;
        }

        if chan_ord.is_ne() {
            return false;
        }

        // skip checking date if synth
        if date != "SYNTH" && date_ord.is_ne() {
            return false;
        }

        if self.triple != triple {
            return false;
        }

        true
    }
}

/// - `rustc 1.83.0-nightly (da889684c 2024-09-20)`,
/// - `rustc 1.83.0-beta.3 (f41c7ed98 2024-10-31)`
/// - `rustc 1.82.0 (f6e511eec 2024-10-15)`,
/// - `rustc 1.65.0 (897e37553 2022-11-02)`,
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RustVersion {
    pub(crate) inner: String,
}

impl RustVersion {
    pub(crate) fn channel(&self) -> RustChannel {
        RustChannel::from_str(&self.inner)
    }

    /// version, channel, date
    pub(crate) fn detailed_cmp(&self, other: &Self) -> (Ordering, Ordering, Ordering) {
        /// version, channel
        pub(crate) fn cmp_semver_rust(this: &Version, other: &Version) -> (Ordering, Ordering) {
            let this_chan = RustChannel::from_str(this.pre.as_str());
            let other_chan = RustChannel::from_str(other.pre.as_str());

            if this_chan == RustChannel::Beta && other_chan == RustChannel::Beta {
                todo!("TODO: proper handling of (beta.9).cmp(beta.14) stuff")
            }

            let chan = this_chan.cmp(&other_chan);

            match this.major.cmp(&other.major) {
                Ordering::Equal => {}
                ord => return (ord, chan),
            }
            match this.minor.cmp(&other.minor) {
                Ordering::Equal => {}
                ord => return (ord, chan),
            }
            match this.patch.cmp(&other.patch) {
                Ordering::Equal => {}
                ord => return (ord, chan),
            }
            (this.build.cmp(&other.build), chan)
        }

        // `rustc 1.83.0-nightly (da889684c 2024-09-20)`,
        let parts1 = self.inner.split(' ').collect::<Vec<_>>();
        let parts2 = other.inner.split(' ').collect::<Vec<_>>();
        let &[rustc1, version1, _hash1, date1] = parts1.as_slice() else {
            panic!("bad Rust version {self:?}")
        };
        let &[rustc2, version2, _hash2, date2] = parts2.as_slice() else {
            panic!("bad Rust version {other:?}")
        };

        assert_eq!(rustc1, "rustc");
        assert_eq!(rustc2, "rustc");
        assert_eq!(date1.split('-').count(), 3, "bad date {self:?} {other:?}");
        assert_eq!(date2.split('-').count(), 3, "bad date {self:?} {other:?}");
        let sem_version1 = Version::parse(version1).expect("version not semver");
        let sem_version2 = Version::parse(version2).expect("version not semver");

        let (ver, chan) = cmp_semver_rust(&sem_version1, &sem_version2);
        let date = date1.cmp(date2); // ymd can be lexically compared :)
        (ver, chan, date)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RustChannel {
    Stable = 0,
    Beta = 1,
    Nightly = 2,
}

impl RustChannel {
    // might be because Ord is unused? that doesnt make sense for traits tho
    pub(crate) fn from_str(s: &str) -> Self {
        if s.contains("nightly") {
            return Self::Nightly;
        }
        if s.contains("beta") {
            return Self::Beta;
        }
        Self::Stable
    }
}

// /// - `rustc 1.83.0-nightly (da889684c 2024-09-20)`,
// /// - `rustc 1.83.0-beta.3 (f41c7ed98 2024-10-31)`
// /// - `rustc 1.82.0 (f6e511eec 2024-10-15)`,
// /// - `rustc 1.65.0 (897e37553 2022-11-02)`,
// pub struct RustVersion2 {
//     pub version: semver::Version,
//     pub channel: RustChannel,
// }

// pub enum RustChannel2 {
//     Stable,
//     Beta,
//     Nightly { date: String },
// }
