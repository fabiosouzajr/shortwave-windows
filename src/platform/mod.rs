// Shortwave - platform/mod.rs
// Copyright (C) 2026  The Shortwave Windows port contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Platform integration layer.
//!
//! Everything that differs between the upstream (Linux / freedesktop) build
//! and the Windows port lives behind this module. Both backends expose the
//! same set of items:
//!
//! - `start_media_controls()`: MPRIS on Linux, SMTC on Windows
//! - `BackgroundIntegration`: background portal on Linux, tray icon on Windows
//! - `background_permissions()`: whether the app may keep playing with no window
//! - `load_texture()`: decode a downloaded station cover
//!
//! The non-Windows backend keeps upstream's code unchanged, so the Linux build
//! behaves exactly as before.

#[cfg(not(windows))]
#[path = "linux.rs"]
mod backend;

#[cfg(windows)]
#[path = "windows.rs"]
mod backend;

pub use backend::*;
