# Porting and compatibility

No C++ omath sources are compiled into the Rust library. Native Lua/ImGui dependencies are separate.
The C++ checkout is not modified by this port or the comparison tool.

## Language adaptations

- Storage order and angle ranges use marker types because stable Rust has no floating-point const generics
  or generic `[T; R * C]` storage. Construct matrices with `from_rows`.
- `const fn` covers hashing, compile-time pattern parsing and key generation. Generic floating-point
  operations run at runtime on stable Rust.
- LLVM vectorizes matrix/batched-solver loops. `ProjPredEngineAvx2` becomes `ProjPredEngineBatched`, without
  promising an AVX2-specific implementation. Pattern scans use SIMD-capable `memchr` anchors.
- Vectors expose `cmp_length`, preserving consistency between Rust equality and ordering.
- HUD descriptors and explicit renderer parameters replace variadic dispatch/shared renderer ownership.
- The Rust crate-import integration test replaces the C++20 module test. Cargo replaces CMake, vcpkg,
  xmake and pixi; the lockfile pins dependencies for reproducible checks.
- Rust/ImGui replaces the Qt/GLFW graphical example shells on Windows; portable CLI examples remain available.
  A loadable graphics-hook DLL example supplies explicit lifecycle exports rather than work in `DllMain`.
  KF1 build 1065 examples include their addresses and reflection; loader integration remains an application input.
- `EpaWorkspace` and `epa_solve_with_workspace` retain scratch capacity between calls in place of C++ PMR.
  Horizon edges retain deterministic sorted order. Native allocator implementations and EPA traversal
  order are not promised to match; penetration results are compared separately.

## Intentional corrections

Existing corrections cover the camera-cached radar sign, invalid solver-step hangs, A* unknown-neighbor
crashes, undefined perpendicular-vector construction and malformed-binary reads. New memory operations
are fallible and validate pointer widths and walker layout divisors/shifts. Wide Unreal names decode
surrogate pairs. Encrypted storage is untyped while obfuscated, and guards cannot outlive the owner.

FOV targeting uses consistent pitch/yaw dimensions and shortest-angle wrapping. The C++ template does compile:
`Vector3` inherits `Vector2`, allowing the implicit base conversion. It measures the current candidate in two
dimensions and the previous best in three, including camera roll. Rust intentionally changes this behavior;
the wraparound and roll cases are demonstrated in the [completeness audit](port-audit.md).

## Retained upstream defects

These remain compatible with existing consumers:

| Behavior | Retained result |
| --- | --- |
| RAGE unit factors | The upstream reversed centimeter conversion factors |
| Triangle intersections | Infinite rays rather than truncation at `Ray::end` |
| `create_plane` | Degenerate first triangle indices `(1, 1, 2)` |
| Default depth ranges | The upstream mismatch between some formula defaults and camera conventions; explicit ranges remain available |
| `WalkBot::reset` | Clears visited-node state and preserves the target |

Changing these requires a compatibility decision and updated consumers. Tests distinguish port correctness
from known upstream behavior.

## Platform validation

The [completion audit](port-audit.md) records successful Windows x64 and x86 all-feature tests, native
graphics checks and DLL builds. The x86 WndProc binding widths are fixed. Navigation text parses neighbor
counts as u64 on both architectures, so oversized missing-neighbor records fail consistently.

Local tests run on Windows x64, including live process/module access, synthetic native detours, ImGui
geometry and Lua HUD errors. DX9/11/12/OpenGL discovery/install/remove were exercised against the local
graphics runtime. The native showcase also renders all three scenes and verifies actual OpenGL swap callbacks.
Full game overlays were not exercised. DX9 shader/composite and ImGui drawing/reset smoke tests pass on
both Windows architectures. Linux x64 default
and hook-enabled libraries and the macOS ARM64 default library passed cross-compilation checks.
Linux/macOS runtime tests are configured in
CI and were not run locally.

The user's C++ checkout contains existing uncommitted edits and changed during validation. A frozen copy
under `target/omath-reference` supplies this run's reference. Reports record its revision, dirty files and
SHA-256 source/header hashes, and the Rust working-source hashes. The committed upstream revision was separately extracted
and passed the same math/camera/HUD comparisons. CI pins that revision.
