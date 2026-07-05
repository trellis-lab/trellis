# Third-Party Licenses

The Trellis CLI binary and Docker images are compiled from Rust source and
statically link the open-source crates listed below. Trellis itself is
proprietary (see [EULA.md](EULA.md)); this file fulfils the attribution
requirements of those upstream dependencies.

All bundled dependencies use permissive or weak-copyleft licenses
(MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0, MPL-2.0, CDLA-Permissive-2.0).

Generated with `cargo-license` from the workspace dependency tree.
Trellis' own crates, internal test crates, development and build dependencies are excluded.

## Summary by license

| License | Crates |
|---|---|
| Apache-2.0 OR MIT | 220 |
| MIT | 98 |
| Unicode-3.0 | 18 |
| MPL-2.0 | 10 |
| Apache-2.0 | 7 |
| MIT OR Unlicense | 7 |
| Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | 5 |
| Apache-2.0 OR MIT OR Zlib | 5 |
| BSD-3-Clause | 3 |
| (Apache-2.0 OR MIT) AND Unicode-3.0 | 2 |
| Apache-2.0 OR BSD-2-Clause OR MIT | 2 |
| Apache-2.0 OR ISC OR MIT | 2 |
| CDLA-Permissive-2.0 | 2 |
| ISC | 2 |
| 0BSD OR Apache-2.0 OR MIT | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR CC0-1.0 OR MIT-0 | 1 |
| Apache-2.0 OR GPL-2.0 | 1 |
| Apache-2.0 OR LGPL-2.1-or-later OR MIT | 1 |
| Apache-2.0 WITH LLVM-exception OR BSL-1.0 | 1 |
| BSD-2-Clause | 1 |
| Zlib | 1 |

**Total: 392 dependencies**

## All dependencies

