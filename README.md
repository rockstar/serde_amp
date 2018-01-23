serde_amp
==

A serialization/deserialization library for [Asynchronous Messaging Protocol](https://amp-protocol.net/)

Usage
--

```
extern crate serde_amp;

use serde_amp;

#[derive(Deserialize, Serialize)]
struct AnStruct {
    count: usize,
    tag: String
}

fn main() {
    let an_struct = AnStruct { count: 83, tag: "an-tag" };

    let serialized = serde_amp::to_amp(&an_struct).unwrap();
    let deserialized = serde_amp::from_bytes(&serialized[..]).unwrap();
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

at your option.`
