# kg-diag

[![Latest Version](https://img.shields.io/crates/v/kg-diag.svg)](https://crates.io/crates/kg-diag)
[![Documentation](https://docs.rs/kg-diag/badge.svg)](https://docs.rs/kg-diag)
[![Build Status](https://travis-ci.org/Kodegenix/kg-diag.svg?branch=master)](https://travis-ci.org/Kodegenix/kg-diag)
[![Coverage Status](https://coveralls.io/repos/github/Kodegenix/kg-diag/badge.svg?branch=master)](https://coveralls.io/github/Kodegenix/kg-diag?branch=master)

Set of crates for error/diagnostic management. I/O routines for reading 
UTF-8 textual data with position tracking.

* crate [`kg-diag`](kg-diag) contains traits `Detail` and `Diag` for diagnostic management; 
contains traits `ByteReader` and `CharReader` for reading textual input with position (line and column) tracking. 
* crate [`kg-diag-derive`](kg-diag-derive) implements macro for `#[derive(Detail)]`

## License

Licensed under either of
* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Copyright

Copyright (c) 2018 Kodegenix Sp. z o.o. [http://www.kodegenix.pl](http://www.kodegenix.pl)
