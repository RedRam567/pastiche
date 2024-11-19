fn main() {
    let target = std::env::var("TARGET").unwrap();
    println!("cargo::rustc-env=TARGET={target}");

    let host = std::env::var("HOST").unwrap();
    println!("cargo::rustc-env=HOST={host}");
}
