# Pikchr-sys

[Pikchr] bindings for the Rust programming language.


## High-level API

This crate provides bindings to the raw low-level C API. For a higher-level safe API to work with Pikchr see [pikt].


## Release support

- `pikchr-sys` v0.1: `pikchr` checkout d9e1502ed74c6aabcb055cf7983c897a28cbe09c


## License

pikchr-sys is licensed under the [BSD Zero Clause License](LICENSE).

It bundles the compiled C89 source code ([`pikchr.c`]) and headers ([`pikchr.h`]) from [Pikchr] which is also licensed under the BSD Zero Clause license.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in pikchr-sys by you shall be licensed as above without any additional terms or conditions.


[pikt]: https://github.com/arnau/pikt/
[Pikchr]: https://pikchr.org/
[`pikchr.c`]: pikchr/pikchr.c
[`pikchr.h`]: pikchr/pikchr.h
