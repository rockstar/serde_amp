serde_amp
==

![build-and-check](https://github.com/rockstar/serde_amp/actions/workflows/build-and-check.yml/badge.svg) ![crates.io](https://img.shields.io/crates/v/serde_amp.svg)

A serialization/deserialization library for [Asynchronous Messaging Protocol](https://amp-protocol.net/)

Usage
--

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct AnStruct {
    count: usize,
    tag: String,
}

fn main() {
    let an_struct = AnStruct { count: 83, tag: "an-tag".to_string() };

    let serialized = serde_amp::to_amp(&an_struct).unwrap();
    let deserialized: AnStruct = serde_amp::from_bytes(&serialized[..]).unwrap();
}
```

**Note:** While `to_amp` can serialize standard types like `usize`, AMP itself is a
key/value protocol, and should be used with key/value types.

License
--

Like Serde, serde_amp is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
