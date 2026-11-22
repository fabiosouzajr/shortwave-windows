// Shortwave - build.rs
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

//! Embeds the Windows resources (icon, version information, manifest).
//! Does nothing on other platforms.

fn main() {
    #[cfg(windows)]
    windows::embed_resources();
}

#[cfg(windows)]
mod windows {
    use std::fs;

    const ICON: &str = "build-aux/windows/shortwave.ico";
    const MANIFEST: &str = "build-aux/windows/shortwave.exe.manifest";

    /// The release version lives in meson.build (Cargo.toml says 0.0.0).
    fn meson_version() -> String {
        let meson = fs::read_to_string("meson.build").unwrap_or_default();
        meson
            .lines()
            .find_map(|line| {
                let rest = line.trim().strip_prefix("version:")?;
                Some(rest.trim().trim_matches(|c| c == '\'' || c == ',').to_string())
            })
            .unwrap_or_else(|| "0.0.0".to_string())
    }

    /// "5.0.0-win.1" -> 0x0005_0000_0000_0001 ("5.0.0" -> ..._0000) for the
    /// binary VERSIONINFO fields: the first four numbers, in order.
    fn numeric_version(version: &str) -> u64 {
        let mut parts = version
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|p| p.parse::<u64>().ok());
        let mut value = 0;
        for shift in [48, 32, 16, 0] {
            value |= (parts.next().unwrap_or(0) & 0xffff) << shift;
        }
        value
    }

    pub fn embed_resources() {
        println!("cargo:rerun-if-changed={ICON}");
        println!("cargo:rerun-if-changed={MANIFEST}");
        println!("cargo:rerun-if-changed=meson.build");

        let version = meson_version();
        let mut res = winresource::WindowsResource::new();
        res.set_icon(ICON)
            .set_manifest_file(MANIFEST)
            .set("ProductName", "Shortwave")
            .set("FileDescription", "Shortwave")
            .set("InternalName", "shortwave")
            .set("OriginalFilename", "shortwave.exe")
            .set("CompanyName", "Unofficial Windows port of GNOME Shortwave")
            .set(
                "LegalCopyright",
                "Copyright (C) Felix Häcker and contributors. GPL-3.0-or-later",
            )
            .set("ProductVersion", &version)
            .set("FileVersion", &version)
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, numeric_version(&version))
            .set_version_info(winresource::VersionInfo::FILEVERSION, numeric_version(&version));

        if let Err(err) = res.compile() {
            panic!("Unable to embed Windows resources: {err}");
        }
    }
}
