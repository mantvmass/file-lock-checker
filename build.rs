// Copyright (c) 2025 Phumin Maliwan
// SPDX-License-Identifier: MIT

use cargo_toml::Manifest;
use std::fs;

fn main() {
    if cfg!(target_os = "windows") {
        let manifest = Manifest::from_path("./Cargo.toml").expect("Failed to read Cargo.toml");
        let version = match manifest.package.expect("No package in Cargo.toml").version {
            cargo_toml::Inheritable::Set(v) => v,
            cargo_toml::Inheritable::Inherited => {
                panic!("Version is inherited, but a specific version is required")
            }
        };
        let numeric_version = version.replace(".", ",") + ",0"; // e.g., "1.0.0" -> "1,0,0,0"

        // Read the app.rc file
        let app_rc_content = fs::read_to_string("res/app.rc")
            .expect("Failed to read res/app.rc");

        // Create the content for tmp.rc with version replacements
        let tmp_rc_content = app_rc_content
            .replace("{{VERSION}}", &format!("{}.0", version))
            .replace("{{NUMERIC_VERSION}}", &numeric_version);

        // Write to tmp.rc
        fs::write("res/tmp.rc", tmp_rc_content)
            .expect("Failed to write res/tmp.rc");

        // Compile the tmp.rc file
        let _ = embed_resource::compile("res/tmp.rc", embed_resource::NONE);
    }
}