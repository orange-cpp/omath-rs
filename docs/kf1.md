# Killing Floor 1 examples

These examples target the same **KF1 build 1065, 32-bit UE2.5** layout as the frozen C++ reference.
Core.dll's GObjects/GNames RVAs and Engine.dll's audio-listener RVA are build-specific; reflected property
offsets are resolved by name. Both external and injected observers share the same reader and actor tracker.

```sh
cargo run --features hooks --example kf1_dumper
cargo run --features hooks,imgui --example kf1_overlay
cargo build --features hooks,imgui --target i686-pc-windows-msvc --example kf1_injected
cargo run --all-features --example kf1_glow_smoke
```

The dumper discovers KillingFloor.exe/Core.dll/Engine.dll, reports reflected classes, local player, view and
properties, then dumps objects. An optional first argument filters names/classes. The external observer is
read-only and aligns its transparent native window to the game's client area. INSERT toggles its settings;
END closes it. Both require a running game with the expected layout and readable process memory.

The injected example builds `target/i686-pc-windows-msvc/debug/examples/kf1_injected.dll`.
An existing application loader must load it; the example supplies no injector. Call `omath_kf1_start` after
loading, outside the loader lock, on the game window's owner/render thread while affected functions are
quiescent. Return values are 0 success, 1 already started, -1 initialization failed. Initialization rolls
back its hooks on failure. All render and engine callbacks are expected on that same thread, as in this
single-threaded game renderer. Do not load a second active ImGui context on the same thread.

Call `omath_kf1_stop` on that same thread outside callbacks, with affected functions quiescent, before
unloading. It restores the window procedure, native detours, controller rotation and modified weapon
values, then frees renderer state. It returns 0 on success, -1 on failure. END disables the injected
features and restores edits/resources; the caller still needs to stop successfully before unloading.
The DLL performs no work in DllMain.

The injected observer contains:

- Incremental GObjects scanning, class-cache reuse detection, local-player rediscovery, reflected health,
  collision bounds, player names and held-weapon association; stale/dead objects are filtered.
- Projected boxes, health bars, names, health/distance text, radar, origin/status display and settings.
- Actual DX9 mask, separable blur and premultiplied-composite shaders for zeds, allies and ally weapons.
  Mesh/level hooks select silhouette groups; DrawIndexedPrimitive is repeated into offscreen targets.
- UGameEngine::Draw restoration/aim timing, native animated head-bone lookup, collision-box fallback,
  engine line traces, cone/range filtering, smoothing and optional silent controller rotation.
- Reflection-based recoil/spread edits with saved originals and class-identity checks on restoration.

Stock ImGui replaces the C++ example's external nullcore menu, icons and visual styling. Settings and game
data use the shared Rust implementation. The external shell uses Win32/OpenGL with color-key transparency
instead of GLFW's transparent framebuffer. The injected shell uses a Rust fixed-function DX9 ImGui backend;
it expands indexed triangles into DrawPrimitiveUP calls and recreates its font texture after Reset.
EPA scratch allocation elsewhere in the library uses a reusable workspace rather than C++ PMR.

Memory-image tests cover reflection, camera discovery, class-address reuse, living-actor filtering,
target selection, tick-input preservation and restoring weapon edits. The real DX9 smoke test compiles
all shaders, checks output pixels and device state, and recreates ImGui resources after a real Reset.
The DLL also builds on x86. **A live KF1 session was not available for end-to-end testing**: native game
exports, bone/trace calls, menu interaction and in-game hook timing remain runtime validation work.
