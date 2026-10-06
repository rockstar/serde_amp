serde_amp
==

![build-and-check](https://github.com/rockstar/serde_amp/actions/workflows/build-and-check.yml/badge.svg)

Rust tooling for [Asynchronous Messaging Protocol](https://amp-protocol.net/)
(AMP), the key/value remoting protocol from Twisted.

This repository is a Cargo workspace. Each crate has its own README with usage
details.

| Crate                         | Description                                                    |
|-------------------------------|----------------------------------------------------------------|
| [amp-protocol](amp-protocol/) | The wire format, box type, and request/response message types |
| [serde_amp](serde_amp/)  | [serde](https://serde.rs) serialization and deserialization of AMP boxes |

License
--

Like Serde, every crate in this workspace is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
