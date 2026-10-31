# omath (Rust)

A Rust port of the general-purpose parts of [omath](https://github.com/orange-cpp/omath), a C++ math and game-physics
library. This is an altered version of the original, not the original itself; see [LICENSE](LICENSE) (Zlib).

## Example

```rust
use omath::engines::source::{Camera, PitchAngle, RollAngle, ViewAngles, YawAngle};
use omath::projection::{FieldOfView, ViewPort};
use omath::{Vector3, Vector};

let a = Vector3::new(1.0_f32, 2.0, 3.0);
let b = Vector3::new(4.0_f32, 5.0, 6.0);
assert_eq!(a.dot(b), 32.0);
assert_eq!(a.cross(b), Vector3::new(-3.0, 6.0, -3.0));

let angles = ViewAngles::new(
    PitchAngle::from_degrees(0.0),
    YawAngle::from_degrees(0.0),
    RollAngle::from_degrees(0.0),
);
let camera = Camera::new(
    Vector3::default(),
    angles,
    ViewPort::new(1920.0, 1080.0),
    FieldOfView::from_degrees(90.0),
    0.01,
    1000.0,
);

let screen = camera.world_to_screen(Vector3::new(100.0, 0.0, 0.0)).unwrap();
assert!((screen.x - 960.0).abs() < 0.01);
```

## Contents

| Module | Provides |
| --- | --- |
| `linear_algebra` | `Vector2/3/4`, `Mat<R, C, T, S>`, `Quaternion`, `Triangle` |
| `trigonometry` | `Angle<T, R>`, `ViewAngles`, FOV conversions |
| `projection` | Generic `Camera<E>`, `CameraEngine`, `FieldOfView`, `ViewPort` |
| `engines` | Conventions of Source, IW, Unity, Unreal, Frostbite, RAGE, Serious, CryEngine and OpenGL |
| `primitives` | `Aabb`, `Obb`, `Mesh`, box and plane builders |
| `collision` | GJK, EPA, `Simplex`, ray tracing, `MeshCollider` |
| `pathfinding` | A*, `NavigationMesh`, `WalkBot` |
| `projectile_prediction` | Legacy, drag and batched solvers, `ProjectileFlight` |
| `algorithm` | `world_to_radar` |
| `utility` | `Color`, wildcard byte patterns, PE/ELF/Mach-O section scanning of files |
| `hashing` | FNV-1a, CRC-32, Base64 |
| `lua` | Lua bindings, behind the optional `lua` feature |

There are no dependencies unless you enable `lua`.

## Differences from the C++ library

- **Left out on purpose:** DirectX/OpenGL hooks, `rev_eng` process memory access, the Unreal `GObjects`/`GNames` walkers,
  scanning of modules loaded in a running process, the HUD entity overlay, `algorithm::targeting`, and
  `EncryptedVariable`. The examples are not ported either.
- **Types:** storage order and angle ranges are marker types (`RowMajor`, `angle_range!`), because stable Rust has no
  float const generics or `[T; R * C]`. `Mat` is built from rows with `from_rows`.
- **`constexpr`:** not reproduced, apart from `const fn` hashing and `consteval_pattern!`.
- **SIMD:** matrix and batched-solver loops rely on LLVM's auto-vectorization instead of AVX2 intrinsics.
  `ProjPredEngineAvx2` is named `ProjPredEngineBatched`.
- **Ordering:** vectors have `cmp_length` instead of `<`, because `PartialOrd` would have to agree with `==`.
- **Fixed C++ defects:** the radar sign cached across cameras, hangs on invalid solver steps, a crash in A* on unknown
  neighbors, undefined behaviour in `Simplex::any_perp`, and unchecked reads when parsing malformed binaries.
