# kg-diag

[![Latest Version](https://img.shields.io/crates/v/kg-diag.svg)](https://crates.io/crates/kg-diag)
[![Documentation](https://docs.rs/kg-diag/badge.svg)](https://docs.rs/kg-diag)
[![Build Status](https://travis-ci.org/Kodegenix/kg-diag.svg?branch=master)](https://travis-ci.org/Kodegenix/kg-diag)
[![codecov](https://codecov.io/gh/kodegenix/kg-diag/branch/master/graph/badge.svg)](https://codecov.io/gh/kodegenix/kg-diag)

Set of crates for error/diagnostic management. I/O routines for reading 
UTF-8 textual data with position tracking.

* crate [`kg-diag`](kg-diag) contains traits `Detail` and `Diag` for diagnostic management; 
contains traits `ByteReader` and `CharReader` for reading textual input with position (line and column) tracking. 
* crate [`kg-diag-derive`](kg-diag-derive) implements macro for `#[derive(Detail)]`

## Builds statuses for Rust channels

| stable            | beta              | nightly           |
|-------------------|-------------------|-------------------|
| [![Build1][3]][4] | [![Build2][2]][4] | [![Build3][1]][4] |

[1]: https://travis-matrix-badges.herokuapp.com/repos/kodegenix/kg-diag/branches/master/1
[2]: https://travis-matrix-badges.herokuapp.com/repos/kodegenix/kg-diag/branches/master/2
[3]: https://travis-matrix-badges.herokuapp.com/repos/kodegenix/kg-diag/branches/master/3
[4]: https://travis-ci.org/kodegenix/kg-diag


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
