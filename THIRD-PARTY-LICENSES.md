# Third-Party Licenses

The Trellis CLI binary and Docker images are compiled from Rust source and
statically link the open-source crates listed below. Trellis itself is
proprietary (see [EULA.md](EULA.md)); this file fulfils the attribution
requirements of those upstream dependencies.

All bundled dependencies use permissive or weak-copyleft licenses
(MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0, MPL-2.0, CDLA-Permissive-2.0).

Generated with `cargo-license` from the workspace dependency tree.
Trellis' own crates and internal test crates are excluded.

## Summary by license

| License | Crate count |
|---|---|
| (Apache-2.0 OR MIT) AND Unicode-3.0 | 1 |
| 0BSD OR Apache-2.0 OR MIT | 1 |
| Apache-2.0 | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR CC0-1.0 | 1 |
| Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | 3 |
| Apache-2.0 OR BSD-2-Clause OR MIT | 2 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR CC0-1.0 OR MIT-0 | 1 |
| Apache-2.0 OR ISC OR MIT | 2 |
| Apache-2.0 OR LGPL-2.1-or-later OR MIT | 1 |
| Apache-2.0 OR MIT | 161 |
| Apache-2.0 OR MIT OR Zlib | 5 |
| BSD-2-Clause | 1 |
| BSD-3-Clause | 3 |
| CDLA-Permissive-2.0 | 2 |
| ISC | 2 |
| MIT | 35 |
| MIT OR Unlicense | 4 |
| MPL-2.0 | 3 |
| Unicode-3.0 | 18 |
| Zlib | 1 |

## All dependencies

