# omath (Rust)

Rust port of [omath](https://github.com/orange-cpp/omath): math, engine conventions, collision,
navigation, ballistics, HUD, process access, Unreal walkers and optional graphics hooks.
This is an altered version of the original; see [LICENSE](LICENSE) (Zlib).

The [completeness audit](docs/port-audit.md) found remaining API, HUD-parity and Windows x86 build gaps.
The existing passing tests and C++ comparison cover a subset; this is not yet a fully equivalent port.

```rust
use omath::engines::source::{Camera, ViewAngles};
use omath::projection::{FieldOfView, ViewPort};
use omath::Vector3;
let camera = Camera::new(
    Vector3::default(), ViewAngles::default(), ViewPort::new(1920.0, 1080.0),
    FieldOfView::from_degrees(90.0), 0.01, 1000.0,
);
let screen = camera.world_to_screen(Vector3::new(100.0, 0.0, 0.0)).unwrap();
assert!((screen.x - 960.0).abs() < 0.01);
```

## Build and integrations

Rust 1.85 or newer. Cargo replaces the C++ build/package manifests.

```sh
cargo test --locked
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --all-features --no-deps --open
cargo bench --bench kernels
```

| Feature | Integration |
| --- | --- |
| default | All math/engine/collision/navigation/ballistics modules, targeting, HUD, process access, live-module scanning, Unreal walkers and encrypted variables |
| `lua` | Vendored Lua 5.5 bindings, including HUD callbacks and current-process module scans |
| `imgui` | `ImguiHudRenderer`, vector and color conversions |
| `hooks` | DX9/11/12 and window procedures on Windows; OpenGL swaps on Windows/Linux; x86/x64 |

Default dependencies are `memchr` and platform bindings (`windows-sys` or `libc`). Lua/ImGui need a native
compiler for their dependencies. No C++ omath sources are compiled into the Rust crate.

## API

| Module | Provides |
| --- | --- |
| `linear_algebra`, `trigonometry` | Vectors, matrices, quaternions, triangles, angle policies and FOV conversions |
| `projection`, `engines` | Cameras and conventions for Source, IW, Unity, Unreal, Frostbite, RAGE, Serious, CryEngine and OpenGL |
| `primitives`, `collision` | Bounding volumes, meshes, GJK/EPA, ray tracing and colliders |
| `pathfinding` | A*, navigation meshes and WalkBot |
| `projectile_prediction` | Legacy, drag and batched solvers, launchers and flight simulation |
| `algorithm` | Radar and filtered distance/FOV target selection |
| `utility`, `hashing` | Colors, wildcard patterns, PE/ELF/Mach-O file/live-module scans, hashes and Base64 |
| `rev_eng`, `engines::unreal` | Fallible process/object access, ABI-explicit calls, UE 2.5–5 name/object walkers |
| `containers` | Encrypted variables with scoped decryption |
| `hud` | Renderer interface, command buffer, entity/screen widgets, gradients and glow |
| `hooks`, `lua` | Optional integrations above |

See [API and examples](docs/api.md), [compatibility notes](docs/porting.md) and
[validation/performance results](docs/validation.md). Generate full API documentation with `cargo doc`.

## C++ comparison

```sh
python tools/compare_cpp.py --cpp ../omath --repeats 5
```

The tool builds an independent C++ executable against the supplied checkout, compares math and all nine
camera conventions, HUD commands and navigation serialization in both directions, then times matched
vector/matrix/pattern-scan kernels. It writes source hashes, compiler flags and raw results to
`target/cpp-comparison/report.json` without modifying the C++ checkout.

Known upstream behavior is retained where documented, including the five legacy defects in the
[compatibility notes](docs/porting.md). The Windows native Rust/ImGui showcase replaces the upstream Qt/GLFW graphical examples, and the loadable
graphics-hook DLL example provides explicit initialization/removal. Game-specific addresses and loader
integration remain application inputs.
