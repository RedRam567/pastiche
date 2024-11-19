A WIP library for exposing non public types, fields, and functions from a crate.

The libraries I use often expose *most* of their lower level details but forget
to mark a few structs or fields as public. Necessitating me to copy over the
struct, a load of traits, all the methods I need, everything those traits
and methods need to import, and all the crates they use. Just for me to be able
to add "`pub`" in front of it.

# Features and Roadmap
- [x] copy definition
- [x] change visibility
- [x] most items: `const`, `enum`, `fn`, `macro_rules`, `static`, `struct`, `trait`, `union`
- [~] attributes
- [x] std library crates
- [ ] re-exports
- ~~[ ] support not specifying crate verion/dont use file system paths~~
    - [x] support not specifying rustc version
- [?] copying modules
- [ ] recursivly copying item dependancies

## Ideas
- enforcing, changing, adding, removing, ignoring:
    - [x] items variant (`Struct`, `fn`, etc)
    - [ ] field names
    - [ ] field types
    - [ ] field size, alignment
    - [ ] associated types and constants
    - [ ] method arguments
    - [ ] method return value

# Example

```rust
use pastiche::pastiche_attr;
use std::num::IntErrorKind;

// TODO: dont make this nessessary for ParseIntError
mod pub_super_hack {
    use super::*;

    #[pastiche_attr]
    #[pastiche_crate = "core@1.82.0"]
    #[pastiche_path = "core::num::error::ParseIntError"]
    pub struct MyParseIntError {
        // body is ignored for now
    }
}
pub use pub_super_hack::*;

fn main() {
    // Directly construct a ParseIntError
    let my_error = MyParseIntError { kind: IntErrorKind::InvalidDigit };
    dbg!(my_error);
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