| Crate | Version | License | Repository |
|---|---|---|---|
| adler2 | 2.0.1 | 0BSD OR Apache-2.0 OR MIT | https://github.com/oyvindln/adler2 |
| anstream | 1.0.0 | Apache-2.0 OR MIT | https://github.com/rust-cli/anstyle.git |
| anstyle | 1.0.14 | Apache-2.0 OR MIT | https://github.com/rust-cli/anstyle.git |
| anstyle-parse | 1.0.0 | Apache-2.0 OR MIT | https://github.com/rust-cli/anstyle.git |
| anstyle-query | 1.1.5 | Apache-2.0 OR MIT | https://github.com/rust-cli/anstyle.git |
| anstyle-wincon | 3.0.11 | Apache-2.0 OR MIT | https://github.com/rust-cli/anstyle.git |
| anyhow | 1.0.102 | Apache-2.0 OR MIT | https://github.com/dtolnay/anyhow |
| arrayref | 0.3.9 | BSD-2-Clause | https://github.com/droundy/arrayref |
| arrayvec | 0.7.6 | Apache-2.0 OR MIT | https://github.com/bluss/arrayvec |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/atomic-waker |
| base64 | 0.22.1 | Apache-2.0 OR MIT | https://github.com/marshallpierce/rust-base64 |
| bitflags | 1.3.2 | Apache-2.0 OR MIT | https://github.com/bitflags/bitflags |
| bitflags | 2.11.1 | Apache-2.0 OR MIT | https://github.com/bitflags/bitflags |
| blake3 | 1.8.5 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR CC0-1.0 | https://github.com/BLAKE3-team/BLAKE3 |
| block-buffer | 0.10.4 | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils |
| bumpalo | 3.20.2 | Apache-2.0 OR MIT | https://github.com/fitzgen/bumpalo |
| bytemuck | 1.25.0 | Apache-2.0 OR MIT OR Zlib | https://github.com/Lokathor/bytemuck |
| bytes | 1.11.1 | MIT | https://github.com/tokio-rs/bytes |
| cfg-if | 1.0.4 | Apache-2.0 OR MIT | https://github.com/rust-lang/cfg-if |
| clap | 4.6.1 | Apache-2.0 OR MIT | https://github.com/clap-rs/clap |
| clap_builder | 4.6.0 | Apache-2.0 OR MIT | https://github.com/clap-rs/clap |
| clap_derive | 4.6.1 | Apache-2.0 OR MIT | https://github.com/clap-rs/clap |
| clap_lex | 1.1.0 | Apache-2.0 OR MIT | https://github.com/clap-rs/clap |
| color_quant | 1.1.0 | MIT | https://github.com/image-rs/color_quant.git |
| colorchoice | 1.0.5 | Apache-2.0 OR MIT | https://github.com/rust-cli/anstyle.git |
| constant_time_eq | 0.4.2 | Apache-2.0 OR CC0-1.0 OR MIT-0 | https://github.com/cesarb/constant_time_eq |
| cpufeatures | 0.2.17 | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils |
| cpufeatures | 0.3.0 | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils |
| crc32fast | 1.5.0 | Apache-2.0 OR MIT | https://github.com/srijs/rust-crc32fast |
| crossbeam-deque | 0.8.6 | Apache-2.0 OR MIT | https://github.com/crossbeam-rs/crossbeam |
| crossbeam-epoch | 0.9.18 | Apache-2.0 OR MIT | https://github.com/crossbeam-rs/crossbeam |
| crossbeam-utils | 0.8.21 | Apache-2.0 OR MIT | https://github.com/crossbeam-rs/crossbeam |
| crypto-common | 0.1.7 | Apache-2.0 OR MIT | https://github.com/RustCrypto/traits |
| data-url | 0.3.2 | Apache-2.0 OR MIT | https://github.com/servo/rust-url |
| digest | 0.10.7 | Apache-2.0 OR MIT | https://github.com/RustCrypto/traits |
| directories | 5.0.1 | Apache-2.0 OR MIT | https://github.com/soc/directories-rs |
| dirs-sys | 0.4.1 | Apache-2.0 OR MIT | https://github.com/dirs-dev/dirs-sys-rs |
| displaydoc | 0.2.6 | Apache-2.0 OR MIT | https://github.com/yaahc/displaydoc |
| either | 1.15.0 | Apache-2.0 OR MIT | https://github.com/rayon-rs/either |
| equivalent | 1.0.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |
| euclid | 0.22.14 | Apache-2.0 OR MIT | https://github.com/servo/euclid |
| fdeflate | 0.3.7 | Apache-2.0 OR MIT | https://github.com/image-rs/fdeflate |
| flate2 | 1.1.9 | Apache-2.0 OR MIT | https://github.com/rust-lang/flate2-rs |
| float-cmp | 0.9.0 | MIT | https://github.com/mikedilger/float-cmp |
| fontconfig-parser | 0.5.8 | MIT | https://github.com/Riey/fontconfig-parser |
| fontdb | 0.18.0 | MIT | https://github.com/RazrFalcon/fontdb |
| form_urlencoded | 1.2.2 | Apache-2.0 OR MIT | https://github.com/servo/rust-url |
| futures-channel | 0.3.32 | Apache-2.0 OR MIT | https://github.com/rust-lang/futures-rs |
| futures-core | 0.3.32 | Apache-2.0 OR MIT | https://github.com/rust-lang/futures-rs |
| futures-io | 0.3.32 | Apache-2.0 OR MIT | https://github.com/rust-lang/futures-rs |
| futures-sink | 0.3.32 | Apache-2.0 OR MIT | https://github.com/rust-lang/futures-rs |
| futures-task | 0.3.32 | Apache-2.0 OR MIT | https://github.com/rust-lang/futures-rs |
| futures-util | 0.3.32 | Apache-2.0 OR MIT | https://github.com/rust-lang/futures-rs |
| generic-array | 0.14.7 | MIT | https://github.com/fizyk20/generic-array.git |
| getrandom | 0.2.17 | Apache-2.0 OR MIT | https://github.com/rust-random/getrandom |
| getrandom | 0.3.4 | Apache-2.0 OR MIT | https://github.com/rust-random/getrandom |
| gif | 0.13.3 | Apache-2.0 OR MIT | https://github.com/image-rs/image-gif |
| hashbrown | 0.17.1 | Apache-2.0 OR MIT | https://github.com/rust-lang/hashbrown |
| heck | 0.5.0 | Apache-2.0 OR MIT | https://github.com/withoutboats/heck |
| hex | 0.4.3 | Apache-2.0 OR MIT | https://github.com/KokaKiwi/rust-hex |
| http | 1.4.2 | Apache-2.0 OR MIT | https://github.com/hyperium/http |
| http-body | 1.0.1 | MIT | https://github.com/hyperium/http-body |
| http-body-util | 0.1.3 | MIT | https://github.com/hyperium/http-body |
| httparse | 1.10.1 | Apache-2.0 OR MIT | https://github.com/seanmonstar/httparse |
| hyper | 1.10.1 | MIT | https://github.com/hyperium/hyper |
| hyper-rustls | 0.27.9 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/hyper-rustls |
| hyper-util | 0.1.20 | MIT | https://github.com/hyperium/hyper-util |
| iconify | 0.3.1 | Apache-2.0 OR MIT | https://github.com/wrapperup/iconify-rs |
| icu_collections | 2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_locale_core | 2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_normalizer | 2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_normalizer_data | 2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_properties | 2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_properties_data | 2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_provider | 2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| idna | 1.1.0 | Apache-2.0 OR MIT | https://github.com/servo/rust-url/ |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | https://github.com/hsivonen/idna_adapter |
| imagesize | 0.12.0 | MIT | https://github.com/Roughsketch/imagesize |
| indexmap | 2.14.0 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |
| ipnet | 2.12.0 | Apache-2.0 OR MIT | https://github.com/krisprice/ipnet |
| is_terminal_polyfill | 1.70.2 | Apache-2.0 OR MIT | https://github.com/polyfill-rs/is_terminal_polyfill |
| itoa | 1.0.18 | Apache-2.0 OR MIT | https://github.com/dtolnay/itoa |
| jpeg-decoder | 0.3.2 | Apache-2.0 OR MIT | https://github.com/image-rs/jpeg-decoder |
| js-sys | 0.3.98 | Apache-2.0 OR MIT | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys |
| kurbo | 0.11.3 | Apache-2.0 OR MIT | https://github.com/linebender/kurbo |
| libc | 0.2.186 | Apache-2.0 OR MIT | https://github.com/rust-lang/libc |
| libredox | 0.1.17 | MIT | https://gitlab.redox-os.org/redox-os/libredox.git |
| litemap | 0.8.2 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| log | 0.4.29 | Apache-2.0 OR MIT | https://github.com/rust-lang/log |
| lru-slab | 0.1.2 | Apache-2.0 OR MIT OR Zlib | https://github.com/Ralith/lru-slab |
| memchr | 2.8.0 | MIT OR Unlicense | https://github.com/BurntSushi/memchr |
| memmap2 | 0.9.10 | Apache-2.0 OR MIT | https://github.com/RazrFalcon/memmap2-rs |
| minimal-lexical | 0.2.1 | Apache-2.0 OR MIT | https://github.com/Alexhuszagh/minimal-lexical |
| miniz_oxide | 0.8.9 | Apache-2.0 OR MIT OR Zlib | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| mio | 1.2.1 | MIT | https://github.com/tokio-rs/mio |
| nom | 7.1.3 | MIT | https://github.com/Geal/nom |
| num-traits | 0.2.19 | Apache-2.0 OR MIT | https://github.com/rust-num/num-traits |
| once_cell | 1.21.4 | Apache-2.0 OR MIT | https://github.com/matklad/once_cell |
| once_cell_polyfill | 1.70.2 | Apache-2.0 OR MIT | https://github.com/polyfill-rs/once_cell_polyfill |
| option-ext | 0.2.0 | MPL-2.0 | https://github.com/soc/option-ext.git |
| percent-encoding | 2.3.2 | Apache-2.0 OR MIT | https://github.com/servo/rust-url/ |
| pico-args | 0.5.0 | MIT | https://github.com/RazrFalcon/pico-args |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project-lite |
| png | 0.17.16 | Apache-2.0 OR MIT | https://github.com/image-rs/image-png |
| potential_utf | 0.1.5 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| ppv-lite86 | 0.2.21 | Apache-2.0 OR MIT | https://github.com/cryptocorrosion/cryptocorrosion |
| proc-macro2 | 1.0.106 | Apache-2.0 OR MIT | https://github.com/dtolnay/proc-macro2 |
| quinn | 0.11.9 | Apache-2.0 OR MIT | https://github.com/quinn-rs/quinn |
| quinn-proto | 0.11.14 | Apache-2.0 OR MIT | https://github.com/quinn-rs/quinn |
| quinn-udp | 0.5.14 | Apache-2.0 OR MIT | https://github.com/quinn-rs/quinn |
| quote | 1.0.45 | Apache-2.0 OR MIT | https://github.com/dtolnay/quote |
| r-efi | 5.3.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT | https://github.com/r-efi/r-efi |
| rand | 0.9.4 | Apache-2.0 OR MIT | https://github.com/rust-random/rand |
| rand_chacha | 0.9.0 | Apache-2.0 OR MIT | https://github.com/rust-random/rand |
| rand_core | 0.9.5 | Apache-2.0 OR MIT | https://github.com/rust-random/rand |
| rayon | 1.12.0 | Apache-2.0 OR MIT | https://github.com/rayon-rs/rayon |
| rayon-core | 1.13.0 | Apache-2.0 OR MIT | https://github.com/rayon-rs/rayon |
| redox_users | 0.4.6 | MIT | https://gitlab.redox-os.org/redox-os/users |
| reqwest | 0.12.28 | Apache-2.0 OR MIT | https://github.com/seanmonstar/reqwest |
| resvg | 0.42.0 | MPL-2.0 | https://github.com/RazrFalcon/resvg |
| rgb | 0.8.53 | MIT | https://github.com/kornelski/rust-rgb |
| ring | 0.17.14 | Apache-2.0 AND ISC | https://github.com/briansmith/ring |
| roxmltree | 0.20.0 | Apache-2.0 OR MIT | https://github.com/RazrFalcon/roxmltree |
| rustc-hash | 2.1.2 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash |
| rustls | 0.23.40 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls |
| rustls-pki-types | 1.14.1 | Apache-2.0 OR MIT | https://github.com/rustls/pki-types |
| rustls-webpki | 0.103.13 | ISC | https://github.com/rustls/webpki |
| rustybuzz | 0.14.1 | MIT | https://github.com/RazrFalcon/rustybuzz |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | https://github.com/dtolnay/ryu |
| same-file | 1.0.6 | MIT OR Unlicense | https://github.com/BurntSushi/same-file |
| serde | 1.0.228 | Apache-2.0 OR MIT | https://github.com/serde-rs/serde |
| serde_core | 1.0.228 | Apache-2.0 OR MIT | https://github.com/serde-rs/serde |
| serde_derive | 1.0.228 | Apache-2.0 OR MIT | https://github.com/serde-rs/serde |
| serde_json | 1.0.149 | Apache-2.0 OR MIT | https://github.com/serde-rs/json |
| serde_spanned | 1.1.1 | Apache-2.0 OR MIT | https://github.com/toml-rs/toml |
| serde_urlencoded | 0.7.1 | Apache-2.0 OR MIT | https://github.com/nox/serde_urlencoded |
| sha2 | 0.10.9 | Apache-2.0 OR MIT | https://github.com/RustCrypto/hashes |
| simd-adler32 | 0.3.9 | MIT | https://github.com/mcountryman/simd-adler32 |
| simplecss | 0.2.2 | Apache-2.0 OR MIT | https://github.com/linebender/simplecss |
| siphasher | 1.0.3 | Apache-2.0 OR MIT | https://github.com/jedisct1/rust-siphash |
| slab | 0.4.12 | MIT | https://github.com/tokio-rs/slab |
| slotmap | 1.1.1 | Zlib | https://github.com/orlp/slotmap |
| smallvec | 1.15.1 | Apache-2.0 OR MIT | https://github.com/servo/rust-smallvec |
| socket2 | 0.6.4 | Apache-2.0 OR MIT | https://github.com/rust-lang/socket2 |
| stable_deref_trait | 1.2.1 | Apache-2.0 OR MIT | https://github.com/storyyeller/stable_deref_trait |
| strict-num | 0.1.1 | MIT | https://github.com/RazrFalcon/strict-num |
| strsim | 0.11.1 | MIT | https://github.com/rapidfuzz/strsim-rs |
| subtle | 2.6.1 | BSD-3-Clause | https://github.com/dalek-cryptography/subtle |
| svgtypes | 0.15.3 | Apache-2.0 OR MIT | https://github.com/linebender/svgtypes |
| syn | 2.0.117 | Apache-2.0 OR MIT | https://github.com/dtolnay/syn |
| sync_wrapper | 1.0.2 | Apache-2.0 | https://github.com/Actyx/sync_wrapper |
| synstructure | 0.13.2 | MIT | https://github.com/mystor/synstructure |
| thiserror | 1.0.69 | Apache-2.0 OR MIT | https://github.com/dtolnay/thiserror |
| thiserror | 2.0.18 | Apache-2.0 OR MIT | https://github.com/dtolnay/thiserror |
| thiserror-impl | 1.0.69 | Apache-2.0 OR MIT | https://github.com/dtolnay/thiserror |
| thiserror-impl | 2.0.18 | Apache-2.0 OR MIT | https://github.com/dtolnay/thiserror |
| tiny-skia | 0.11.4 | BSD-3-Clause | https://github.com/RazrFalcon/tiny-skia |
| tiny-skia-path | 0.11.4 | BSD-3-Clause | https://github.com/RazrFalcon/tiny-skia/tree/master/path |
| tinystr | 0.8.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| tinyvec | 1.11.0 | Apache-2.0 OR MIT OR Zlib | https://github.com/Lokathor/tinyvec |
| tinyvec_macros | 0.1.1 | Apache-2.0 OR MIT OR Zlib | https://github.com/Soveu/tinyvec_macros |
| tokio | 1.52.3 | MIT | https://github.com/tokio-rs/tokio |
| tokio-rustls | 0.26.4 | Apache-2.0 OR MIT | https://github.com/rustls/tokio-rustls |
| toml | 1.1.2+spec-1.1.0 | Apache-2.0 OR MIT | https://github.com/toml-rs/toml |
| toml_datetime | 1.1.1+spec-1.1.0 | Apache-2.0 OR MIT | https://github.com/toml-rs/toml |
| toml_parser | 1.1.2+spec-1.1.0 | Apache-2.0 OR MIT | https://github.com/toml-rs/toml |
| toml_writer | 1.1.1+spec-1.1.0 | Apache-2.0 OR MIT | https://github.com/toml-rs/toml |
| tower | 0.5.3 | MIT | https://github.com/tower-rs/tower |
| tower-http | 0.6.11 | MIT | https://github.com/tower-rs/tower-http |
| tower-layer | 0.3.3 | MIT | https://github.com/tower-rs/tower |
| tower-service | 0.3.3 | MIT | https://github.com/tower-rs/tower |
| tracing | 0.1.44 | MIT | https://github.com/tokio-rs/tracing |
| tracing-core | 0.1.36 | MIT | https://github.com/tokio-rs/tracing |
| try-lock | 0.2.5 | MIT | https://github.com/seanmonstar/try-lock |
| ttf-parser | 0.21.1 | Apache-2.0 OR MIT | https://github.com/RazrFalcon/ttf-parser |
| typenum | 1.20.1 | Apache-2.0 OR MIT | https://github.com/paholg/typenum |
| unicode-bidi | 0.3.18 | Apache-2.0 OR MIT | https://github.com/servo/unicode-bidi |
| unicode-bidi-mirroring | 0.2.0 | Apache-2.0 OR MIT | https://github.com/RazrFalcon/unicode-bidi-mirroring |
| unicode-ccc | 0.2.0 | Apache-2.0 OR MIT | https://github.com/RazrFalcon/unicode-ccc |
| unicode-ident | 1.0.24 | (Apache-2.0 OR MIT) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| unicode-properties | 0.1.4 | Apache-2.0 OR MIT | https://github.com/unicode-rs/unicode-properties |
| unicode-script | 0.5.8 | Apache-2.0 OR MIT | https://github.com/unicode-rs/unicode-script |
| unicode-vo | 0.1.0 | Apache-2.0 OR MIT | https://github.com/RazrFalcon/unicode-vo |
| untrusted | 0.9.0 | ISC | https://github.com/briansmith/untrusted |
| ureq | 2.12.1 | Apache-2.0 OR MIT | https://github.com/algesten/ureq |
| url | 2.5.8 | Apache-2.0 OR MIT | https://github.com/servo/rust-url |
| usvg | 0.42.0 | MPL-2.0 | https://github.com/RazrFalcon/resvg |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT | https://github.com/hsivonen/utf8_iter |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | https://github.com/alacritty/vte |
| walkdir | 2.5.0 | MIT OR Unlicense | https://github.com/BurntSushi/walkdir |
| want | 0.3.1 | MIT | https://github.com/seanmonstar/want |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | https://github.com/bytecodealliance/wasi |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | https://github.com/bytecodealliance/wasi-rs |
| wasm-bindgen | 0.2.121 | Apache-2.0 OR MIT | https://github.com/wasm-bindgen/wasm-bindgen |
| wasm-bindgen-futures | 0.4.71 | Apache-2.0 OR MIT | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures |
| wasm-bindgen-macro | 0.2.121 | Apache-2.0 OR MIT | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro |
| wasm-bindgen-macro-support | 0.2.121 | Apache-2.0 OR MIT | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support |
| wasm-bindgen-shared | 0.2.121 | Apache-2.0 OR MIT | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared |
| web-sys | 0.3.98 | Apache-2.0 OR MIT | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys |
| web-time | 1.1.0 | Apache-2.0 OR MIT | https://github.com/daxpedda/web-time |
| webpki-roots | 0.26.11 | CDLA-Permissive-2.0 | https://github.com/rustls/webpki-roots |
| webpki-roots | 1.0.7 | CDLA-Permissive-2.0 | https://github.com/rustls/webpki-roots |
| weezl | 0.1.12 | Apache-2.0 OR MIT | https://github.com/image-rs/weezl |
| winapi-util | 0.1.11 | MIT OR Unlicense | https://github.com/BurntSushi/winapi-util |
| windows-link | 0.2.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows-sys | 0.48.0 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows-sys | 0.52.0 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows-sys | 0.60.2 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows-sys | 0.61.2 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows-targets | 0.48.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows-targets | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows-targets | 0.53.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_aarch64_gnullvm | 0.48.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_aarch64_gnullvm | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_aarch64_gnullvm | 0.53.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_aarch64_msvc | 0.48.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_aarch64_msvc | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_aarch64_msvc | 0.53.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_i686_gnu | 0.48.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_i686_gnu | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_i686_gnu | 0.53.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_i686_gnullvm | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_i686_gnullvm | 0.53.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_i686_msvc | 0.48.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_i686_msvc | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_i686_msvc | 0.53.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_gnu | 0.48.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_gnu | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_gnu | 0.53.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_gnullvm | 0.48.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_gnullvm | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_gnullvm | 0.53.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_msvc | 0.48.5 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_msvc | 0.52.6 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| windows_x86_64_msvc | 0.53.1 | Apache-2.0 OR MIT | https://github.com/microsoft/windows-rs |
| winnow | 1.0.2 | MIT | https://github.com/winnow-rs/winnow |
| wit-bindgen | 0.57.1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | https://github.com/bytecodealliance/wit-bindgen |
| writeable | 0.6.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| xmlwriter | 0.1.0 | MIT | https://github.com/RazrFalcon/xmlwriter |
| yoke | 0.8.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| yoke-derive | 0.8.2 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zerocopy | 0.8.48 | Apache-2.0 OR BSD-2-Clause OR MIT | https://github.com/google/zerocopy |
| zerocopy-derive | 0.8.48 | Apache-2.0 OR BSD-2-Clause OR MIT | https://github.com/google/zerocopy |
| zerofrom | 0.1.8 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zerofrom-derive | 0.1.7 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zeroize | 1.9.0 | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils |
| zerotrie | 0.2.4 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zerovec | 0.11.6 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zerovec-derive | 0.11.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zmij | 1.0.21 | MIT | https://github.com/dtolnay/zmij |
