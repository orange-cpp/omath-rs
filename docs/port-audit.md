# Port completeness audit

**Verdict: not fully ported / not fully equivalent yet.** The earlier completion claim was too strong.

Library revision examined: `b936e4d1aba372e7d2bbae8ca9d00db9ace77b66`.
C++ revision: `d05d9c2e6729b13d77b7aad17cd8a0d139ff010e`, including the user's existing working-tree changes.
This audit adds probes, native-call tests and documentation; it does not change library implementations.

## Confirmed gaps

| Area | Evidence | Consequence |
| --- | --- | --- |
| Windows x86 hooks | `cargo check --all-features --target i686-pc-windows-msvc` fails with four `E0308` errors in `src/hooks/mod.rs` at the `SetWindowLongPtrW`/`GetWindowLongPtrW` calls and saved original procedure | The documented x86 configuration cannot build. On this target the bindings take/return `i32`, while the wrapper uses `isize` without adapting the calls. Windows x64 passes. |
| Reverse ImGui conversions | C++ `Vector2<float>::from_im_vec2` and `Vector4<float>::from_im_vec4` compile. The corresponding Rust calls fail with `E0599`. No equivalent `From<ImVec2/ImVec4>` implementations exist. | Two upstream public conversion APIs were omitted. The existing Rust `to_im_*` methods only cover the other direction. |
| Corner-box glow | The C++ command stream draws all glow lines first, then fill and normal corner lines. Rust draws the fill first and interleaves each corner's glow and normal lines. | The composition order of overlapping translucent primitives differs. The first C++ command is a blue glow line; Rust starts with a green filled-box outline. |
| Dashed side bars | Right-bar C++ outline starts at x=129; Rust starts at x=130. Left-bar C++ starts at x=65; Rust starts at x=66 for the same fixture. | Rust omits the upstream extra pixel on the outline's left edge. Rectangle corner order is also normalized. |
| ImGui gradient quantization | The upstream static gradient circle/text paths truncate `channel * 255`; a 0.5 channel gives 127. The Rust circle probe produces 128. | Gradient vertex colors are not byte-identical. This comparison checks the C++ conversion formula against actual Rust vertex output, not whole-renderer pixel equality. |

The dashed-top fixture also differs in rectangle corner ordering, without the side-bar width difference.
Its visual significance depends on the renderer; it is recorded as normalization rather than asserted to be
an independently proven rendering defect. The dashed-bottom command stream matches.

## Intentional differences, not missing code

The C++ FOV-targeting template **does compile and is callable**. `Vector3` inherits `Vector2`; my earlier
documentation incorrectly described the mixed dimensions as making the template unusable.

The audit demonstrates two semantic changes in Rust: shortest-angle wrapping selects target 0 where C++
selects target 1 across the yaw boundary, and ignoring roll in the distance metric likewise selects target 0
where the C++ mixed two/three-dimensional comparison selects target 1. These are reasonable corrections,
but they are deviations from literal parity and are now documented accurately.

Other documented adaptations include marker-type angle ranges/storage order, Rust error/ownership contracts,
renderer descriptors instead of C++ overloads, and the batched solver replacing an AVX2-specific implementation.
The five retained upstream defects remain listed in [porting notes](porting.md); their presence is separate
from the omissions identified here.

## Scope and coverage gaps

- All 133 C++ headers have mapped Rust implementation files in the [machine-readable inventory](validation/port-audit.json).
  This is a file-level inventory, not proof that every method/overload or edge case has been reproduced.
- The baseline differential harness still passes 428 math/camera rows, 72 HUD commands and navigation
  serialization in both directions. It does not compare every public method or every subsystem. The new
  counterexamples exercise paths absent from those 500 rows.
- The Rust showcase and generic hook DLL are adaptations. The full KF1 overlay, its tracking/menu behavior,
  and its DX9 actor-silhouette/blur/composite shader implementation do not have corresponding Rust examples.
- The three Rust microbenchmark kernels are not the complete upstream benchmark suite. Camera workloads,
  view-matrix construction, projectile prediction, collision and line-tracer benchmark suites remain absent;
  matrix/pattern scanning have representative kernels rather than all original cases.
- Linux/macOS runtime equivalence and full in-game overlays have not been established locally. The existing
  CI matrix does not cover Windows x86. A successful cross-compilation check is not a runtime test.

## Successful checks

- Windows x64: 1,796 all-feature tests and 1,529 default-feature tests pass, including two new native-call
  tests for direct dispatch and secondary-vtable `this` adjustment. No failed or ignored tests.
- Clippy passes for all targets/features with warnings denied; formatting passes.
- The original C++ comparison still passes. The native OpenGL showcase/swap-callback smoke test passes.
- Rust 1.85 all-feature library checking passes. Linux x64 with hooks and macOS ARM64 default-library
  cross-checks pass. Windows x86 default-library checking passes; enabling hooks fails as described above.

## Reproduce

```sh
python tools/audit_port.py --cpp ../omath --check-x86
```

The audit intentionally exits with status 1 while gaps are confirmed. It writes per-language command streams,
compiler diagnostics, header mappings and a report to `target/port-audit/`. The x86 check requires the Rust
target installed with `rustup target add i686-pc-windows-msvc`. The C++ ImGui API compile check discovers the
local vcpkg header or accepts `--imgui-include <directory>`.

The saved [audit report](validation/port-audit.json), [x86 diagnostics](validation/port-audit-x86.log) and
[missing API diagnostics](validation/port-audit-imgui-api.log) preserve this run. Passing every current probe
would remove these known counterexamples; it would still not constitute exhaustive equivalence proof.
