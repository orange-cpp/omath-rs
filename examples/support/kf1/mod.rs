//! Killing Floor 1 build 1065 example support. All target pointers are 32-bit.
//! The memory backend is explicit so recorded memory images can exercise the same reader as a live game.
#![allow(dead_code)] // Shared example support; each executable uses a different subset.
#[cfg(all(windows, feature = "hooks", feature = "imgui"))]
pub mod dx9_imgui;
pub mod gameplay;
#[cfg(all(windows, feature = "hooks", feature = "imgui"))]
pub mod glow;
pub mod overlay;
pub mod reader;
