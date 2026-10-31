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
  Game-specific addresses and loader integration remain application inputs. See `api.md`.

## Intentional corrections

Existing corrections cover the camera-cached radar sign, invalid solver-step hangs, A* unknown-neighbor
crashes, undefined perpendicular-vector construction and malformed-binary reads. New memory operations
are fallible and validate pointer widths and walker layout divisors/shifts. Wide Unreal names decode
surrogate pairs. Encrypted storage is untyped while obfuscated, and guards cannot outlive the owner.

FOV targeting uses consistent pitch/yaw dimensions and shortest-angle wrapping. The upstream template
mixes vector dimensions and is not a usable literal implementation to compare against.

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

Local tests run on Windows x64, including live process/module access, synthetic native detours, ImGui
geometry and Lua HUD errors. DX9/11/12/OpenGL discovery/install/remove were exercised against the local
graphics runtime. The native showcase also renders all three scenes and verifies actual OpenGL swap callbacks.
Full game overlays were not exercised. Linux x64 default
and hook-enabled libraries and the macOS ARM64 default library passed cross-compilation checks.
Linux/macOS runtime tests are configured in
CI and were not run locally.

The user's C++ checkout contains existing uncommitted edits. The comparison report records its revision,
dirty files and SHA-256 source/header hashes. The committed upstream revision was separately extracted
and passed the same math/camera/HUD comparisons. CI pins that revision.
