# Validation and benchmarks

C++: MSVC `/O2 /fp:precise`. Rust: Cargo release profile.
Machine: Windows x64, Intel64 Family 6 Model 183 Stepping 1, GenuineIntel. Toolchain: rustc 1.99.0 (b940084d7 2026-09-28).
Upstream revision: `d05d9c2e6729b13d77b7aad17cd8a0d139ff010e`. The [raw report](validation/windows-x64.json) records
the local C++ user edits, source/header hashes, flags and all timing samples.

## Behavioral comparison

- 1,794 tests pass with all features; 1,527 pass without optional features. No failures or ignored tests.
- Clippy passes for all targets/features with `-D warnings`; formatting, warning-free rustdoc and Cargo
  package verification pass. Rust 1.85 builds all library features. Linux x64 (including hooks) and macOS
  ARM64 libraries cross-check successfully; runtime validation remains Windows x64 locally.
- 428 numeric rows: vectors, matrices and all nine engine cameras.
- 72 ordered HUD commands: boxes, bars, labels/outlines, rings, icons, snap lines,
  dashed corners, skeletons and screen indicators.
- Navigation serialization matches six-significant-digit floats, exponent notation, signed zero,
  neighbors and event tags. Both language directions round-trip; vertex iteration order is normalized.
- The same behavior fixtures pass against a clean extraction of the committed upstream revision.

## Release microbenchmarks

Five samples per language after warmup; medians exclude startup. Workloads and iteration counts are
matched, checksums agree, and volatile iteration indices prevent constant folding. These limited kernels
do not establish whole-application performance parity.

| Kernel | Iterations | C++ median (ms) | Rust median (ms) | Rust / C++ time |
| --- | ---: | ---: | ---: | ---: |
| vector | 500,000 | 1.016 | 1.002 | 0.986 |
| matrix | 100,000 | 1.208 | 1.245 | 1.030 |
| scan | 256 | 0.582 | 0.168 | 0.289 |

The original Rust scanner took about 48 times the C++ time on this zero-filled-buffer workload.
SIMD-capable `memchr` anchoring removed that regression. Generated-input tests verify wildcard and
boundary semantics after the optimization.

## Reproduce

```sh
cargo test --locked --no-default-features
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --locked --all-features --no-deps
cargo bench --bench kernels
python tools/compare_cpp.py --cpp ../omath --repeats 5
```

The tool writes reference outputs, nav fixtures and a report to `target/cpp-comparison/`. Visual Studio
is discovered automatically on Windows. `--compiler` selects a C++23 compiler; its standard library must
support upstream `<format>` and `<expected>`.

Native detour tests check callback arguments, forwarded return values, replacement during dispatch
and restoration on removal. Window-procedure interception/restoration also passes. Local DX9/11/12/OpenGL
discovery/install/remove succeeded, including from the loadable DLL. The native showcase rendered its three
scenes and observed one OpenGL callback per swap with the real driver. Full game overlays were not tested.
Linux passed library cross-checks; Linux/macOS runtime coverage is in CI.
