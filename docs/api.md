# API and examples

`cargo doc --all-features --no-deps --open` builds the Rust API reference.

## Examples

| C++ use case | Rust command |
| --- | --- |
| Barycentric interpolation | `cargo run --example barycentric` |
| Projection matrix builder | `cargo run --example projection_matrix` |
| Signature scanning | `cargo run --example signature_scan -- image.exe "48 8B ??"` |
| HUD layout | `cargo run --example hud` |
| Interactive HUD, barycentric and projection showcase (Windows) | `cargo run --features hooks,imgui --example showcase` |
| Killing Floor 1 object/name dumper | `cargo run --example kf1_dumper -- PID GOBJECTS_HEX GNAMES_HEX` |
| DX9/11/12 or OpenGL hook setup | `cargo run --features hooks --example graphics_hook -- dx11` (or `dx9`, `dx12`, `opengl`) |
| Loadable graphics-hook DLL/shared library | `cargo build --features hooks --example hook_dll` |
| C++ comparison/performance probe | `cargo run --release --example parity_probe` |

The interactive examples use a Rust/ImGui native OpenGL window instead of the upstream Qt/GLFW shells.
The Windows showcase needs no additional GUI framework and offers HUD, barycentric and projection controls.
`--frames=3 --hook-test` runs a hidden render/swap smoke check; `--mode=1` or `--mode=2` selects the other scenes.
The hook DLL exports `omath_set_graphics_callback`, `omath_start_graphics_hooks` and
`omath_stop_graphics_hooks`. Load it using the application's existing loader and initialize after loading,
outside `DllMain`. API modes are 0 OpenGL, 9 DX9, 11 DX11 and 12 DX12. Callback event IDs follow `EventKind`
declaration order; their five raw argument words are borrowed for that call. Thread quiescence and removal
before unloading are caller obligations. This example does not inject into another process.

For game overlays, use `ImguiHudRenderer` in the graphics callback and the Unreal memory/layout APIs from
the dumper example. Game-specific addresses, object fields and the external loader remain application inputs.

## Target selection

Both `get_closest_target_by_distance` and `get_closest_target_by_fov` accept an iterator, a position accessor
and a predicate. Pass `|_| true` for no filter. Returns `Option<Iterator::Item>`; iterate borrowed targets to
return a reference. Ties preserve the first eligible target. FOV uses pitch/yaw and shortest-angle wrapping.

## Process memory and Unreal

`ProcessMemory::open(pid, writable)` owns the native handle where applicable. `current()` reads the current
process. `MemoryRead::read_memory` accepts `MemoryValue`: primitive numbers and arrays are provided. Padded
structs, `bool`, references and invalid enum representations cannot be cast through the safe API. Custom
plain types can implement the documented unsafe trait.

`MemoryView { base, bytes }` is a bounded backend for dumps/tests. Custom `MemoryRead` backends work with the
same Unreal walkers. `ExternalReverseEngineeredObject` reads/writes offsets through its backend.
`InternalReverseEngineeredObject` adds virtual-method resolution and direct/pattern-resolved calls. Callers
specify the exact function-pointer signature and ABI in an unsafe call; x86 `thiscall`/`stdcall` are not inferred.

`NameLayout::{ue2_5, ue3, ue4_legacy, ue4, ue5}` and
`ObjectArrayLayout::{ue2_5, ue3, ue4_fixed, ue4, ue5}` mirror upstream presets. Fields are public because game
builds move members. Walkers take a backend explicitly and report unreadable memory, null tables, invalid
layouts and missing objects. Pointer size refers to the target, not the host.

## Loaded modules

`LoadedModule::list()` enumerates current-process modules; `find("")` selects the executable. Other names
match a path or filename. Pass the returned module to a scanner's `scan_for_pattern_in_loaded_module`.
Results are actual ASLR-adjusted addresses. PE scans `VirtualSize`; Mach-O uses the loader slide; ELF reads
section metadata from the file and scans live bytes, so named sections require an on-disk section table.

Lua `scan_in_module(base, pattern, section?)` accepts an enumerated current-process module base. Unknown
bases return `nil`. File/buffer scanning keeps the existing C++ names and result fields.

## HUD

`HudRenderer` receives drawing primitives; `CommandBuffer` records them. `EntityOverlay` stores layout/side
cursors; `ScreenOverlay` stores dimensions. Pass a mutable renderer to `contents`, `draw` or `add_side_widget`
instead of retaining shared renderer ownership. `EntityWidget`, `SideWidget` and `ScreenWidget` replace C++
variant/template dispatch. `widget::when` returns an `Option`; flatten optional descriptors for `contents`.

Widgets include solid/corner/dashed boxes, skeletons, snap lines, markers, aim indicators, side bars, labels,
icons, rings and spacers, plus crosshairs, FOV circles, threat arrows, hit markers, center bars and anchors.
`Paint` supports colors/corner gradients; `BarPaint` also blends empty/full colors with the ratio. Glow and
dash helpers are public. Invalid dash lengths draw nothing rather than looping indefinitely.

With `imgui`, construct `ImguiHudRenderer::new(ui)` during a frame; texture handles use `TextureId::id()`.
Lua exposes `omath.hud.CanvasBox`, `Renderer`, `EntityOverlay` and `ProjectileAimFigure`, upstream imperative
methods and chaining. `Renderer.new(callbacks)` invokes optional functions with the C++ arguments; absent
text measurement returns zero. Errors propagate to Lua; reentrant mutable access is rejected. Texture
objects preserve their Lua identity during rendering.

## Encrypted variables

`EncryptedVariable::new(value, key)` uses the upstream byte XOR schedule. `drop_anchor()` returns a mutable
guard which decrypts and re-encrypts on drop, including unwinding. Encrypted storage is untyped bytes.
This is reversible obfuscation, not cryptography. `encrypted_variable!(value, key_size)` supplies a
deterministic source-location key.

## Graphics hooks

`HooksManager::get()` is process-global. Windows `hook_dx9`, `hook_dx11` and `hook_dx12` discover entry points
with temporary hidden devices. Existing device/swap-chain/queue pointers may be supplied instead.
OpenGL intercepts WGL/GDI swaps on Windows, suppressing nested duplicates, or Linux `glXSwapBuffers`.
Window-procedure interception supports an optional intercepted result.

Use typed `set_on_*` callbacks or `set_callback(EventKind, ...)`. Callbacks run before originals, outside
callback locks; panics are contained at the native boundary. `retour` supplies instruction relocation and
trampolines; the manager, discovery and callbacks are implemented in Rust.

Installing/removing inline hooks is unsafe: threads must be outside the affected originals/detours. Do not
unhook from a callback. Window-procedure changes belong on the window's owner thread; remove before window
destruction. This API does not promise automatic thread suspension. Hooks support x86/x64 Windows/Linux;
the rest of the crate also supports macOS.
