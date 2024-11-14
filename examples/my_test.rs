
fn main() {
    let str = pastiche::get_span!(use std::str;);
    println!("{}", str);
    let str = r#"use pastiche::get_span;
    
    fn main() {
        let str = pastiche::get_span!(use std::str;);
        panic!("{}", str);
    }"#;
    dbg!(&str[71+12..74+12]);
    panic!()
}
