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
| Apache-2.0 OR MIT | 268 |
| MIT | 125 |
| Unicode-3.0 | 18 |
| Apache-2.0 | 10 |
| MIT OR Unlicense | 9 |
| Apache-2.0 OR MIT OR Zlib | 8 |
| MPL-2.0 | 8 |
| Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | 5 |
| Apache-2.0 OR ISC OR MIT | 3 |
| BSD-3-Clause | 3 |
| CDLA-Permissive-2.0 | 3 |
| (Apache-2.0 OR MIT) AND Unicode-3.0 | 2 |
| Apache-2.0 OR BSD-2-Clause OR MIT | 2 |
| Apache-2.0 OR CC0-1.0 OR MIT-0 | 2 |
| Apache-2.0 OR LGPL-2.1-or-later OR MIT | 2 |
| ISC | 2 |
| (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) AND (Apache-2.0 OR ISC) AND Apache-2.0 AND BSD-3-Clause AND ISC AND MIT | 1 |
| (Apache-2.0 OR ISC) AND ISC | 1 |
| 0BSD OR Apache-2.0 OR MIT | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR CC0-1.0 | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR GPL-2.0 | 1 |
| Apache-2.0 WITH LLVM-exception OR BSL-1.0 | 1 |
| BSD-2-Clause | 1 |
| Zlib | 1 |

**Total: 480 dependencies**

## All dependencies

