# Port completion audit

The library implementation scope and the previously omitted examples and benchmark suites are ported
against the frozen C++ reference. The five concrete API/HUD/x86 defects from the previous audit are fixed.
The current probes find no new omissions in their tested scope. This is a functional Rust adaptation,
with the compatibility differences and runtime limits below; it is not an exhaustive equivalence proof.

The reference is revision `d05d9c2e6729b13d77b7aad17cd8a0d139ff010e` plus the user's captured working changes.
The sibling checkout changed during the work, so all comparisons use `target/omath-reference` instead.
All 448 captured files still match the [frozen manifest](validation/cpp-reference.json). C++ sources were
not edited. Reports include Rust source hashes because validation ran on the working tree before commit.

## Resolved audit findings

| Finding | Change and evidence |
| --- | --- |
| Windows x86 hooks did not build | WndProc bindings adapt their native i32/isize widths. All-feature x86 tests, all-target Clippy and both DLL builds pass. |
| Missing reverse ImGui conversions | Generic `Vector2::from_im_vec2` and `Vector4::from_im_vec4`, compile probes and round-trip tests are present. |
| Corner-box glow order | All glow lines precede fill and ordinary corners, in upstream order; the command stream matches C++. |
| Dashed side-bar outlines | Reversed rectangle corners and the extra left pixel match upstream; all four side fixtures pass. |
| Gradient quantization | Static RGB channels and alpha use upstream truncation; the actual Rust vertex-color probe matches the C++ formula. |
| KF1 example was incomplete | Shared reflected reader/tracker, auto-attaching dumper, external observer, x86 injected DX9 hooks, glow shaders, menu, aim and reversible fire-mode edits are implemented. See [KF1 guide](kf1.md). |
| Only three benchmark kernels | All seven upstream suites and all 67 expanded cases run in Rust and C++; projection/hit/face counters match. |
| Missing x86 CI | A dedicated x86 test/lint/DLL-build job is configured. |

The x86 runtime tests also found an architecture-dependent oversized navigation-neighbor count. Parsing
that text count as u64 makes missing-neighbor errors consistent across x86 and x64 without allocating from
an untrusted count.

## Scope and verification

All 133 upstream headers have corresponding Rust implementation mappings in the [raw audit](validation/port-audit.json).
This inventory is useful for finding omissions but does not prove every overload or edge case equivalent.
The original omitted hooks, process access, Unreal walkers, loaded-module scanning, HUD/Lua, targeting
and ImGui conversions have implementations and tests. `EncryptedVariable` is deliberately not ported. Cargo, rustdoc and the crate
import test supply the Rust packaging/documentation/module equivalents.

- Windows x64: 1,803 all-feature tests; x86: 1,803 all-feature tests; default x64: 1,535 tests. No failures or ignored tests.
- C++ differential checks: 428 math/camera rows, 72 HUD commands, 6,369 additional ray/GJK/EPA/projectile rows,
  plus navigation serialization in both directions. Reused EPA storage agrees with fresh storage in the extended fixtures.
- The audit's HUD and conversion regressions now pass. FOV wrap/roll differences are intentional and remain reported.
- All 67 benchmark cases execute in both languages. Two C++ bounded-arena cases throw `bad allocation` at
  100 iterations with this MSVC library. Reports preserve those failures and compare fresh 10-iteration
  runs for those two cases. Other cases use 100 iterations, with three samples per language.
- Real DX9 x64/x86 tests compile the four shaders, verify output pixels, state restoration, ImGui texture
  and indexed drawing, and font-resource recreation after Reset. Hidden OpenGL smoke tests pass in all three showcase modes.
- Formatting, Clippy with warnings denied on x64/x86, rustdoc, package verification and Rust 1.85 library checking pass.
- Linux x64 library checking with hooks and macOS ARM64 default-library checking pass. CI is configured for host runtime tests.

[Saved check evidence](validation/completion-checks.json) and [comparison/performance results](validation.md)
record the scope and limitations of these results.

## Deliberate adaptations and validation limits

FOV targeting uses consistent pitch/yaw and shortest-angle wrapping. The C++ template compiles; it includes
roll inconsistently in its current/best comparisons. Its wrap/roll counterexamples remain semantic differences,
not missing implementations. Marker types, ownership/errors, renderer descriptors, batched prediction and
reusable EPA workspace replace their C++ language/allocator counterparts. The [five retained upstream defects](porting.md#retained-upstream-defects)
are unchanged. Stock ImGui/native Rust shells replace Qt/GLFW/nullcore styling.

A live KF1 session was unavailable: native game exports, animation bone/line-trace calls, menu interaction
and in-game timing have build/fixture/graphics evidence but no end-to-end game run. Linux/macOS runtimes
were not executed locally. The Linux all-feature cross-check needs native cross compilers absent here;
the hooks-enabled library cross-check succeeds. CI jobs are configured, not claimed to have run remotely.
These are validation limits, so there is no claim of universal bit-for-bit or platform equivalence.

## Reproduce

```sh
cargo test --locked --all-features
cargo test --locked --all-features --target i686-pc-windows-msvc
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo run --locked --all-features --example kf1_glow_smoke
python tools/compare_cpp.py --cpp ../omath --repeats 5
python tools/audit_port.py --cpp ../omath --check-x86
python tools/compare_benchmarks.py --cpp ../omath --iterations 100 --repeats 3
```

Use `--cpp target/omath-reference` to repeat this captured run. The audit accepts `--imgui-include` for
the C++ conversion compile probe. `fully_ported: null` in its JSON deliberately means the finite probe
runner cannot issue an exhaustive completeness certificate; `known_gaps_detected: false` records its actual result.
