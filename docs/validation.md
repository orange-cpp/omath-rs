# Validation and benchmarks

The [completion audit](port-audit.md) records the resolved omissions and remaining runtime limits.
C++ uses MSVC `/O2 /fp:precise`; Rust uses Cargo release. The host is Windows x64, Intel64 Family 6 Model
183 Stepping 1. Reports record compiler/toolchain details, source hashes and samples.

## Behavioral evidence

Windows x64 and x86 each pass **1,803 all-feature tests**. The default x64 build passes **1,535**.
These totals include doctests; no tests failed or were ignored. Clippy passes with warnings denied on
both architectures. Formatting, rustdoc, Rust 1.85 library checking and Cargo package verification pass.
Linux x64 with hooks and default macOS ARM64 libraries cross-check; local runtime tests cover Windows.

The [C++ comparison](validation/windows-x64.json) passes 428 math/camera rows, 72 ordered HUD commands,
and 6,369 additional collision/projectile rows. Navigation's float notation, signed zero, neighbors and
events round-trip between languages in both directions. The [audit probes](validation/port-audit.json)
cover previously missed glow ordering, dashed outlines, gradient truncation and reverse ImGui APIs.

KF1 memory fixtures test reflection, camera/player discovery, class-address reuse, living-actor filtering,
aim selection and restoring controller/weapon changes. The real DX9 smoke test passes on x64 and x86:
it checks shader output, device state, ImGui geometry/font rendering and recreation after Reset. The
hidden OpenGL showcase smoke test passes in all three modes. Live KF1 and Linux/macOS runtime tests
were not performed locally. Full in-game equivalence remains unverified.

## Matched release kernels

Five samples per language after warmup; startup is outside timing. Counts and checksums match. These
small kernels provide local observations, not whole-application performance claims.

| Kernel | Iterations | C++ median (ms) | Rust median (ms) | Rust / C++ time |
| --- | ---: | ---: | ---: | ---: |
| vector | 500,000 | 1.002 | 0.897 | 0.896 |
| matrix | 100,000 | 1.177 | 1.228 | 1.043 |
| scan | 256 | 0.578 | 0.151 | 0.262 |

## Upstream benchmark suites

The [suite report](validation/upstream-benchmarks.json) covers **all 67 cases** registered by the seven
upstream suites: camera, collision, line tracing, matrices, pattern scanning, projectile prediction and
view matrices. The original C++ source files are compiled unchanged against a small bounded driver.
Deterministic clouds/rays, sphere meshes and MT19937 scan buffers reproduce upstream workload setup.
Projection percentages, ray-hit percentages and mesh-face counters agree between languages.

The driver is not Google Benchmark. It times measured loops while excluding setup, but optimization
barriers and allocators differ between languages. The default comparison uses 100 iterations and three
samples per case; small-case timings are particularly noisy. Use longer filtered runs when investigating
an individual workload, rather than interpreting these ratios as a speed guarantee.

On this MSVC library, `monotonic_buffer_resource::release()` continues consuming the caller's initial
block. The unchanged C++ `BM_Epa_MonotonicBuffer` and `BM_GjkEpa_Pipeline` throw `bad allocation` within
100 iterations. The report preserves those errors and uses fresh **10-iteration runs in both languages**
for those cases. Rust's reusable `EpaWorkspace` retains vector/heap/horizon capacity, which is a deliberate
allocator adaptation rather than a literal implementation of PMR. Fresh/reused numerical results match.

## Reproduce

```sh
cargo test --locked --no-default-features
cargo test --locked --all-features
cargo test --locked --all-features --target i686-pc-windows-msvc
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --locked --all-features --no-deps
cargo bench --bench upstream -- --smoke
cargo bench --bench upstream -- --filter=camera_ --iterations=100000
python tools/compare_cpp.py --cpp ../omath --repeats 5
python tools/audit_port.py --cpp ../omath --check-x86
python tools/compare_benchmarks.py --cpp ../omath --iterations 100 --repeats 3
```

The comparison tools leave the C++ checkout untouched. This run uses the captured working-tree version
in `target/omath-reference`; [its manifest](validation/cpp-reference.json) records all 448 file hashes.
The saved reports retain the captured dirty status and Rust source hashes. CI pins the committed upstream
revision and is configured for Windows/Linux/macOS host tests plus a separate Windows x86 job.