| Crate | Version | License | Repository |
|---|---|---|---|
| adler2 | 2.0.1 | 0BSD OR Apache-2.0 OR MIT | [https://github.com/oyvindln/adler2](https://github.com/oyvindln/adler2) |
| ahash | 0.7.8 | Apache-2.0 OR MIT | [https://github.com/tkaitchuck/ahash](https://github.com/tkaitchuck/ahash) |
| ahash | 0.8.12 | Apache-2.0 OR MIT | [https://github.com/tkaitchuck/ahash](https://github.com/tkaitchuck/ahash) |
| aho-corasick | 1.1.5 | MIT OR Unlicense | [https://github.com/BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick) |
| alloca | 0.4.0 | MIT | [https://github.com/playXE/alloca-rs](https://github.com/playXE/alloca-rs) |
| allocator-api2 | 0.2.21 | Apache-2.0 OR MIT | [https://github.com/zakarumych/allocator-api2](https://github.com/zakarumych/allocator-api2) |
| anes | 0.1.6 | Apache-2.0 OR MIT | [https://github.com/zrzka/anes-rs](https://github.com/zrzka/anes-rs) |
| anstream | 1.0.0 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle | 1.0.14 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-parse | 1.0.0 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-query | 1.1.5 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-wincon | 3.0.11 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| arrayref | 0.3.9 | BSD-2-Clause | [https://github.com/droundy/arrayref](https://github.com/droundy/arrayref) |
| arrayvec | 0.7.8 | Apache-2.0 OR MIT | [https://github.com/bluss/arrayvec](https://github.com/bluss/arrayvec) |
| assert-json-diff | 2.0.2 | MIT | [https://github.com/davidpdrsn/assert-json-diff.git](https://github.com/davidpdrsn/assert-json-diff.git) |
| assert_cmd | 2.2.2 | Apache-2.0 OR MIT | [https://github.com/assert-rs/assert_cmd.git](https://github.com/assert-rs/assert_cmd.git) |
| assert_fs | 1.1.4 | Apache-2.0 OR MIT | [https://github.com/assert-rs/assert_fs.git](https://github.com/assert-rs/assert_fs.git) |
| async-lock | 3.4.2 | Apache-2.0 OR MIT | [https://github.com/smol-rs/async-lock](https://github.com/smol-rs/async-lock) |
| async-object-pool | 0.2.0 | MIT | [https://github.com/alexliesenfeld/async-object-pool](https://github.com/alexliesenfeld/async-object-pool) |
| async-trait | 0.1.92 | Apache-2.0 OR MIT | [https://github.com/dtolnay/async-trait](https://github.com/dtolnay/async-trait) |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | [https://github.com/smol-rs/atomic-waker](https://github.com/smol-rs/atomic-waker) |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | [https://github.com/cuviper/autocfg](https://github.com/cuviper/autocfg) |
| aws-lc-rs | 1.18.0 | (Apache-2.0 OR ISC) AND ISC | [https://github.com/aws/aws-lc-rs](https://github.com/aws/aws-lc-rs) |
| aws-lc-sys | 0.44.0 | (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) AND (Apache-2.0 OR ISC) AND Apache-2.0 AND BSD-3-Clause AND ISC AND MIT | [https://github.com/aws/aws-lc-rs](https://github.com/aws/aws-lc-rs) |
| base64 | 0.22.1 | Apache-2.0 OR MIT | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| base64 | 0.23.1 | Apache-2.0 OR MIT | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| base64-simd | 0.7.0 | MIT | [https://github.com/Nugine/simd](https://github.com/Nugine/simd) |
| base64-simd | 0.8.0 | MIT | [https://github.com/Nugine/simd](https://github.com/Nugine/simd) |
| bitflags | 2.13.1 | Apache-2.0 OR MIT | [https://github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| bitvec | 1.1.1 | MIT | [https://github.com/bitvecto-rs/bitvec](https://github.com/bitvecto-rs/bitvec) |
| blake3 | 1.8.6 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR CC0-1.0 | [https://github.com/BLAKE3-team/BLAKE3](https://github.com/BLAKE3-team/BLAKE3) |
| block-buffer | 0.10.4 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| block-buffer | 0.12.1 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| bstr | 1.13.1 | Apache-2.0 OR MIT | [https://github.com/BurntSushi/bstr](https://github.com/BurntSushi/bstr) |
| bumpalo | 3.19.0 | Apache-2.0 OR MIT | [https://github.com/fitzgen/bumpalo](https://github.com/fitzgen/bumpalo) |
| bytecheck | 0.6.12 | MIT | [https://github.com/djkoloski/bytecheck](https://github.com/djkoloski/bytecheck) |
| bytecheck_derive | 0.6.12 | MIT | [https://github.com/djkoloski/bytecheck](https://github.com/djkoloski/bytecheck) |
| bytemuck | 1.25.2 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Lokathor/bytemuck](https://github.com/Lokathor/bytemuck) |
| bytemuck_derive | 1.12.0 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Lokathor/bytemuck](https://github.com/Lokathor/bytemuck) |
| byteorder-lite | 0.1.0 | MIT OR Unlicense | [https://github.com/image-rs/byteorder-lite](https://github.com/image-rs/byteorder-lite) |
| bytes | 1.12.1 | MIT | [https://github.com/tokio-rs/bytes](https://github.com/tokio-rs/bytes) |
| cast | 0.3.0 | Apache-2.0 OR MIT | [https://github.com/japaric/cast.rs](https://github.com/japaric/cast.rs) |
| castaway | 0.2.4 | MIT | [https://github.com/sagebind/castaway](https://github.com/sagebind/castaway) |
| cc | 1.4.3 | Apache-2.0 OR MIT | [https://github.com/rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| cesu8 | 1.1.0 | Apache-2.0 OR MIT | [https://github.com/emk/cesu8-rs](https://github.com/emk/cesu8-rs) |
| cfg-if | 1.0.4 | Apache-2.0 OR MIT | [https://github.com/rust-lang/cfg-if](https://github.com/rust-lang/cfg-if) |
| cfg_aliases | 0.2.2 | MIT | [https://github.com/katharostech/cfg_aliases](https://github.com/katharostech/cfg_aliases) |
| chacha20 | 0.10.1 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/stream-ciphers](https://github.com/RustCrypto/stream-ciphers) |
| ciborium | 0.2.2 | Apache-2.0 | [https://github.com/enarx/ciborium](https://github.com/enarx/ciborium) |
| ciborium-io | 0.2.2 | Apache-2.0 | [https://github.com/enarx/ciborium](https://github.com/enarx/ciborium) |
| ciborium-ll | 0.2.2 | Apache-2.0 | [https://github.com/enarx/ciborium](https://github.com/enarx/ciborium) |
| clap | 4.6.6 | Apache-2.0 OR MIT | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_builder | 4.6.6 | Apache-2.0 OR MIT | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_derive | 4.6.4 | Apache-2.0 OR MIT | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_lex | 1.1.0 | Apache-2.0 OR MIT | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| cmake | 0.1.58 | Apache-2.0 OR MIT | [https://github.com/rust-lang/cmake-rs](https://github.com/rust-lang/cmake-rs) |
| cobs | 0.3.0 | Apache-2.0 OR MIT | [https://github.com/jamesmunns/cobs.rs](https://github.com/jamesmunns/cobs.rs) |
| color_quant | 1.1.0 | MIT | [https://github.com/image-rs/color_quant.git](https://github.com/image-rs/color_quant.git) |
| colorchoice | 1.0.5 | Apache-2.0 OR MIT | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| combine | 4.6.7 | MIT | [https://github.com/Marwes/combine](https://github.com/Marwes/combine) |
| compact_str | 0.9.1 | MIT | [https://github.com/ParkMyCar/compact_str](https://github.com/ParkMyCar/compact_str) |
| console | 0.16.4 | MIT | [https://github.com/console-rs/console](https://github.com/console-rs/console) |
| const-oid | 0.10.2 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/formats](https://github.com/RustCrypto/formats) |
| const-str | 1.1.0 | MIT | [https://github.com/Nugine/const-str](https://github.com/Nugine/const-str) |
| constant_time_eq | 0.4.2 | Apache-2.0 OR CC0-1.0 OR MIT-0 | [https://github.com/cesarb/constant_time_eq](https://github.com/cesarb/constant_time_eq) |
| convert_case | 0.6.0 | MIT | [https://github.com/rutrum/convert-case](https://github.com/rutrum/convert-case) |
| core-foundation | 0.10.1 | Apache-2.0 OR MIT | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| core-foundation-sys | 0.8.7 | Apache-2.0 OR MIT | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| cow-utils | 0.1.3 | MIT | [https://github.com/RReverser/cow-utils-rs](https://github.com/RReverser/cow-utils-rs) |
| cpufeatures | 0.2.17 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| cpufeatures | 0.3.0 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| crc32fast | 1.5.0 | Apache-2.0 OR MIT | [https://github.com/srijs/rust-crc32fast](https://github.com/srijs/rust-crc32fast) |
| criterion | 0.8.2 | Apache-2.0 OR MIT | [https://github.com/criterion-rs/criterion.rs](https://github.com/criterion-rs/criterion.rs) |
| criterion-plot | 0.8.2 | Apache-2.0 OR MIT | [https://github.com/criterion-rs/criterion.rs](https://github.com/criterion-rs/criterion.rs) |
| crossbeam-deque | 0.8.7 | Apache-2.0 OR MIT | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| crossbeam-epoch | 0.9.20 | Apache-2.0 OR MIT | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| crossbeam-utils | 0.8.22 | Apache-2.0 OR MIT | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| crunchy | 0.2.4 | MIT | [https://github.com/eira-fransham/crunchy](https://github.com/eira-fransham/crunchy) |
| crypto-common | 0.1.7 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| crypto-common | 0.2.2 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| cssparser | 0.37.0 | MPL-2.0 | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| cssparser-color | 0.5.0 | MPL-2.0 | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| cssparser-macros | 0.7.0 | MPL-2.0 | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| dashmap | 5.5.3 | MIT | [https://github.com/xacrimon/dashmap](https://github.com/xacrimon/dashmap) |
| data-encoding | 2.11.1 | MIT | [https://github.com/ia0/data-encoding](https://github.com/ia0/data-encoding) |
| data-url | 0.1.1 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| data-url | 0.3.2 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| difflib | 0.4.0 | MIT | [https://github.com/DimaKudosh/difflib](https://github.com/DimaKudosh/difflib) |
| digest | 0.10.7 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| digest | 0.11.3 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| directories | 5.0.1 | Apache-2.0 OR MIT | [https://github.com/soc/directories-rs](https://github.com/soc/directories-rs) |
| directories | 6.0.0 | Apache-2.0 OR MIT | [https://github.com/soc/directories-rs](https://github.com/soc/directories-rs) |
| dirs-sys | 0.4.1 | Apache-2.0 OR MIT | [https://github.com/dirs-dev/dirs-sys-rs](https://github.com/dirs-dev/dirs-sys-rs) |
| dirs-sys | 0.5.0 | Apache-2.0 OR MIT | [https://github.com/dirs-dev/dirs-sys-rs](https://github.com/dirs-dev/dirs-sys-rs) |
| displaydoc | 0.2.7 | Apache-2.0 OR MIT | [https://github.com/yaahc/displaydoc](https://github.com/yaahc/displaydoc) |
| dragonbox_ecma | 0.0.5 | Apache-2.0 WITH LLVM-exception OR BSL-1.0 | [https://github.com/magic-akari/dragonbox](https://github.com/magic-akari/dragonbox) |
| dtoa | 1.0.11 | Apache-2.0 OR MIT | [https://github.com/dtolnay/dtoa](https://github.com/dtolnay/dtoa) |
| dtoa-short | 0.3.5 | MPL-2.0 | [https://github.com/upsuper/dtoa-short](https://github.com/upsuper/dtoa-short) |
| dunce | 1.0.5 | Apache-2.0 OR CC0-1.0 OR MIT-0 | [https://gitlab.com/kornelski/dunce](https://gitlab.com/kornelski/dunce) |
| either | 1.17.0 | Apache-2.0 OR MIT | [https://github.com/rayon-rs/either](https://github.com/rayon-rs/either) |
| embedded-io | 0.4.0 | Apache-2.0 OR MIT | [https://github.com/embassy-rs/embedded-io](https://github.com/embassy-rs/embedded-io) |
| embedded-io | 0.6.1 | Apache-2.0 OR MIT | [https://github.com/rust-embedded/embedded-hal](https://github.com/rust-embedded/embedded-hal) |
| encode_unicode | 1.0.0 | Apache-2.0 OR MIT | [https://github.com/tormol/encode_unicode](https://github.com/tormol/encode_unicode) |
| equivalent | 1.0.2 | Apache-2.0 OR MIT | [https://github.com/indexmap-rs/equivalent](https://github.com/indexmap-rs/equivalent) |
| errno | 0.3.14 | Apache-2.0 OR MIT | [https://github.com/lambda-fairy/rust-errno](https://github.com/lambda-fairy/rust-errno) |
| escargot | 0.5.15 | Apache-2.0 OR MIT | [https://github.com/crate-ci/escargot.git](https://github.com/crate-ci/escargot.git) |
| euclid | 0.22.14 | Apache-2.0 OR MIT | [https://github.com/servo/euclid](https://github.com/servo/euclid) |
| event-listener | 5.4.2 | Apache-2.0 OR MIT | [https://github.com/smol-rs/event-listener](https://github.com/smol-rs/event-listener) |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | [https://github.com/smol-rs/event-listener-strategy](https://github.com/smol-rs/event-listener-strategy) |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | [https://github.com/smol-rs/fastrand](https://github.com/smol-rs/fastrand) |
| fdeflate | 0.3.7 | Apache-2.0 OR MIT | [https://github.com/image-rs/fdeflate](https://github.com/image-rs/fdeflate) |
| find-msvc-tools | 0.1.11 | Apache-2.0 OR MIT | [https://github.com/rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| flate2 | 1.1.9 | Apache-2.0 OR MIT | [https://github.com/rust-lang/flate2-rs](https://github.com/rust-lang/flate2-rs) |
| float-cmp | 0.10.0 | MIT | [https://github.com/mikedilger/float-cmp](https://github.com/mikedilger/float-cmp) |
| float-cmp | 0.9.0 | MIT | [https://github.com/mikedilger/float-cmp](https://github.com/mikedilger/float-cmp) |
| fnv | 1.0.7 | Apache-2.0 OR MIT | [https://github.com/servo/rust-fnv](https://github.com/servo/rust-fnv) |
| font-types | 0.12.3 | Apache-2.0 OR MIT | [https://github.com/googlefonts/fontations](https://github.com/googlefonts/fontations) |
| fontconfig-parser | 0.5.8 | MIT | [https://github.com/Riey/fontconfig-parser](https://github.com/Riey/fontconfig-parser) |
| fontdb | 0.24.0 | MIT | [https://github.com/RazrFalcon/fontdb](https://github.com/RazrFalcon/fontdb) |
| form_urlencoded | 1.2.2 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| fs_extra | 1.3.0 | MIT | [https://github.com/webdesus/fs_extra](https://github.com/webdesus/fs_extra) |
| funty | 2.0.0 | MIT | [https://github.com/myrrlyn/funty](https://github.com/myrrlyn/funty) |
| futures-channel | 0.3.34 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-core | 0.3.34 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-io | 0.3.34 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-macro | 0.3.34 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-sink | 0.3.34 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-task | 0.3.34 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-timer | 3.0.4 | Apache-2.0 OR MIT | [https://github.com/async-rs/futures-timer](https://github.com/async-rs/futures-timer) |
| futures-util | 0.3.34 | Apache-2.0 OR MIT | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| generic-array | 0.14.7 | MIT | [https://github.com/fizyk20/generic-array.git](https://github.com/fizyk20/generic-array.git) |
| getrandom | 0.2.17 | Apache-2.0 OR MIT | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| getrandom | 0.3.4 | Apache-2.0 OR MIT | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| getrandom | 0.4.3 | Apache-2.0 OR MIT | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| gif | 0.14.2 | Apache-2.0 OR MIT | [https://github.com/image-rs/image-gif](https://github.com/image-rs/image-gif) |
| globset | 0.4.20 | MIT OR Unlicense | [https://github.com/BurntSushi/ripgrep/tree/master/crates/globset](https://github.com/BurntSushi/ripgrep/tree/master/crates/globset) |
| globwalk | 0.9.1 | MIT | [https://github.com/gilnaa/globwalk](https://github.com/gilnaa/globwalk) |
| h2 | 0.4.15 | MIT | [https://github.com/hyperium/h2](https://github.com/hyperium/h2) |
| half | 2.7.1 | Apache-2.0 OR MIT | [https://github.com/VoidStarKat/half-rs](https://github.com/VoidStarKat/half-rs) |
| harfrust | 0.12.0 | MIT | [https://github.com/harfbuzz/harfrust](https://github.com/harfbuzz/harfrust) |
| hashbrown | 0.12.3 | Apache-2.0 OR MIT | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.14.5 | Apache-2.0 OR MIT | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.16.1 | Apache-2.0 OR MIT | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.17.1 | Apache-2.0 OR MIT | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| headers | 0.4.1 | MIT | [https://github.com/hyperium/headers](https://github.com/hyperium/headers) |
| headers-core | 0.3.0 | MIT | [https://github.com/hyperium/headers](https://github.com/hyperium/headers) |
| heck | 0.5.0 | Apache-2.0 OR MIT | [https://github.com/withoutboats/heck](https://github.com/withoutboats/heck) |
| hex | 0.4.3 | Apache-2.0 OR MIT | [https://github.com/KokaKiwi/rust-hex](https://github.com/KokaKiwi/rust-hex) |
| http | 1.5.0 | Apache-2.0 OR MIT | [https://github.com/hyperium/http](https://github.com/hyperium/http) |
| http-body | 1.1.0 | MIT | [https://github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| http-body-util | 0.1.5 | MIT | [https://github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| httparse | 1.10.1 | Apache-2.0 OR MIT | [https://github.com/seanmonstar/httparse](https://github.com/seanmonstar/httparse) |
| httpdate | 1.0.3 | Apache-2.0 OR MIT | [https://github.com/pyfisch/httpdate](https://github.com/pyfisch/httpdate) |
| httpmock | 0.8.3 | MIT | [https://github.com/httpmock/httpmock](https://github.com/httpmock/httpmock) |
| hybrid-array | 0.4.14 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/hybrid-array](https://github.com/RustCrypto/hybrid-array) |
| hyper | 1.11.0 | MIT | [https://github.com/hyperium/hyper](https://github.com/hyperium/hyper) |
| hyper-rustls | 0.27.9 | Apache-2.0 OR ISC OR MIT | [https://github.com/rustls/hyper-rustls](https://github.com/rustls/hyper-rustls) |
| hyper-util | 0.1.20 | MIT | [https://github.com/hyperium/hyper-util](https://github.com/hyperium/hyper-util) |
| iconify | 0.3.1 | Apache-2.0 OR MIT | [https://github.com/wrapperup/iconify-rs](https://github.com/wrapperup/iconify-rs) |
| icu_collections | 2.3.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_locale_core | 2.3.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_normalizer | 2.3.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_normalizer_data | 2.3.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_properties | 2.3.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_properties_data | 2.3.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_provider | 2.3.0 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| idna | 1.1.0 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | [https://github.com/hsivonen/idna_adapter](https://github.com/hsivonen/idna_adapter) |
| ignore | 0.4.33 | MIT OR Unlicense | [https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) |
| image-webp | 0.2.4 | Apache-2.0 OR MIT | [https://github.com/image-rs/image-webp](https://github.com/image-rs/image-webp) |
| imagesize | 0.15.0 | MIT | [https://github.com/Roughsketch/imagesize](https://github.com/Roughsketch/imagesize) |
| indexmap | 2.14.0 | Apache-2.0 OR MIT | [https://github.com/indexmap-rs/indexmap](https://github.com/indexmap-rs/indexmap) |
| insta | 1.48.0 | Apache-2.0 | [https://github.com/mitsuhiko/insta](https://github.com/mitsuhiko/insta) |
| ipnet | 2.12.1 | Apache-2.0 OR MIT | [https://github.com/krisprice/ipnet](https://github.com/krisprice/ipnet) |
| is_terminal_polyfill | 1.70.2 | Apache-2.0 OR MIT | [https://github.com/polyfill-rs/is_terminal_polyfill](https://github.com/polyfill-rs/is_terminal_polyfill) |
| itertools | 0.10.5 | Apache-2.0 OR MIT | [https://github.com/rust-itertools/itertools](https://github.com/rust-itertools/itertools) |
| itertools | 0.13.0 | Apache-2.0 OR MIT | [https://github.com/rust-itertools/itertools](https://github.com/rust-itertools/itertools) |
| itertools | 0.14.0 | Apache-2.0 OR MIT | [https://github.com/rust-itertools/itertools](https://github.com/rust-itertools/itertools) |
| itoa | 1.0.18 | Apache-2.0 OR MIT | [https://github.com/dtolnay/itoa](https://github.com/dtolnay/itoa) |
| jni | 0.21.1 | Apache-2.0 OR MIT | [https://github.com/jni-rs/jni-rs](https://github.com/jni-rs/jni-rs) |
| jni-sys | 0.3.1 | Apache-2.0 OR MIT | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| jni-sys | 0.4.1 | Apache-2.0 OR MIT | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| jni-sys-macros | 0.4.1 | Apache-2.0 OR MIT | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| jobserver | 0.1.35 | Apache-2.0 OR MIT | [https://github.com/rust-lang/jobserver-rs](https://github.com/rust-lang/jobserver-rs) |
| js-sys | 0.3.104 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| json-escape-simd | 3.1.1 | MIT | [https://github.com/napi-rs/json-escape-simd](https://github.com/napi-rs/json-escape-simd) |
| kurbo | 0.13.1 | Apache-2.0 OR MIT | [https://github.com/linebender/kurbo](https://github.com/linebender/kurbo) |
| lazy_static | 1.5.0 | Apache-2.0 OR MIT | [https://github.com/rust-lang-nursery/lazy-static.rs](https://github.com/rust-lang-nursery/lazy-static.rs) |
| libc | 0.2.189 | Apache-2.0 OR MIT | [https://github.com/rust-lang/libc](https://github.com/rust-lang/libc) |
| libredox | 0.1.20 | MIT | [https://gitlab.redox-os.org/redox-os/libredox.git](https://gitlab.redox-os.org/redox-os/libredox.git) |
| lightningcss | 1.0.0-alpha.72 | MPL-2.0 | [https://github.com/parcel-bundler/lightningcss](https://github.com/parcel-bundler/lightningcss) |
| lightningcss-derive | 1.0.0-alpha.43 | MPL-2.0 | [https://github.com/parcel-bundler/lightningcss](https://github.com/parcel-bundler/lightningcss) |
| linux-raw-sys | 0.12.1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/sunfishcode/linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) |
| litemap | 0.8.3 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| lock_api | 0.4.14 | Apache-2.0 OR MIT | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| log | 0.4.33 | Apache-2.0 OR MIT | [https://github.com/rust-lang/log](https://github.com/rust-lang/log) |
| lru-slab | 0.1.2 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Ralith/lru-slab](https://github.com/Ralith/lru-slab) |
| markdown | 1.0.0 | MIT | [https://github.com/wooorm/markdown-rs](https://github.com/wooorm/markdown-rs) |
| matches | 0.1.10 | MIT | [https://github.com/SimonSapin/rust-std-candidates](https://github.com/SimonSapin/rust-std-candidates) |
| memchr | 2.8.3 | MIT OR Unlicense | [https://github.com/BurntSushi/memchr](https://github.com/BurntSushi/memchr) |
| memmap2 | 0.9.11 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/memmap2-rs](https://github.com/RazrFalcon/memmap2-rs) |
| mime | 0.3.17 | Apache-2.0 OR MIT | [https://github.com/hyperium/mime](https://github.com/hyperium/mime) |
| minify-html | 0.18.1 | MIT | [https://github.com/wilsonzlin/minify-html.git](https://github.com/wilsonzlin/minify-html.git) |
| minify-html-common | 0.0.3 | MIT | [https://github.com/wilsonzlin/minify-html.git](https://github.com/wilsonzlin/minify-html.git) |
| miniz_oxide | 0.8.9 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| mio | 1.2.2 | MIT | [https://github.com/tokio-rs/mio](https://github.com/tokio-rs/mio) |
| nonmax | 0.5.5 | Apache-2.0 OR MIT | [https://github.com/LPGhatguy/nonmax](https://github.com/LPGhatguy/nonmax) |
| normalize-line-endings | 0.3.0 | Apache-2.0 | [https://github.com/derekdreery/normalize-line-endings](https://github.com/derekdreery/normalize-line-endings) |
| num-bigint | 0.4.8 | Apache-2.0 OR MIT | [https://github.com/rust-num/num-bigint](https://github.com/rust-num/num-bigint) |
| num-integer | 0.1.47 | Apache-2.0 OR MIT | [https://github.com/rust-num/num-integer](https://github.com/rust-num/num-integer) |
| num-traits | 0.2.19 | Apache-2.0 OR MIT | [https://github.com/rust-num/num-traits](https://github.com/rust-num/num-traits) |
| once_cell | 1.21.4 | Apache-2.0 OR MIT | [https://github.com/matklad/once_cell](https://github.com/matklad/once_cell) |
| once_cell_polyfill | 1.70.2 | Apache-2.0 OR MIT | [https://github.com/polyfill-rs/once_cell_polyfill](https://github.com/polyfill-rs/once_cell_polyfill) |
| oorandom | 11.1.5 | MIT | [https://hg.sr.ht/~icefox/oorandom](https://hg.sr.ht/~icefox/oorandom) |
| openssl-probe | 0.2.1 | Apache-2.0 OR MIT | [https://github.com/rustls/openssl-probe](https://github.com/rustls/openssl-probe) |
| option-ext | 0.2.0 | MPL-2.0 | [https://github.com/soc/option-ext.git](https://github.com/soc/option-ext.git) |
| outref | 0.1.0 | MIT | [https://github.com/Nugine/outref](https://github.com/Nugine/outref) |
| outref | 0.5.2 | MIT | [https://github.com/Nugine/outref](https://github.com/Nugine/outref) |
| owo-colors | 4.3.0 | MIT | [https://github.com/owo-colors/owo-colors](https://github.com/owo-colors/owo-colors) |
| oxc-browserslist | 2.3.1 | MIT | [https://github.com/oxc-project/oxc-browserslist](https://github.com/oxc-project/oxc-browserslist) |
| oxc-miette | 2.7.1 | Apache-2.0 | [https://github.com/oxc-project/oxc-miette](https://github.com/oxc-project/oxc-miette) |
| oxc-miette-derive | 2.7.1 | Apache-2.0 | [https://github.com/oxc-project/oxc-miette](https://github.com/oxc-project/oxc-miette) |
| oxc_allocator | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_ast | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
| oxc_ast_macros | 0.95.0 | MIT | [https://github.com/oxc-project/oxc](https://github.com/oxc-project/oxc) |
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
| page_size | 0.6.0 | Apache-2.0 OR MIT | [https://github.com/Elzair/page_size_rs](https://github.com/Elzair/page_size_rs) |
| parcel_selectors | 0.28.3 | MPL-2.0 | [https://github.com/parcel-bundler/lightningcss](https://github.com/parcel-bundler/lightningcss) |
| parcel_sourcemap | 2.1.1 | MIT | [https://github.com/parcel-bundler/source-map](https://github.com/parcel-bundler/source-map) |
| parking | 2.2.1 | Apache-2.0 OR MIT | [https://github.com/smol-rs/parking](https://github.com/smol-rs/parking) |
| parking_lot_core | 0.9.12 | Apache-2.0 OR MIT | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| pastey | 0.1.1 | Apache-2.0 OR MIT | [https://github.com/as1100k/pastey](https://github.com/as1100k/pastey) |
| path-tree | 0.8.3 | Apache-2.0 OR MIT | [https://github.com/viz-rs/path-tree](https://github.com/viz-rs/path-tree) |
| pathdiff | 0.2.3 | Apache-2.0 OR MIT | [https://github.com/Manishearth/pathdiff](https://github.com/Manishearth/pathdiff) |
| percent-encoding | 2.3.2 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| phf | 0.11.3 | MIT | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf | 0.13.1 | MIT | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_codegen | 0.11.3 | MIT | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_generator | 0.11.3 | MIT | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_generator | 0.13.1 | MIT | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_macros | 0.13.1 | MIT | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_shared | 0.11.3 | MIT | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_shared | 0.13.1 | MIT | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| pico-args | 0.5.0 | MIT | [https://github.com/RazrFalcon/pico-args](https://github.com/RazrFalcon/pico-args) |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | [https://github.com/taiki-e/pin-project-lite](https://github.com/taiki-e/pin-project-lite) |
| pkg-config | 0.3.34 | Apache-2.0 OR MIT | [https://github.com/rust-lang/pkg-config-rs](https://github.com/rust-lang/pkg-config-rs) |
| plotters | 0.3.7 | MIT | [https://github.com/plotters-rs/plotters](https://github.com/plotters-rs/plotters) |
| plotters-backend | 0.3.7 | MIT | [https://github.com/plotters-rs/plotters](https://github.com/plotters-rs/plotters) |
| plotters-svg | 0.3.7 | MIT | [https://github.com/plotters-rs/plotters.git](https://github.com/plotters-rs/plotters.git) |
| png | 0.18.1 | Apache-2.0 OR MIT | [https://github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| polycool | 0.4.0 | Apache-2.0 OR MIT | [https://github.com/linebender/kurbo](https://github.com/linebender/kurbo) |
| postcard | 1.1.3 | Apache-2.0 OR MIT | [https://github.com/jamesmunns/postcard](https://github.com/jamesmunns/postcard) |
| potential_utf | 0.1.6 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| precomputed-hash | 0.1.1 | MIT | [https://github.com/emilio/precomputed-hash](https://github.com/emilio/precomputed-hash) |
| predicates | 3.1.4 | Apache-2.0 OR MIT | [https://github.com/assert-rs/predicates-rs](https://github.com/assert-rs/predicates-rs) |
| predicates-core | 1.0.10 | Apache-2.0 OR MIT | [https://github.com/assert-rs/predicates-rs](https://github.com/assert-rs/predicates-rs) |
| predicates-tree | 1.0.13 | Apache-2.0 OR MIT | [https://github.com/assert-rs/predicates-rs](https://github.com/assert-rs/predicates-rs) |
| proc-macro2 | 1.0.107 | Apache-2.0 OR MIT | [https://github.com/dtolnay/proc-macro2](https://github.com/dtolnay/proc-macro2) |
| ptr_meta | 0.1.4 | MIT | [https://github.com/djkoloski/ptr_meta](https://github.com/djkoloski/ptr_meta) |
| ptr_meta_derive | 0.1.4 | MIT | [https://github.com/djkoloski/ptr_meta](https://github.com/djkoloski/ptr_meta) |
| quick-error | 2.0.1 | Apache-2.0 OR MIT | [http://github.com/tailhook/quick-error](http://github.com/tailhook/quick-error) |
| quinn | 0.11.11 | Apache-2.0 OR MIT | [https://github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| quinn-proto | 0.11.16 | Apache-2.0 OR MIT | [https://github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| quinn-udp | 0.5.15 | Apache-2.0 OR MIT | [https://github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| quote | 1.0.47 | Apache-2.0 OR MIT | [https://github.com/dtolnay/quote](https://github.com/dtolnay/quote) |
| r-efi | 5.3.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT | [https://github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| r-efi | 6.0.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT | [https://github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| radium | 0.7.0 | MIT | [https://github.com/bitvecto-rs/radium](https://github.com/bitvecto-rs/radium) |
| rand | 0.10.2 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| rand | 0.8.7 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| rand_core | 0.10.1 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand_core](https://github.com/rust-random/rand_core) |
| rand_core | 0.6.4 | Apache-2.0 OR MIT | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| rand_pcg | 0.10.2 | Apache-2.0 OR MIT | [https://github.com/rust-random/rngs](https://github.com/rust-random/rngs) |
| rayon | 1.12.0 | Apache-2.0 OR MIT | [https://github.com/rayon-rs/rayon](https://github.com/rayon-rs/rayon) |
| rayon-core | 1.13.0 | Apache-2.0 OR MIT | [https://github.com/rayon-rs/rayon](https://github.com/rayon-rs/rayon) |
| read-fonts | 0.41.0 | Apache-2.0 OR MIT | [https://github.com/googlefonts/fontations](https://github.com/googlefonts/fontations) |
| redox_syscall | 0.5.18 | MIT | [https://gitlab.redox-os.org/redox-os/syscall](https://gitlab.redox-os.org/redox-os/syscall) |
| redox_users | 0.4.6 | MIT | [https://gitlab.redox-os.org/redox-os/users](https://gitlab.redox-os.org/redox-os/users) |
| redox_users | 0.5.2 | MIT | [https://gitlab.redox-os.org/redox-os/users](https://gitlab.redox-os.org/redox-os/users) |
| regex | 1.13.1 | Apache-2.0 OR MIT | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| regex-automata | 0.4.18 | Apache-2.0 OR MIT | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| regex-syntax | 0.8.11 | Apache-2.0 OR MIT | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| rend | 0.4.2 | MIT | [https://github.com/djkoloski/rend](https://github.com/djkoloski/rend) |
| reqwest | 0.13.1 | Apache-2.0 OR MIT | [https://github.com/seanmonstar/reqwest](https://github.com/seanmonstar/reqwest) |
| resvg | 0.48.1 | Apache-2.0 OR MIT | [https://github.com/linebender/resvg](https://github.com/linebender/resvg) |
| rgb | 0.8.53 | MIT | [https://github.com/kornelski/rust-rgb](https://github.com/kornelski/rust-rgb) |
| ring | 0.17.14 | Apache-2.0 AND ISC | [https://github.com/briansmith/ring](https://github.com/briansmith/ring) |
| rkyv | 0.7.46 | MIT | [https://github.com/rkyv/rkyv](https://github.com/rkyv/rkyv) |
| rkyv_derive | 0.7.46 | MIT | [https://github.com/rkyv/rkyv](https://github.com/rkyv/rkyv) |
| roxmltree | 0.20.0 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/roxmltree](https://github.com/RazrFalcon/roxmltree) |
| roxmltree | 0.21.1 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/roxmltree](https://github.com/RazrFalcon/roxmltree) |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | [https://github.com/rust-lang/rustc-hash](https://github.com/rust-lang/rustc-hash) |
| rustix | 1.1.4 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| rustls | 0.23.43 | Apache-2.0 OR ISC OR MIT | [https://github.com/rustls/rustls](https://github.com/rustls/rustls) |
| rustls-native-certs | 0.8.4 | Apache-2.0 OR ISC OR MIT | [https://github.com/rustls/rustls-native-certs](https://github.com/rustls/rustls-native-certs) |
| rustls-pki-types | 1.15.1 | Apache-2.0 OR MIT | [https://github.com/rustls/pki-types](https://github.com/rustls/pki-types) |
| rustls-platform-verifier | 0.6.2 | Apache-2.0 OR MIT | [https://github.com/rustls/rustls-platform-verifier](https://github.com/rustls/rustls-platform-verifier) |
| rustls-platform-verifier-android | 0.1.1 | Apache-2.0 OR MIT | [https://github.com/rustls/rustls-platform-verifier](https://github.com/rustls/rustls-platform-verifier) |
| rustls-webpki | 0.103.14 | ISC | [https://github.com/rustls/webpki](https://github.com/rustls/webpki) |
| rustversion | 1.0.23 | Apache-2.0 OR MIT | [https://github.com/dtolnay/rustversion](https://github.com/dtolnay/rustversion) |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | [https://github.com/dtolnay/ryu](https://github.com/dtolnay/ryu) |
| same-file | 1.0.6 | MIT OR Unlicense | [https://github.com/BurntSushi/same-file](https://github.com/BurntSushi/same-file) |
| schannel | 0.1.29 | MIT | [https://github.com/steffengy/schannel-rs](https://github.com/steffengy/schannel-rs) |
| scopeguard | 1.2.0 | Apache-2.0 OR MIT | [https://github.com/bluss/scopeguard](https://github.com/bluss/scopeguard) |
| seahash | 4.1.0 | MIT | [https://gitlab.redox-os.org/redox-os/seahash](https://gitlab.redox-os.org/redox-os/seahash) |
| security-framework | 3.7.0 | Apache-2.0 OR MIT | [https://github.com/kornelski/rust-security-framework](https://github.com/kornelski/rust-security-framework) |
| security-framework-sys | 2.17.0 | Apache-2.0 OR MIT | [https://github.com/kornelski/rust-security-framework](https://github.com/kornelski/rust-security-framework) |
| self_cell | 1.3.0 | Apache-2.0 OR GPL-2.0 | [https://github.com/Voultapher/self_cell](https://github.com/Voultapher/self_cell) |
| seq-macro | 0.3.6 | Apache-2.0 OR MIT | [https://github.com/dtolnay/seq-macro](https://github.com/dtolnay/seq-macro) |
| serde | 1.0.229 | Apache-2.0 OR MIT | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| serde-content | 0.1.2 | Apache-2.0 OR MIT | [https://github.com/rushmorem/serde-content](https://github.com/rushmorem/serde-content) |
| serde_core | 1.0.229 | Apache-2.0 OR MIT | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| serde_derive | 1.0.229 | Apache-2.0 OR MIT | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| serde_json | 1.0.151 | Apache-2.0 OR MIT | [https://github.com/serde-rs/json](https://github.com/serde-rs/json) |
| serde_regex | 1.2.0 | Apache-2.0 OR MIT |  |
| serde_spanned | 1.1.1 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| sha1 | 0.10.7 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| sha2 | 0.11.0 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| shlex | 2.0.1 | Apache-2.0 OR MIT | [https://github.com/comex/rust-shlex](https://github.com/comex/rust-shlex) |
| signal-hook-registry | 1.4.8 | Apache-2.0 OR MIT | [https://github.com/vorner/signal-hook](https://github.com/vorner/signal-hook) |
| simd-abstraction | 0.7.1 | MIT | [https://github.com/Nugine/simd](https://github.com/Nugine/simd) |
| simd-adler32 | 0.3.10 | MIT | [https://github.com/mcountryman/simd-adler32](https://github.com/mcountryman/simd-adler32) |
| simdutf8 | 0.1.5 | Apache-2.0 OR MIT | [https://github.com/rusticstuff/simdutf8](https://github.com/rusticstuff/simdutf8) |
| similar | 2.7.0 | Apache-2.0 | [https://github.com/mitsuhiko/similar](https://github.com/mitsuhiko/similar) |
| simplecss | 0.2.2 | Apache-2.0 OR MIT | [https://github.com/linebender/simplecss](https://github.com/linebender/simplecss) |
| siphasher | 1.0.3 | Apache-2.0 OR MIT | [https://github.com/jedisct1/rust-siphash](https://github.com/jedisct1/rust-siphash) |
| skrifa | 0.44.0 | Apache-2.0 OR MIT | [https://github.com/googlefonts/fontations](https://github.com/googlefonts/fontations) |
| slab | 0.4.12 | MIT | [https://github.com/tokio-rs/slab](https://github.com/tokio-rs/slab) |
| slotmap | 1.1.1 | Zlib | [https://github.com/orlp/slotmap](https://github.com/orlp/slotmap) |
| smallvec | 1.15.2 | Apache-2.0 OR MIT | [https://github.com/servo/rust-smallvec](https://github.com/servo/rust-smallvec) |
| smawk | 0.3.3 | MIT | [https://github.com/mgeisler/smawk](https://github.com/mgeisler/smawk) |
| socket2 | 0.6.5 | Apache-2.0 OR MIT | [https://github.com/rust-lang/socket2](https://github.com/rust-lang/socket2) |
| stable_deref_trait | 1.2.1 | Apache-2.0 OR MIT | [https://github.com/storyyeller/stable_deref_trait](https://github.com/storyyeller/stable_deref_trait) |
| static_assertions | 1.1.0 | Apache-2.0 OR MIT | [https://github.com/nvzqz/static-assertions-rs](https://github.com/nvzqz/static-assertions-rs) |
| strict-num | 0.1.1 | MIT | [https://github.com/RazrFalcon/strict-num](https://github.com/RazrFalcon/strict-num) |
| strip-ansi-escapes | 0.2.1 | Apache-2.0 OR MIT | [https://github.com/luser/strip-ansi-escapes](https://github.com/luser/strip-ansi-escapes) |
| strsim | 0.11.1 | MIT | [https://github.com/rapidfuzz/strsim-rs](https://github.com/rapidfuzz/strsim-rs) |
| subtle | 2.6.1 | BSD-3-Clause | [https://github.com/dalek-cryptography/subtle](https://github.com/dalek-cryptography/subtle) |
| svgtypes | 0.16.1 | Apache-2.0 OR MIT | [https://github.com/linebender/svgtypes](https://github.com/linebender/svgtypes) |
| syn | 1.0.109 | Apache-2.0 OR MIT | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| syn | 2.0.119 | Apache-2.0 OR MIT | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| syn | 3.0.3 | Apache-2.0 OR MIT | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| sync_wrapper | 1.0.2 | Apache-2.0 | [https://github.com/Actyx/sync_wrapper](https://github.com/Actyx/sync_wrapper) |
| synstructure | 0.13.2 | MIT | [https://github.com/mystor/synstructure](https://github.com/mystor/synstructure) |
| tabwriter | 1.4.1 | MIT OR Unlicense | [https://github.com/BurntSushi/tabwriter](https://github.com/BurntSushi/tabwriter) |
| tap | 1.0.1 | MIT | [https://github.com/myrrlyn/tap](https://github.com/myrrlyn/tap) |
| tempfile | 3.27.0 | Apache-2.0 OR MIT | [https://github.com/Stebalien/tempfile](https://github.com/Stebalien/tempfile) |
| termtree | 0.5.1 | MIT | [https://github.com/rust-cli/termtree](https://github.com/rust-cli/termtree) |
| textwrap | 0.16.2 | MIT | [https://github.com/mgeisler/textwrap](https://github.com/mgeisler/textwrap) |
| thiserror | 1.0.69 | Apache-2.0 OR MIT | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| thiserror | 2.0.20 | Apache-2.0 OR MIT | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| thiserror-impl | 1.0.69 | Apache-2.0 OR MIT | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| thiserror-impl | 2.0.20 | Apache-2.0 OR MIT | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| tiny-skia | 0.12.0 | BSD-3-Clause | [https://github.com/linebender/tiny-skia](https://github.com/linebender/tiny-skia) |
| tiny-skia-path | 0.12.0 | BSD-3-Clause | [https://github.com/linebender/tiny-skia/tree/master/path](https://github.com/linebender/tiny-skia/tree/master/path) |
| tinystr | 0.8.4 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| tinytemplate | 1.2.1 | Apache-2.0 OR MIT | [https://github.com/bheisler/TinyTemplate](https://github.com/bheisler/TinyTemplate) |
| tinyvec | 1.12.0 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Lokathor/tinyvec](https://github.com/Lokathor/tinyvec) |
| tinyvec_macros | 0.1.1 | Apache-2.0 OR MIT OR Zlib | [https://github.com/Soveu/tinyvec_macros](https://github.com/Soveu/tinyvec_macros) |
| tokio | 1.53.1 | MIT | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| tokio-macros | 2.7.2 | MIT | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| tokio-rustls | 0.26.4 | Apache-2.0 OR MIT | [https://github.com/rustls/tokio-rustls](https://github.com/rustls/tokio-rustls) |
| tokio-util | 0.7.19 | MIT | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| toml | 1.1.4+spec-1.1.0 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_datetime | 1.1.1+spec-1.1.0 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_parser | 1.1.3+spec-1.1.0 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_writer | 1.1.2+spec-1.1.0 | Apache-2.0 OR MIT | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| tower | 0.5.3 | MIT | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tower-http | 0.6.11 | MIT | [https://github.com/tower-rs/tower-http](https://github.com/tower-rs/tower-http) |
| tower-layer | 0.3.3 | MIT | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tower-service | 0.3.3 | MIT | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tracing | 0.1.44 | MIT | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| tracing-attributes | 0.1.31 | MIT | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| tracing-core | 0.1.36 | MIT | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| try-lock | 0.2.5 | MIT | [https://github.com/seanmonstar/try-lock](https://github.com/seanmonstar/try-lock) |
| typenum | 1.20.1 | Apache-2.0 OR MIT | [https://github.com/paholg/typenum](https://github.com/paholg/typenum) |
| unicode-bidi | 0.3.18 | Apache-2.0 OR MIT | [https://github.com/servo/unicode-bidi](https://github.com/servo/unicode-bidi) |
| unicode-id | 0.3.6 | Apache-2.0 OR MIT | [https://github.com/Boshen/unicode-id](https://github.com/Boshen/unicode-id) |
| unicode-id-start | 1.4.0 | (Apache-2.0 OR MIT) AND Unicode-3.0 | [https://github.com/Boshen/unicode-id-start](https://github.com/Boshen/unicode-id-start) |
| unicode-ident | 1.0.24 | (Apache-2.0 OR MIT) AND Unicode-3.0 | [https://github.com/dtolnay/unicode-ident](https://github.com/dtolnay/unicode-ident) |
| unicode-linebreak | 0.1.5 | Apache-2.0 | [https://github.com/axelf4/unicode-linebreak](https://github.com/axelf4/unicode-linebreak) |
| unicode-script | 0.5.8 | Apache-2.0 OR MIT | [https://github.com/unicode-rs/unicode-script](https://github.com/unicode-rs/unicode-script) |
| unicode-segmentation | 1.13.3 | Apache-2.0 OR MIT | [https://github.com/unicode-rs/unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation) |
| unicode-vo | 0.1.0 | Apache-2.0 OR MIT | [https://github.com/RazrFalcon/unicode-vo](https://github.com/RazrFalcon/unicode-vo) |
| unicode-width | 0.2.2 | Apache-2.0 OR MIT | [https://github.com/unicode-rs/unicode-width](https://github.com/unicode-rs/unicode-width) |
| untrusted | 0.9.0 | ISC | [https://github.com/briansmith/untrusted](https://github.com/briansmith/untrusted) |
| ureq | 2.12.1 | Apache-2.0 OR MIT | [https://github.com/algesten/ureq](https://github.com/algesten/ureq) |
| url | 2.5.8 | Apache-2.0 OR MIT | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| usvg | 0.48.1 | Apache-2.0 OR MIT | [https://github.com/linebender/resvg](https://github.com/linebender/resvg) |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT | [https://github.com/hsivonen/utf8_iter](https://github.com/hsivonen/utf8_iter) |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | [https://github.com/alacritty/vte](https://github.com/alacritty/vte) |
| uuid | 1.24.1 | Apache-2.0 OR MIT | [https://github.com/uuid-rs/uuid](https://github.com/uuid-rs/uuid) |
| version_check | 0.9.5 | Apache-2.0 OR MIT | [https://github.com/SergioBenitez/version_check](https://github.com/SergioBenitez/version_check) |
| vlq | 0.5.1 | Apache-2.0 OR MIT | [https://github.com/tromey/vlq](https://github.com/tromey/vlq) |
| vsimd | 0.8.0 | MIT | [https://github.com/Nugine/simd](https://github.com/Nugine/simd) |
| vte | 0.14.1 | Apache-2.0 OR MIT | [https://github.com/alacritty/vte](https://github.com/alacritty/vte) |
| wait-timeout | 0.2.1 | Apache-2.0 OR MIT | [https://github.com/alexcrichton/wait-timeout](https://github.com/alexcrichton/wait-timeout) |
| walkdir | 2.5.0 | MIT OR Unlicense | [https://github.com/BurntSushi/walkdir](https://github.com/BurntSushi/walkdir) |
| want | 0.3.1 | MIT | [https://github.com/seanmonstar/want](https://github.com/seanmonstar/want) |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/bytecodealliance/wasi](https://github.com/bytecodealliance/wasi) |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/bytecodealliance/wasi-rs](https://github.com/bytecodealliance/wasi-rs) |
| wasm-bindgen | 0.2.127 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) |
| wasm-bindgen-futures | 0.4.77 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures) |
| wasm-bindgen-macro | 0.2.127 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| wasm-bindgen-macro-support | 0.2.127 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support) |
| wasm-bindgen-shared | 0.2.127 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| web-sys | 0.3.104 | Apache-2.0 OR MIT | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| web-time | 1.1.0 | Apache-2.0 OR MIT | [https://github.com/daxpedda/web-time](https://github.com/daxpedda/web-time) |
| webpki-root-certs | 1.0.9 | CDLA-Permissive-2.0 | [https://github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| webpki-roots | 0.26.11 | CDLA-Permissive-2.0 | [https://github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| webpki-roots | 1.0.9 | CDLA-Permissive-2.0 | [https://github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| weezl | 0.1.12 | Apache-2.0 OR MIT | [https://github.com/image-rs/weezl](https://github.com/image-rs/weezl) |
| winapi | 0.3.9 | Apache-2.0 OR MIT | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| winapi-i686-pc-windows-gnu | 0.4.0 | Apache-2.0 OR MIT | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| winapi-util | 0.1.11 | MIT OR Unlicense | [https://github.com/BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util) |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | Apache-2.0 OR MIT | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| windows-link | 0.2.1 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.45.0 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.48.0 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.52.0 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.61.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.42.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.42.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.42.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.42.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnullvm | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.42.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.42.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.42.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.42.2 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.48.5 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.52.6 | Apache-2.0 OR MIT | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| winnow | 1.0.4 | MIT | [https://github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| wit-bindgen | 0.57.1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [https://github.com/bytecodealliance/wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) |
| writeable | 0.6.4 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| wyz | 0.5.1 | MIT | [https://github.com/myrrlyn/wyz](https://github.com/myrrlyn/wyz) |
| xmlwriter | 0.1.0 | MIT | [https://github.com/RazrFalcon/xmlwriter](https://github.com/RazrFalcon/xmlwriter) |
| yoke | 0.8.3 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| yoke-derive | 0.8.2 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerocopy | 0.8.56 | Apache-2.0 OR BSD-2-Clause OR MIT | [https://github.com/google/zerocopy](https://github.com/google/zerocopy) |
| zerocopy-derive | 0.8.56 | Apache-2.0 OR BSD-2-Clause OR MIT | [https://github.com/google/zerocopy](https://github.com/google/zerocopy) |
| zerofrom | 0.1.8 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerofrom-derive | 0.1.7 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zeroize | 1.9.0 | Apache-2.0 OR MIT | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| zerotrie | 0.2.5 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerovec | 0.11.7 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerovec-derive | 0.11.4 | Unicode-3.0 | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zmij | 1.0.23 | MIT | [https://github.com/dtolnay/zmij](https://github.com/dtolnay/zmij) |
| zune-core | 0.5.3 | Apache-2.0 OR MIT OR Zlib | [https://github.com/etemesi254/zune-image](https://github.com/etemesi254/zune-image) |
| zune-jpeg | 0.5.15 | Apache-2.0 OR MIT OR Zlib | [https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg](https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg) |
