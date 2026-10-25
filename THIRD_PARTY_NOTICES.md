# Third-party notices / Сторонние компоненты

CubeXCraft Installer использует сторонние компоненты. Каждый распространяется по своей лицензии; лицензия CubeXCraft
(`LICENSE`) на них не распространяется и не ограничивает права, которые дают их лицензии.

CubeXCraft Installer uses the third-party components listed below, each under its own license; the CubeXCraft license
(`LICENSE`) does not apply to them and does not restrict their licenses.

## Основа / Base

| Component | Copyright | License |
|---|---|---|
| iloader (the installer this one is based on) | Copyright (c) 2025 nab138 | MIT — `LICENSE-MIT` |
| isideload | nab138 (https://github.com/nab138/isideload) | MIT |
| idevice | jkcoxson (https://github.com/jkcoxson/idevice) | MIT |
| apple-codesign-quick | see https://crates.io/crates/apple-codesign-quick | LGPL-2.1-or-later — you may replace it and rebuild the installer from this repository |

## Fonts / Шрифты

Bundled unmodified in `src/assets/fonts/` under the SIL Open Font License 1.1 (texts: `src/assets/fonts/OFL-*.txt`):
JetBrains Mono (Copyright 2020 The JetBrains Mono Project Authors), Manrope (Copyright 2019 The Manrope Project Authors),
Unbounded (Copyright 2022 The Unbounded Project Authors).

## Icons / Иконки

Font Awesome Free 6 icons (Fonticons, Inc., https://fontawesome.com), used through react-icons: the icons are licensed
under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/), the react-icons code under MIT.

## JavaScript

| Package | License |
|---|---|
| react, react-dom | MIT |
| i18next, i18next-browser-languagedetector, react-i18next | MIT |
| react-icons | MIT (icons: see above) |
| react-virtuoso | MIT |
| sonner | MIT |
| @tauri-apps/api, @tauri-apps/plugin-dialog, -opener, -process, -store, -updater | Apache-2.0 OR MIT |

## Rust crates

The installer core is built with the crates pinned in `src-tauri/Cargo.lock` (all target platforms together). Their
license texts and copyright notices ship with each crate's source on https://crates.io. For crates offered under several
licenses, CubeXCraft uses a permissive option (MIT or Apache-2.0 where available). The MPL-2.0 crates are used unmodified.

### MIT OR Apache-2.0 (295)

aead, aes, android_system_properties, anyhow, arbitrary, asn1-rs, asn1-rs-derive, asn1-rs-impl, async-broadcast, async-compression, async-recursion, async-trait, base64, bitflags, block-buffer, block-padding, bs58, bumpalo, camino, cargo-platform, cbc, cc, cfg-expr, cfg-if, chacha20, chrono, cipher, cmake, compression-codecs, compression-core, cookie, core-foundation, core-foundation-sys, core-graphics, core-graphics-types, core_detect, cpubits, cpufeatures, crc32fast, crossbeam-channel, crossbeam-deque, crossbeam-epoch, crossbeam-utils, crypto-common, ctr, curve25519-dalek-derive, dbus-secret-service, defmt, defmt-macros, defmt-parser, der-parser, deranged, derive_arbitrary, des, digest, dirs, dirs-sys, displaydoc, dtoa, dyn-clone, either, embed_plist, enumflags2, enumflags2_derive, erased-serde, errno, fdeflate, field-offset, filetime, find-msvc-tools, flate2, foreign-types, foreign-types-macros, foreign-types-shared, form_urlencoded, futures, futures-channel, futures-core, futures-executor, futures-io, futures-macro, futures-sink, futures-task, futures-util, getrandom, glob, hashbrown, heck, hermit-abi, hex, hkdf, hmac, html5ever, http, httparse, hybrid-array, hyper-tls, iana-time-zone, iana-time-zone-haiku, ident_case, idevice-srp, idna, inout, ipnet, isideload-p12-keystore, isideload-vfs, itoa, jni, jni-macros, jni-sys, jni-sys-macros, jobserver, js-sys, json-patch, jsonptr, keyboard-types, keyring, lazy_static, libc, lock_api, log, mac-notification-sys, markup5ever, md-5, memmap2, mime, minimal-lexical, multiversion, multiversion-macros, native-tls, ndk, ndk-context, ndk-sys, notify-rust, num-bigint, num-bigint-dig, num-conv, num-integer, num-iter, num-traits, oid-registry, once_cell, opaque-debug, openssl-macros, openssl-probe, ordered-stream, osakit, parking_lot, parking_lot_core, pbkdf2, percent-encoding, pin-utils, piper, pkg-config, png, powerfmt, ppv-lite86, proc-macro-crate, proc-macro-error, proc-macro-error-attr, proc-macro2, qrcode, quinn, quinn-proto, quinn-udp, quote, rand, rand_chacha, rand_core, rand_pcg, rayon, rayon-core, rc2, rcgen, ref-cast, ref-cast-impl, regex, regex-automata, regex-syntax, reqwest, reqwest-middleware, rootcause, rootcause-internals, rsa, rustc_version, rusticata-macros, rustls-pki-types, rustls-platform-verifier, rustls-platform-verifier-android, rustversion, salsa20, scopeguard, scrypt, security-framework, security-framework-sys, semver, serde, serde-untagged, serde_core, serde_derive, serde_derive_internals, serde_json, serde_repr, serde_spanned, serde_with, serde_with_macros, serialize-to-javascript, serialize-to-javascript-impl, servo_arc, sha1, sha2, sha3, shlex, signal-hook-registry, simdutf8, siphasher, smallvec, socket2, softbuffer, srp, stable_deref_trait, string_cache, string_cache_codegen, swift-rs, symlink, syn, system-configuration, system-configuration-sys, system-deps, tao-macros, tar, tauri-winrt-notification, tempfile, tendril, thiserror, thiserror-impl, thread_local, time, time-core, time-macros, tokio-rustls, toml, toml_datetime, toml_edit, toml_parser, toml_writer, tray-icon, triomphe, tungstenite, typed-path, typeid, typenum, unicode-segmentation, universal-hash, url, vcpkg, version_check, wasm-bindgen, wasm-bindgen-futures, wasm-bindgen-macro, wasm-bindgen-macro-support, wasm-bindgen-shared, wasm-streams, web-sys, web-time, web_atoms, winapi, winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu, windows, windows-collections, windows-core, windows-future, windows-implement, windows-interface, windows-link, windows-numerics, windows-registry, windows-result, windows-strings, windows-sys, windows-targets, windows-threading, windows-version, windows_aarch64_gnullvm, windows_aarch64_msvc, windows_i686_gnu, windows_i686_gnullvm, windows_i686_msvc, windows_x86_64_gnu, windows_x86_64_gnullvm, windows_x86_64_msvc, x509-parser, xattr, yasna

### MIT (145)

async-stream, async-stream-impl, async_zip, atk, atk-sys, auto-launch, block2, bytes, cairo-rs, cairo-sys-rs, cargo_metadata, cfb, cfg_aliases, combine, darling, darling_core, darling_macro, data-encoding, derive_more, derive_more-impl, dlopen2, dlopen2_derive, dom_query, embed-resource, endi, fs_extra, gdk, gdk-pixbuf, gdk-pixbuf-sys, gdk-sys, gdkwayland-sys, gdkx11, gdkx11-sys, generic-array, gio, gio-sys, glib, glib-macros, glib-sys, gobject-sys, gtk, gtk-sys, gtk3-macros, h2, http-body, http-body-util, hyper, hyper-util, ico, idevice, infer, is-docker, is-wsl, isideload, javascriptcore-rs, javascriptcore-rs-sys, jktcp, libm, libredox, memoffset, minisign-verify, mio, new_debug_unreachable, nix, nom, nu-ansi-term, objc2, objc2-encode, objc2-foundation, open, openssl-sys, os_info, pango, pango-sys, pem, phf, phf_codegen, phf_generator, phf_macros, phf_shared, plist, plist-macro, precomputed-hash, quick-xml, redox_syscall, redox_users, rfd, schannel, schemars, schemars_derive, sharded-slab, simd-adler32, slab, smappservice-rs, soup3, soup3-sys, spin, strsim, strum, strum_macros, synstructure, tauri-winres, tokio, tokio-macros, tokio-native-tls, tokio-tungstenite, tokio-util, tower, tower-http, tower-layer, tower-service, tracing, tracing-appender, tracing-attributes, tracing-core, tracing-log, tracing-subscriber, try-lock, uds_windows, urlencoding, urlpattern, valuable, version-compare, vswhom, vswhom-sys, want, wasmtimer, webkit2gtk, webkit2gtk-sys, webview2-com, webview2-com-macros, webview2-com-sys, winnow, winreg, x11, x11-dl, zbus, zbus_macros, zbus_names, zcheapstr, zip, zmij, zvariant, zvariant_derive, zvariant_utils

### Apache-2.0 OR MIT (90)

aes-gcm, async-channel, async-executor, async-io, async-lock, async-process, async-signal, async-task, atomic-waker, autocfg, base16ct, base64ct, bit-set, bit-vec, blocking, cargo_toml, cesu8, chacha20, chacha20poly1305, cmov, cms, concurrent-queue, const-oid, crypto-bigint, ctor, ctutils, dbus, der, der_derive, ed25519, equivalent, event-listener, event-listener-strategy, fastrand, fnv, futures-lite, ghash, idna_adapter, indexmap, keccak, libappindicator, libappindicator-sys, libdbus-sys, linux-keyutils, muda, multiversion_no_op, parking, pem-rfc7468, pin-project, pin-project-internal, pin-project-lite, pkcs1, pkcs12, pkcs5, pkcs8, polling, poly1305, polyval, portable-atomic, portable-atomic-util, rustc-hash, serdect, signature, simd_cesu8, spki, tauri, tauri-build, tauri-codegen, tauri-macros, tauri-plugin, tauri-plugin-autostart, tauri-plugin-dialog, tauri-plugin-fs, tauri-plugin-notification, tauri-plugin-opener, tauri-plugin-process, tauri-plugin-store, tauri-plugin-updater, tauri-runtime, tauri-runtime-wry, tauri-utils, tls_codec, tls_codec_derive, utf8_iter, uuid, window-vibrancy, wry, x509-cert, zeroize, zeroize_derive

### Zlib OR Apache-2.0 OR MIT (20)

bytemuck, dispatch2, objc2-app-kit, objc2-cloud-kit, objc2-core-data, objc2-core-foundation, objc2-core-graphics, objc2-core-image, objc2-core-location, objc2-core-text, objc2-exception-helper, objc2-io-surface, objc2-osa-kit, objc2-quartz-core, objc2-security, objc2-service-management, objc2-ui-kit, objc2-user-notifications, objc2-web-kit, tinyvec

### Unicode-3.0 (18)

icu_collections, icu_locale_core, icu_normalizer, icu_normalizer_data, icu_properties, icu_properties_data, icu_provider, litemap, potential_utf, tinystr, writeable, yoke, yoke-derive, zerofrom, zerofrom-derive, zerotrie, zerovec, zerovec-derive

### Unlicense OR MIT (11)

aho-corasick, byteorder, jiff, jiff-core, jiff-static, jiff-tzdb, jiff-tzdb-platform, memchr, same-file, walkdir, winapi-util

### BSD-3-Clause (6)

alloc-no-stdlib, alloc-stdlib, curve25519-dalek, ed25519-dalek, subtle, x25519-dalek

### MPL-2.0 (5)

cssparser, cssparser-macros, dtoa-short, option-ext, selectors

### Apache-2.0 (5)

flagset, openssl, sync_wrapper, tao, zopfli

### Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT (5)

linux-raw-sys, rustix, wasi, wasip2, wit-bindgen

### Apache-2.0 OR ISC OR MIT (3)

hyper-rustls, rustls, rustls-native-certs

### ISC (3)

libloading, rustls-webpki, untrusted

### MIT OR Apache-2.0 OR Zlib (3)

lru-slab, raw-window-handle, static_vcruntime

### Zlib (2)

foldhash, zlib-rs

### BSD-3-Clause OR MIT OR Apache-2.0 (2)

num_enum, num_enum_derive

### CDLA-Permissive-2.0 (2)

webpki-root-certs, webpki-roots

### BSD-2-Clause OR Apache-2.0 OR MIT (2)

zerocopy, zerocopy-derive

### 0BSD OR MIT OR Apache-2.0 (1)

adler2

### LGPL-2.1-or-later (1)

apple-codesign-quick

### ISC AND (Apache-2.0 OR ISC) (1)

aws-lc-rs

### ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) (1)

aws-lc-sys

### BSD-3-Clause AND MIT (1)

brotli

### BSD-3-Clause OR MIT (1)

brotli-decompressor

### Apache-2.0 AND MIT (1)

dpi

### CC0-1.0 OR MIT-0 OR Apache-2.0 (1)

dunce

### (Apache-2.0 OR MIT) AND BSD-3-Clause (1)

encoding_rs

### MIT OR Apache-2.0 OR BSD-1-Clause (1)

fiat-crypto

### MIT OR Zlib OR Apache-2.0 (1)

miniz_oxide

### MIT OR Apache-2.0 OR LGPL-2.1-or-later (1)

r-efi

### Apache-2.0 AND ISC (1)

ring

### Apache-2.0 WITH LLVM-exception (1)

target-lexicon

### (MIT OR Apache-2.0) AND Unicode-3.0 (1)

unicode-ident