| Crate | Version | License | Repository |
|---|---|---|---|
| adler2 | 2.0.1 | 0BSD OR Apache-2.0 OR MIT | [https://github.com/oyvindln/adler2](https://github.com/oyvindln/adler2) |
| ahash | 0.7.8 | Apache-2.0 OR MIT | [https://github.com/tkaitchuck/ahash](https://github.com/tkaitchuck/ahash) |
| ahash | 0.8.12 | Apache-2.0 OR MIT | [https://github.com/tkaitchuck/ahash](https://github.com/tkaitchuck/ahash) |
| aho-corasick | 1.1.4 | MIT OR Unlicense | [https://github.com/BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick) |
| allocator-api2 | 0.2.21 | Apache-2.0 OR MIT | [https://github.com/zakarumych/allocator-api2](https://github.com/zakarumych/allocator-api2) |
| anstream | 1.0.0 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-parse | 1.0.0 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-query | 1.1.5 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-wincon | 3.0.11 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anyhow | 1.0.102 | Apache-2.0 OR MIT | [https://github.com/dtolnay/anyhow](https://github.com/dtolnay/anyhow) |
| arrayref | 0.3.9 | BSD-2-Clause | [https://github.com/droundy/arrayref](https://github.com/droundy/arrayref) |
| arrayvec | 0.7.6 | Apache-2.0 OR MIT | [https://github.com/bluss/arrayvec](https://github.com/bluss/arrayvec) |
| assert-json-diff | 2.0.2 | MIT | [https://github.com/davidpdrsn/assert-json-diff.git](https://github.com/davidpdrsn/assert-json-diff.git) |
| assert_cmd | 2.2.2 | Apache-2.0 OR MIT | [https://github.com/assert-rs/assert_cmd.git](https://github.com/assert-rs/assert_cmd.git) |
| assert_fs | 1.1.4 | Apache-2.0 OR MIT | [https://github.com/assert-rs/assert_fs.git](https://github.com/assert-rs/assert_fs.git) |
| async-attributes | 1.1.2 | Apache-2.0 OR MIT | [https://github.com/async-rs/async-attributes](https://github.com/async-rs/async-attributes) |
| async-channel | 1.9.0 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-channel](https://github.com/smol-rs/async-channel) |
| async-channel | 2.5.0 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-channel](https://github.com/smol-rs/async-channel) |
| async-executor | 1.14.0 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-executor](https://github.com/smol-rs/async-executor) |
| async-global-executor | 2.4.1 | Apache-2.0 OR MIT | [https://github.com/Keruspe/async-global-executor](https://github.com/Keruspe/async-global-executor) |
| async-io | 2.6.0 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-io](https://github.com/smol-rs/async-io) |
| async-lock | 3.4.2 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-lock](https://github.com/smol-rs/async-lock) |
| async-object-pool | 0.1.5 | MIT | [https://github.com/alexliesenfeld/async-object-pool](https://github.com/alexliesenfeld/async-object-pool) |
| async-process | 2.5.0 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-process](https://github.com/smol-rs/async-process) |
| async-signal | 0.2.14 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-signal](https://github.com/smol-rs/async-signal) |
| async-task | 4.7.1 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-task](https://github.com/smol-rs/async-task) |
| async-trait | 0.1.89 | Apache-2.0 OR MIT | [https://github.com/dtolnay/async-trait](https://github.com/dtolnay/async-trait) |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | [https://github.com/smol-rs/atomic-waker](https://github.com/smol-rs/atomic-waker) |
| base64 | 0.21.7 | Apache-2.0 OR MIT | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| base64 | 0.22.1 | Apache-2.0 OR MIT | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| base64-simd | 0.7.0 | MIT | [https://github.com/Nugine/simd](https://github.com/Nugine/simd) |
| base64-simd | 0.8.0 | MIT | [https://github.com/Nugine/simd](https://github.com/Nugine/simd) |
| basic-cookies | 0.1.5 | MIT | [https://github.com/drjokepu/basic-cookies](https://github.com/drjokepu/basic-cookies) |
| bitflags | 1.3.2 | Apache-2.0 OR MIT | [https://github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| bitflags | 2.11.1 | Apache-2.0 OR MIT | [https://github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| bitvec | 1.1.1 | MIT | [https://github.com/bitvecto-rs/bitvec](https://github.com/bitvecto-rs/bitvec) |
| block-buffer | 0.10.4 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| blocking | 1.6.2 | Apache-2.0 OR MIT | [https://github.com/smol-rs/blocking](https://github.com/smol-rs/blocking) |
| bstr | 1.12.3 | Apache-2.0 OR MIT | [https://github.com/BurntSushi/bstr](https://github.com/BurntSushi/bstr) |
| bumpalo | 3.19.0 | Apache-2.0 OR MIT | [https://github.com/fitzgen/bumpalo](https://github.com/fitzgen/bumpalo) |
| bytecheck | 0.6.12 | MIT | [https://github.com/djkoloski/bytecheck](https://github.com/djkoloski/bytecheck) |
| bytemuck | 1.25.0 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Lokathor/bytemuck](https://github.com/Lokathor/bytemuck) |
| bytes | 1.11.1 | MIT | [https://github.com/tokio-rs/bytes](https://github.com/tokio-rs/bytes) |
| castaway | 0.2.4 | MIT | [https://github.com/sagebind/castaway](https://github.com/sagebind/castaway) |
| cfg-if | 1.0.4 | Apache-2.0 OR MIT | [https://github.com/rust-lang/cfg-if](https://github.com/rust-lang/cfg-if) |
| clap | 4.6.1 | Apache-2.0 OR MIT | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_builder | 4.6.0 | Apache-2.0 OR MIT | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_lex | 1.1.0 | Apache-2.0 OR MIT | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| cobs | 0.3.0 | Apache-2.0 OR MIT | [https://github.com/jamesmunns/cobs.rs](https://github.com/jamesmunns/cobs.rs) |
| color_quant | 1.1.0 | MIT | [https://github.com/image-rs/color_quant.git](https://github.com/image-rs/color_quant.git) |
| colorchoice | 1.0.5 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| compact_str | 0.9.1 | MIT | [https://github.com/ParkMyCar/compact_str](https://github.com/ParkMyCar/compact_str) |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | [https://github.com/smol-rs/concurrent-queue](https://github.com/smol-rs/concurrent-queue) |
| console | 0.16.4 | MIT | [https://github.com/console-rs/console](https://github.com/console-rs/console) |
| const-str | 0.3.2 | MIT | [https://github.com/Nugine/const-str](https://github.com/Nugine/const-str) |
| const-str-proc-macro | 0.3.2 | MIT | [https://github.com/Nugine/const-str](https://github.com/Nugine/const-str) |
| constant_time_eq | 0.4.2 | Apache-2.0 OR CC0-1.0 OR MIT-0 | [https://github.com/cesarb/constant_time_eq](https://github.com/cesarb/constant_time_eq) |
| cow-utils | 0.1.3 | MIT | [https://github.com/RReverser/cow-utils-rs](https://github.com/RReverser/cow-utils-rs) |
| cpufeatures | 0.2.17 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| cpufeatures | 0.3.0 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| crc32fast | 1.5.0 | Apache-2.0 OR MIT | [https://github.com/srijs/rust-crc32fast](https://github.com/srijs/rust-crc32fast) |
| crossbeam-deque | 0.8.6 | Apache-2.0 OR MIT | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| crossbeam-epoch | 0.9.18 | Apache-2.0 OR MIT | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| crossbeam-utils | 0.8.21 | Apache-2.0 OR MIT | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| crypto-common | 0.1.7 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| cssparser | 0.33.0 | MPL-2.0 | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| cssparser-color | 0.1.0 | MPL-2.0 | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| cssparser-macros | 0.6.1 | MPL-2.0 | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| dashmap | 5.5.3 | MIT | [https://github.com/xacrimon/dashmap](https://github.com/xacrimon/dashmap) |
| data-encoding | 2.11.0 | MIT | [https://github.com/ia0/data-encoding](https://github.com/ia0/data-encoding) |
| data-url | 0.1.1 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| data-url | 0.3.2 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| difflib | 0.4.0 | MIT | [https://github.com/DimaKudosh/difflib](https://github.com/DimaKudosh/difflib) |
| digest | 0.10.7 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| dirs-sys | 0.4.1 | Apache-2.0 OR MIT | [https://github.com/dirs-dev/dirs-sys-rs](https://github.com/dirs-dev/dirs-sys-rs) |
| dragonbox_ecma | 0.0.5 | Apache-2.0 WITH LLVM-exception OR BSL-1.0 | [https://github.com/magic-akari/dragonbox](https://github.com/magic-akari/dragonbox) |
| dtoa | 1.0.11 | Apache-2.0 OR MIT | [https://github.com/dtolnay/dtoa](https://github.com/dtolnay/dtoa) |
| dtoa-short | 0.3.5 | MPL-2.0 | [https://github.com/upsuper/dtoa-short](https://github.com/upsuper/dtoa-short) |
| either | 1.15.0 | Apache-2.0 OR MIT | [https://github.com/rayon-rs/either](https://github.com/rayon-rs/either) |
| embedded-io | 0.4.0 | Apache-2.0 OR MIT | [https://github.com/embassy-rs/embedded-io](https://github.com/embassy-rs/embedded-io) |
| embedded-io | 0.6.1 | Apache-2.0 OR MIT | [https://github.com/rust-embedded/embedded-hal](https://github.com/rust-embedded/embedded-hal) |
| encode_unicode | 1.0.0 | Apache-2.0 OR MIT | [https://github.com/tormol/encode_unicode](https://github.com/tormol/encode_unicode) |
| equivalent | 1.0.2 | Apache-2.0 OR MIT | [https://github.com/indexmap-rs/equivalent](https://github.com/indexmap-rs/equivalent) |
| errno | 0.3.14 | Apache-2.0 OR MIT | [https://github.com/lambda-fairy/rust-errno](https://github.com/lambda-fairy/rust-errno) |
| euclid | 0.22.14 | Apache-2.0 OR MIT | [https://github.com/servo/euclid](https://github.com/servo/euclid) |
| event-listener | 2.5.3 | Apache-2.0 OR MIT | [https://github.com/smol-rs/event-listener](https://github.com/smol-rs/event-listener) |
| event-listener | 5.4.1 | Apache-2.0 OR MIT | [https://github.com/smol-rs/event-listener](https://github.com/smol-rs/event-listener) |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | [https://github.com/smol-rs/event-listener-strategy](https://github.com/smol-rs/event-listener-strategy) |
| fastrand | 2.4.1 | Apache-2.0 OR MIT | [https://github.com/smol-rs/fastrand](https://github.com/smol-rs/fastrand) |
| fdeflate | 0.3.7 | Apache-2.0 OR MIT | [https://github.com/image-rs/fdeflate](https://github.com/image-rs/fdeflate) |
| flate2 | 1.1.9 | Apache-2.0 OR MIT | [https://github.com/rust-lang/flate2-rs](https://github.com/rust-lang/flate2-rs) |
| float-cmp | 0.9.0 | MIT | [https://github.com/mikedilger/float-cmp](https://github.com/mikedilger/float-cmp) |
| float-cmp | 0.10.0 | MIT | [https://github.com/mikedilger/float-cmp](https://github.com/mikedilger/float-cmp) |
| fnv | 1.0.7 | Apache-2.0 OR MIT | [https://github.com/servo/rust-fnv](https://github.com/servo/rust-fnv) |
| fontconfig-parser | 0.5.8 | MIT | [https://github.com/Riey/fontconfig-parser](https://github.com/Riey/fontconfig-parser) |
| fontdb | 0.18.0 | MIT | [https://github.com/RazrFalcon/fontdb](https://github.com/RazrFalcon/fontdb) |
| form_urlencoded | 1.2.2 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| funty | 2.0.0 | MIT | [https://github.com/myrrlyn/funty](https://github.com/myrrlyn/funty) |
| futures-channel | 0.3.32 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-core | 0.3.32 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-io | 0.3.32 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT | [https://github.com/smol-rs/futures-lite](https://github.com/smol-rs/futures-lite) |
| futures-macro | 0.3.32 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-sink | 0.3.32 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-task | 0.3.32 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-util | 0.3.32 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| generic-array | 0.14.7 | MIT | [https://github.com/fizyk20/generic-array.git](https://github.com/fizyk20/generic-array.git) |
| getrandom | 0.2.17 | Apache-2.0 OR MIT | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| getrandom | 0.3.4 | Apache-2.0 OR MIT | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| gif | 0.13.3 | Apache-2.0 OR MIT | [https://github.com/image-rs/image-gif](https://github.com/image-rs/image-gif) |
| globset | 0.4.18 | MIT OR Unlicense | [https://github.com/BurntSushi/ripgrep/tree/master/crates/globset](https://github.com/BurntSushi/ripgrep/tree/master/crates/globset) |
| globwalk | 0.9.1 | MIT | [https://github.com/gilnaa/globwalk](https://github.com/gilnaa/globwalk) |
| gloo-timers | 0.3.0 | Apache-2.0 OR MIT | [https://github.com/rustwasm/gloo/tree/master/crates/timers](https://github.com/rustwasm/gloo/tree/master/crates/timers) |
| hashbrown | 0.12.3 | Apache-2.0 OR MIT | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.14.5 | Apache-2.0 OR MIT | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.16.1 | Apache-2.0 OR MIT | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.17.1 | Apache-2.0 OR MIT | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| hermit-abi | 0.5.2 | Apache-2.0 OR MIT | [https://github.com/hermit-os/hermit-rs](https://github.com/hermit-os/hermit-rs) |
| http-body | 0.4.6 | MIT | [https://github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| http-body | 1.0.1 | MIT | [https://github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| http-body-util | 0.1.3 | MIT | [https://github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| httparse | 1.10.1 | Apache-2.0 OR MIT | [https://github.com/seanmonstar/httparse](https://github.com/seanmonstar/httparse) |
| httpdate | 1.0.3 | Apache-2.0 OR MIT | [https://github.com/pyfisch/httpdate](https://github.com/pyfisch/httpdate) |
| httpmock | 0.7.0 | MIT | [https://github.com/alexliesenfeld/httpmock](https://github.com/alexliesenfeld/httpmock) |
| hyper | 0.14.32 | MIT | [https://github.com/hyperium/hyper](https://github.com/hyperium/hyper) |
| hyper | 1.10.1 | MIT | [https://github.com/hyperium/hyper](https://github.com/hyperium/hyper) |
| hyper-rustls | 0.27.9 | Apache-2.0 OR ISC OR MIT | [https://github.com/rustls/hyper-rustls](https://github.com/rustls/hyper-rustls) |
| hyper-util | 0.1.20 | MIT | [https://github.com/hyperium/hyper-util](https://github.com/hyperium/hyper-util) |
| icu_collections | 2.2.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_locale_core | 2.2.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_normalizer | 2.2.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_normalizer_data | 2.2.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_properties | 2.2.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_properties_data | 2.2.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_provider | 2.2.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| idna | 1.1.0 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | [https://github.com/hsivonen/idna_adapter](https://github.com/hsivonen/idna_adapter) |
| ignore | 0.4.27 | MIT OR Unlicense | [https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) |
| imagesize | 0.12.0 | MIT | [https://github.com/Roughsketch/imagesize](https://github.com/Roughsketch/imagesize) |
| indexmap | 2.14.0 | Apache-2.0 OR MIT | [https://github.com/indexmap-rs/indexmap](https://github.com/indexmap-rs/indexmap) |
| insta | 1.48.0 | Apache-2.0 | [https://github.com/mitsuhiko/insta](https://github.com/mitsuhiko/insta) |
| ipnet | 2.12.0 | Apache-2.0 OR MIT | [https://github.com/krisprice/ipnet](https://github.com/krisprice/ipnet) |
| is_terminal_polyfill | 1.70.2 | Apache-2.0 OR MIT | [https://github.com/polyfill-rs/is_terminal_polyfill](https://github.com/polyfill-rs/is_terminal_polyfill) |
| itertools | 0.10.5 | Apache-2.0 OR MIT | [https://github.com/rust-itertools/itertools](https://github.com/rust-itertools/itertools) |
| itertools | 0.14.0 | Apache-2.0 OR MIT | [https://github.com/rust-itertools/itertools](https://github.com/rust-itertools/itertools) |
| itoa | 1.0.18 | Apache-2.0 OR MIT | [https://github.com/dtolnay/itoa](https://github.com/dtolnay/itoa) |
| jpeg-decoder | 0.3.2 | Apache-2.0 OR MIT | [https://github.com/image-rs/jpeg-decoder](https://github.com/image-rs/jpeg-decoder) |
| js-sys | 0.3.98 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| json-escape-simd | 3.0.2 | MIT | [https://github.com/napi-rs/json-escape-simd](https://github.com/napi-rs/json-escape-simd) |
| kurbo | 0.11.3 | Apache-2.0 OR MIT | [https://github.com/linebender/kurbo](https://github.com/linebender/kurbo) |
| kv-log-macro | 1.0.7 | Apache-2.0 OR MIT | [https://github.com/yoshuawuyts/kv-log-macro](https://github.com/yoshuawuyts/kv-log-macro) |
| lalrpop-util | 0.20.2 | Apache-2.0 OR MIT | [https://github.com/lalrpop/lalrpop](https://github.com/lalrpop/lalrpop) |
| lazy_static | 1.5.0 | Apache-2.0 OR MIT | [https://github.com/rust-lang-nursery/lazy-static.rs](https://github.com/rust-lang-nursery/lazy-static.rs) |
| levenshtein | 1.0.5 | MIT | [https://github.com/wooorm/levenshtein-rs](https://github.com/wooorm/levenshtein-rs) |
| libredox | 0.1.17 | MIT | [https://gitlab.redox-os.org/redox-os/libredox.git](https://gitlab.redox-os.org/redox-os/libredox.git) |
| lightningcss | 1.0.0-alpha.71 | MPL-2.0 | [https://github.com/parcel-bundler/lightningcss](https://github.com/parcel-bundler/lightningcss) |
| lightningcss-derive | 1.0.0-alpha.43 | MPL-2.0 | [https://github.com/parcel-bundler/lightningcss](https://github.com/parcel-bundler/lightningcss) |
| linux-raw-sys | 0.12.1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/sunfishcode/linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) |
| litemap | 0.8.2 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| lock_api | 0.4.14 | Apache-2.0 OR MIT | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| log | 0.4.29 | Apache-2.0 OR MIT | [https://github.com/rust-lang/log](https://github.com/rust-lang/log) |
| lru-slab | 0.1.2 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Ralith/lru-slab](https://github.com/Ralith/lru-slab) |
| matches | 0.1.10 | MIT | [https://github.com/SimonSapin/rust-std-candidates](https://github.com/SimonSapin/rust-std-candidates) |
| memchr | 2.8.0 | MIT OR Unlicense | [https://github.com/BurntSushi/memchr](https://github.com/BurntSushi/memchr) |
| memmap2 | 0.9.10 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/memmap2-rs](https://github.com/RazrFalcon/memmap2-rs) |
| minify-html | 0.18.1 | MIT | [https://github.com/wilsonzlin/minify-html.git](https://github.com/wilsonzlin/minify-html.git) |
| minify-html-common | 0.0.3 | MIT | [https://github.com/wilsonzlin/minify-html.git](https://github.com/wilsonzlin/minify-html.git) |
| minimal-lexical | 0.2.1 | Apache-2.0 OR MIT | [https://github.com/Alexhuszagh/minimal-lexical](https://github.com/Alexhuszagh/minimal-lexical) |
| miniz_oxide | 0.8.9 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| mio | 1.2.1 | MIT | [https://github.com/tokio-rs/mio](https://github.com/tokio-rs/mio) |
| nom | 7.1.3 | MIT | [https://github.com/Geal/nom](https://github.com/Geal/nom) |
| nonmax | 0.5.5 | Apache-2.0 OR MIT | [https://github.com/LPGhatguy/nonmax](https://github.com/LPGhatguy/nonmax) |
| normalize-line-endings | 0.3.0 | Apache-2.0 | [https://github.com/derekdreery/normalize-line-endings](https://github.com/derekdreery/normalize-line-endings) |
| num-bigint | 0.4.6 | Apache-2.0 OR MIT | [https://github.com/rust-num/num-bigint](https://github.com/rust-num/num-bigint) |
| num-integer | 0.1.46 | Apache-2.0 OR MIT | [https://github.com/rust-num/num-integer](https://github.com/rust-num/num-integer) |
| num-traits | 0.2.19 | Apache-2.0 OR MIT | [https://github.com/rust-num/num-traits](https://github.com/rust-num/num-traits) |
| once_cell | 1.21.4 | Apache-2.0 OR MIT | [https://github.com/matklad/once_cell](https://github.com/matklad/once_cell) |
| once_cell_polyfill | 1.70.2 | Apache-2.0 OR MIT | [https://github.com/polyfill-rs/once_cell_polyfill](https://github.com/polyfill-rs/once_cell_polyfill) |
| option-ext | 0.2.0 | MPL-2.0 | [https://github.com/soc/option-ext.git](https://github.com/soc/option-ext.git) |
| outref | 0.1.0 | MIT | [https://github.com/Nugine/outref](https://github.com/Nugine/outref) |
| outref | 0.5.2 | MIT | [https://github.com/Nugine/outref](https://github.com/Nugine/outref) |
| owo-colors | 4.3.0 | MIT | [https://github.com/owo-colors/owo-colors](https://github.com/owo-colors/owo-colors) |
| oxc-browserslist | 2.3.1 | MIT | [https://github.com/oxc-project/oxc-browserslist](https://github.com/oxc-project/oxc-browserslist) |
| oxc-miette | 2.7.1 | Apache-2.0 | [https://github.com/oxc-project/oxc-miette](https://github.com/oxc-project/oxc-miette) |
| oxc-miette-derive | 2.7.1 | Apache-2.0 | [https://github.com/oxc-project/oxc-miette](https://github.com/oxc-project/oxc-miette) |
| oxc_allocator | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_ast | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_ast_visit | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_codegen | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_compat | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_data_structures | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_diagnostics | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_ecmascript | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_estree | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_index | 4.1.0 | MIT | [https://github.com/oxc-project/oxc-index-vec](https://github.com/oxc-project/oxc-index-vec) |
| oxc_mangler | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_minifier | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_parser | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_regular_expression | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_semantic | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_sourcemap | 6.1.1 | MIT | [https://github.com/oxc-project/oxc-sourcemap](https://github.com/oxc-project/oxc-sourcemap) |
| oxc_span | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_syntax | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_traverse | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| parcel_selectors | 0.28.2 | MPL-2.0 | [https://github.com/parcel-bundler/lightningcss](https://github.com/parcel-bundler/lightningcss) |
| parcel_sourcemap | 2.1.1 | MIT | [https://github.com/parcel-bundler/source-map](https://github.com/parcel-bundler/source-map) |
| parking | 2.2.1 | Apache-2.0 OR MIT | [https://github.com/smol-rs/parking](https://github.com/smol-rs/parking) |
| parking_lot_core | 0.9.12 | Apache-2.0 OR MIT | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| pathdiff | 0.2.3 | Apache-2.0 OR MIT | [https://github.com/Manishearth/pathdiff](https://github.com/Manishearth/pathdiff) |
| percent-encoding | 2.3.2 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| pico-args | 0.5.0 | MIT | [https://github.com/RazrFalcon/pico-args](https://github.com/RazrFalcon/pico-args) |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | [https://github.com/taiki-e/pin-project-lite](https://github.com/taiki-e/pin-project-lite) |
| pin-utils | 0.1.0 | Apache-2.0 OR MIT | [https://github.com/rust-lang-nursery/pin-utils](https://github.com/rust-lang-nursery/pin-utils) |
| piper | 0.2.5 | Apache-2.0 OR MIT | [https://github.com/smol-rs/piper](https://github.com/smol-rs/piper) |
| png | 0.17.16 | Apache-2.0 OR MIT | [https://github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| polling | 3.11.0 | Apache-2.0 OR MIT | [https://github.com/smol-rs/polling](https://github.com/smol-rs/polling) |
| postcard | 1.1.3 | Apache-2.0 OR MIT | [https://github.com/jamesmunns/postcard](https://github.com/jamesmunns/postcard) |
| potential_utf | 0.1.5 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| ppv-lite86 | 0.2.21 | Apache-2.0 OR MIT | [https://github.com/cryptocorrosion/cryptocorrosion](https://github.com/cryptocorrosion/cryptocorrosion) |
| precomputed-hash | 0.1.1 | MIT | [https://github.com/emilio/precomputed-hash](https://github.com/emilio/precomputed-hash) |
| predicates | 3.1.4 | Apache-2.0 OR MIT | [https://github.com/assert-rs/predicates-rs](https://github.com/assert-rs/predicates-rs) |
| predicates-core | 1.0.10 | Apache-2.0 OR MIT | [https://github.com/assert-rs/predicates-rs](https://github.com/assert-rs/predicates-rs) |
| predicates-tree | 1.0.13 | Apache-2.0 OR MIT | [https://github.com/assert-rs/predicates-rs](https://github.com/assert-rs/predicates-rs) |
| ptr_meta | 0.1.4 | MIT | [https://github.com/djkoloski/ptr_meta](https://github.com/djkoloski/ptr_meta) |
| quinn | 0.11.9 | Apache-2.0 OR MIT | [https://github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| quinn-proto | 0.11.14 | Apache-2.0 OR MIT | [https://github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| quinn-udp | 0.5.14 | Apache-2.0 OR MIT | [https://github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| r-efi | 5.3.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT | [https://github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| radium | 0.7.0 | MIT | [https://github.com/bitvecto-rs/radium](https://github.com/bitvecto-rs/radium) |
| rand | 0.8.6 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| rand | 0.9.4 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| rand_chacha | 0.9.0 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| rand_core | 0.6.4 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| rand_core | 0.9.5 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| rayon | 1.12.0 | Apache-2.0 OR MIT | [https://github.com/rayon-rs/rayon](https://github.com/rayon-rs/rayon) |
| rayon-core | 1.13.0 | Apache-2.0 OR MIT | [https://github.com/rayon-rs/rayon](https://github.com/rayon-rs/rayon) |
| redox_syscall | 0.5.18 | MIT | [https://gitlab.redox-os.org/redox-os/syscall](https://gitlab.redox-os.org/redox-os/syscall) |
| redox_users | 0.4.6 | MIT | [https://gitlab.redox-os.org/redox-os/users](https://gitlab.redox-os.org/redox-os/users) |
| regex-automata | 0.4.14 | Apache-2.0 OR MIT | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| regex-syntax | 0.8.10 | Apache-2.0 OR MIT | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| rend | 0.4.2 | MIT | [https://github.com/djkoloski/rend](https://github.com/djkoloski/rend) |
| reqwest | 0.12.28 | Apache-2.0 OR MIT | [https://github.com/seanmonstar/reqwest](https://github.com/seanmonstar/reqwest) |
| resvg | 0.42.0 | MPL-2.0 | [https://github.com/RazrFalcon/resvg](https://github.com/RazrFalcon/resvg) |
| rgb | 0.8.53 | MIT | [https://github.com/kornelski/rust-rgb](https://github.com/kornelski/rust-rgb) |
| ring | 0.17.14 | Apache-2.0 AND ISC | [https://github.com/briansmith/ring](https://github.com/briansmith/ring) |
| rkyv | 0.7.46 | MIT | [https://github.com/rkyv/rkyv](https://github.com/rkyv/rkyv) |
| roxmltree | 0.20.0 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/roxmltree](https://github.com/RazrFalcon/roxmltree) |
| rustc-hash | 2.1.2 | Apache-2.0 OR MIT | [https://github.com/rust-lang/rustc-hash](https://github.com/rust-lang/rustc-hash) |
| rustix | 1.1.4 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| rustls | 0.23.40 | Apache-2.0 OR ISC OR MIT | [https://github.com/rustls/rustls](https://github.com/rustls/rustls) |
| rustls-pki-types | 1.14.1 | Apache-2.0 OR MIT | [https://github.com/rustls/pki-types](https://github.com/rustls/pki-types) |
| rustls-webpki | 0.103.13 | ISC | [https://github.com/rustls/webpki](https://github.com/rustls/webpki) |
| rustybuzz | 0.14.1 | MIT | [https://github.com/RazrFalcon/rustybuzz](https://github.com/RazrFalcon/rustybuzz) |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | [https://github.com/dtolnay/ryu](https://github.com/dtolnay/ryu) |
| same-file | 1.0.6 | MIT OR Unlicense | [https://github.com/BurntSushi/same-file](https://github.com/BurntSushi/same-file) |
| scopeguard | 1.2.0 | Apache-2.0 OR MIT | [https://github.com/bluss/scopeguard](https://github.com/bluss/scopeguard) |
| seahash | 4.1.0 | MIT | [https://gitlab.redox-os.org/redox-os/seahash](https://gitlab.redox-os.org/redox-os/seahash) |
| self_cell | 1.2.2 | Apache-2.0 OR GPL-2.0 | [https://github.com/Voultapher/self_cell](https://github.com/Voultapher/self_cell) |
| seq-macro | 0.3.6 | Apache-2.0 OR MIT | [https://github.com/dtolnay/seq-macro](https://github.com/dtolnay/seq-macro) |
| serde-content | 0.1.2 | Apache-2.0 OR MIT | [https://github.com/rushmorem/serde-content](https://github.com/rushmorem/serde-content) |
| serde_core | 1.0.228 | Apache-2.0 OR MIT | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| serde_regex | 1.2.0 | Apache-2.0 OR MIT |  |
| serde_spanned | 1.1.1 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| serde_urlencoded | 0.7.1 | Apache-2.0 OR MIT | [https://github.com/nox/serde_urlencoded](https://github.com/nox/serde_urlencoded) |
| sha2 | 0.10.9 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| signal-hook-registry | 1.4.8 | Apache-2.0 OR MIT | [https://github.com/vorner/signal-hook](https://github.com/vorner/signal-hook) |
| simd-abstraction | 0.7.1 | MIT | [https://github.com/Nugine/simd](https://github.com/Nugine/simd) |
| simd-adler32 | 0.3.9 | MIT | [https://github.com/mcountryman/simd-adler32](https://github.com/mcountryman/simd-adler32) |
| simdutf8 | 0.1.5 | Apache-2.0 OR MIT | [https://github.com/rusticstuff/simdutf8](https://github.com/rusticstuff/simdutf8) |
| similar | 2.7.0 | Apache-2.0 | [https://github.com/mitsuhiko/similar](https://github.com/mitsuhiko/similar) |
| simplecss | 0.2.2 | Apache-2.0 OR MIT | [https://github.com/linebender/simplecss](https://github.com/linebender/simplecss) |
| siphasher | 1.0.3 | Apache-2.0 OR MIT | [https://github.com/jedisct1/rust-siphash](https://github.com/jedisct1/rust-siphash) |
| slab | 0.4.12 | MIT | [https://github.com/tokio-rs/slab](https://github.com/tokio-rs/slab) |
| slotmap | 1.1.1 | Zlib | [https://github.com/orlp/slotmap](https://github.com/orlp/slotmap) |
| smallvec | 1.15.1 | Apache-2.0 OR MIT | [https://github.com/servo/rust-smallvec](https://github.com/servo/rust-smallvec) |
| smawk | 0.3.3 | MIT | [https://github.com/mgeisler/smawk](https://github.com/mgeisler/smawk) |
| socket2 | 0.5.10 | Apache-2.0 OR MIT | [https://github.com/rust-lang/socket2](https://github.com/rust-lang/socket2) |
| socket2 | 0.6.4 | Apache-2.0 OR MIT | [https://github.com/rust-lang/socket2](https://github.com/rust-lang/socket2) |
| stable_deref_trait | 1.2.1 | Apache-2.0 OR MIT | [https://github.com/storyyeller/stable_deref_trait](https://github.com/storyyeller/stable_deref_trait) |
| strict-num | 0.1.1 | MIT | [https://github.com/RazrFalcon/strict-num](https://github.com/RazrFalcon/strict-num) |
| strip-ansi-escapes | 0.2.1 | Apache-2.0 OR MIT | [https://github.com/luser/strip-ansi-escapes](https://github.com/luser/strip-ansi-escapes) |
| strsim | 0.11.1 | MIT | [https://github.com/rapidfuzz/strsim-rs](https://github.com/rapidfuzz/strsim-rs) |
| subtle | 2.6.1 | BSD-3-Clause | [https://github.com/dalek-cryptography/subtle](https://github.com/dalek-cryptography/subtle) |
| svgtypes | 0.15.3 | Apache-2.0 OR MIT | [https://github.com/linebender/svgtypes](https://github.com/linebender/svgtypes) |
| sync_wrapper | 1.0.2 | Apache-2.0 | [https://github.com/Actyx/sync_wrapper](https://github.com/Actyx/sync_wrapper) |
| tap | 1.0.1 | MIT | [https://github.com/myrrlyn/tap](https://github.com/myrrlyn/tap) |
| tempfile | 3.27.0 | Apache-2.0 OR MIT | [https://github.com/Stebalien/tempfile](https://github.com/Stebalien/tempfile) |
| termtree | 0.5.1 | MIT | [https://github.com/rust-cli/termtree](https://github.com/rust-cli/termtree) |
| textwrap | 0.16.2 | MIT | [https://github.com/mgeisler/textwrap](https://github.com/mgeisler/textwrap) |
| thiserror-impl | 1.0.69 | Apache-2.0 OR MIT | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| thiserror-impl | 2.0.18 | Apache-2.0 OR MIT | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| tiny-skia | 0.11.4 | BSD-3-Clause | [https://github.com/RazrFalcon/tiny-skia](https://github.com/RazrFalcon/tiny-skia) |
| tiny-skia-path | 0.11.4 | BSD-3-Clause | [https://github.com/RazrFalcon/tiny-skia/tree/master/path](https://github.com/RazrFalcon/tiny-skia/tree/master/path) |
| tinystr | 0.8.3 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| tinyvec | 1.11.0 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Lokathor/tinyvec](https://github.com/Lokathor/tinyvec) |
| tinyvec_macros | 0.1.1 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Soveu/tinyvec_macros](https://github.com/Soveu/tinyvec_macros) |
| tokio-macros | 2.7.0 | MIT | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| tokio-rustls | 0.26.4 | Apache-2.0 OR MIT | [https://github.com/rustls/tokio-rustls](https://github.com/rustls/tokio-rustls) |
| toml | 1.1.2+spec-1.1.0 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_datetime | 1.1.1+spec-1.1.0 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_parser | 1.1.2+spec-1.1.0 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_writer | 1.1.1+spec-1.1.0 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| tower | 0.5.3 | MIT | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tower-http | 0.6.11 | MIT | [https://github.com/tower-rs/tower-http](https://github.com/tower-rs/tower-http) |
| tower-layer | 0.3.3 | MIT | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tower-service | 0.3.3 | MIT | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tracing-core | 0.1.36 | MIT | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| try-lock | 0.2.5 | MIT | [https://github.com/seanmonstar/try-lock](https://github.com/seanmonstar/try-lock) |
| ttf-parser | 0.21.1 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/ttf-parser](https://github.com/RazrFalcon/ttf-parser) |
| typenum | 1.20.1 | Apache-2.0 OR MIT | [https://github.com/paholg/typenum](https://github.com/paholg/typenum) |
| unicode-bidi | 0.3.18 | Apache-2.0 OR MIT | [https://github.com/servo/unicode-bidi](https://github.com/servo/unicode-bidi) |
| unicode-bidi-mirroring | 0.2.0 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/unicode-bidi-mirroring](https://github.com/RazrFalcon/unicode-bidi-mirroring) |
| unicode-ccc | 0.2.0 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/unicode-ccc](https://github.com/RazrFalcon/unicode-ccc) |
| unicode-id-start | 1.4.0 | (Apache-2.0 OR MIT) AND Unicode-3.0 | [https://github.com/Boshen/unicode-id-start](https://github.com/Boshen/unicode-id-start) |
| unicode-ident | 1.0.24 | (Apache-2.0 OR MIT) AND Unicode-3.0 | [https://github.com/dtolnay/unicode-ident](https://github.com/dtolnay/unicode-ident) |
| unicode-linebreak | 0.1.5 | Apache-2.0 | [https://github.com/axelf4/unicode-linebreak](https://github.com/axelf4/unicode-linebreak) |
| unicode-properties | 0.1.4 | Apache-2.0 OR MIT | [https://github.com/unicode-rs/unicode-properties](https://github.com/unicode-rs/unicode-properties) |
| unicode-script | 0.5.8 | Apache-2.0 OR MIT | [https://github.com/unicode-rs/unicode-script](https://github.com/unicode-rs/unicode-script) |
| unicode-segmentation | 1.13.3 | Apache-2.0 OR MIT | [https://github.com/unicode-rs/unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation) |
| unicode-vo | 0.1.0 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/unicode-vo](https://github.com/RazrFalcon/unicode-vo) |
| unicode-width | 0.2.2 | Apache-2.0 OR MIT | [https://github.com/unicode-rs/unicode-width](https://github.com/unicode-rs/unicode-width) |
| untrusted | 0.9.0 | ISC | [https://github.com/briansmith/untrusted](https://github.com/briansmith/untrusted) |
| usvg | 0.42.0 | MPL-2.0 | [https://github.com/RazrFalcon/resvg](https://github.com/RazrFalcon/resvg) |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT | [https://github.com/hsivonen/utf8_iter](https://github.com/hsivonen/utf8_iter) |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | [https://github.com/alacritty/vte](https://github.com/alacritty/vte) |
| uuid | 1.23.4 | Apache-2.0 OR MIT | [https://github.com/uuid-rs/uuid](https://github.com/uuid-rs/uuid) |
| value-bag | 1.12.0 | Apache-2.0 OR MIT | [https://github.com/sval-rs/value-bag](https://github.com/sval-rs/value-bag) |
| vlq | 0.5.1 | Apache-2.0 OR MIT | [https://github.com/tromey/vlq](https://github.com/tromey/vlq) |
| vsimd | 0.8.0 | MIT | [https://github.com/Nugine/simd](https://github.com/Nugine/simd) |
| vte | 0.14.1 | Apache-2.0 OR MIT | [https://github.com/alacritty/vte](https://github.com/alacritty/vte) |
| wait-timeout | 0.2.1 | Apache-2.0 OR MIT | [https://github.com/alexcrichton/wait-timeout](https://github.com/alexcrichton/wait-timeout) |
| walkdir | 2.5.0 | MIT OR Unlicense | [https://github.com/BurntSushi/walkdir](https://github.com/BurntSushi/walkdir) |
| want | 0.3.1 | MIT | [https://github.com/seanmonstar/want](https://github.com/seanmonstar/want) |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/bytecodealliance/wasi](https://github.com/bytecodealliance/wasi) |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/bytecodealliance/wasi-rs](https://github.com/bytecodealliance/wasi-rs) |
| wasm-bindgen | 0.2.121 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) |
| wasm-bindgen-futures | 0.4.71 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures) |
| wasm-bindgen-macro | 0.2.121 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| wasm-bindgen-shared | 0.2.121 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| web-sys | 0.3.98 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| web-time | 1.1.0 | Apache-2.0 OR MIT | [https://github.com/daxpedda/web-time](https://github.com/daxpedda/web-time) |
| webpki-roots | 0.26.11 | CDLA-Permissive-2.0 | [https://github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| webpki-roots | 1.0.7 | CDLA-Permissive-2.0 | [https://github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| weezl | 0.1.12 | Apache-2.0 OR MIT | [https://github.com/image-rs/weezl](https://github.com/image-rs/weezl) |
| winapi-util | 0.1.11 | MIT OR Unlicense | [https://github.com/BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util) |
| windows-link | 0.2.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.48.0 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.52.0 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.60.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.61.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.53.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.53.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.53.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.53.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnullvm | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnullvm | 0.53.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.53.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.53.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.53.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.53.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| winnow | 1.0.2 | MIT | [https://github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| wit-bindgen | 0.57.1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/bytecodealliance/wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) |
| writeable | 0.6.3 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| wyz | 0.5.1 | MIT | [https://github.com/myrrlyn/wyz](https://github.com/myrrlyn/wyz) |
| xmlwriter | 0.1.0 | MIT | [https://github.com/RazrFalcon/xmlwriter](https://github.com/RazrFalcon/xmlwriter) |
| yoke | 0.8.3 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| yoke-derive | 0.8.2 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerocopy | 0.8.48 | Apache-2.0 OR BSD-2-Clause OR MIT | [https://github.com/google/zerocopy](https://github.com/google/zerocopy) |
| zerocopy-derive | 0.8.48 | Apache-2.0 OR BSD-2-Clause OR MIT | [https://github.com/google/zerocopy](https://github.com/google/zerocopy) |
| zerofrom | 0.1.8 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerofrom-derive | 0.1.7 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zeroize | 1.9.0 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| zerotrie | 0.2.4 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerovec | 0.11.6 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerovec-derive | 0.11.3 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zmij | 1.0.21 | MIT | [https://github.com/dtolnay/zmij](https://github.com/dtolnay/zmij) |
