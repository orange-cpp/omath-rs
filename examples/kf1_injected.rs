//! KF1 1065 x86 DX9 DLL. See docs/kf1.md for lifecycle and native ABI assumptions.
#[cfg(all(windows, target_arch = "x86", feature = "hooks", feature = "imgui"))]
#[path = "support/kf1/injected.rs"]
mod injected;
#[path = "support/kf1/mod.rs"]
mod kf1;
