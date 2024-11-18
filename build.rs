fn main() {
    let host_triple = std::env::var("HOST").unwrap(); // from cargo
                                                      // re export build.rs only HOST as idk normal env var for lib.rs
    println!("cargo:rustc-env=BUILD_RS_HOST={}", host_triple);
    println!("cargo:rustc-env=BUILD_RS_HOST_OS={}", get_os_from_triple(&host_triple).unwrap());
}

#[allow(dead_code)]
fn get_os_from_triple(triple: &str) -> Option<&str> {
    const FIRST: usize = 0;
    const SECOND: usize = 1;
    const THIRD: usize = 2;
    const FORTH: usize = 3;
    let parts = triple.split('-').collect::<Vec<_>>();

    // NOTE: all hosts with rustc tools have os in third part
    let os = match parts.len() {
        _ if parts[SECOND] == "apple" && parts[THIRD] == "darwin" => "macos",
        _ if parts[SECOND] == "apple" => parts[THIRD],
        // x86_64-unknown-linux-gnu
        4 => parts[THIRD],
        // thumbv6m-none-eabi is not eabi
        3 if parts[SECOND] == "none" => "none",
        // arm<BLAH>-bruh-<eabi>
        3 if parts[THIRD].contains("eabi") => parts[SECOND],
        // x86_64-unknown-freebsd
        3 => parts[THIRD],
        _ => return None,
    };
    Some(os)
}
