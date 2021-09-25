// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use blend::*;

pub fn dump_blend_structure(blend: &Blend, file_path: &std::path::Path) {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .expect("Failed to open file for writing");
    for inst in blend.get_all_root_blocks() {
        writeln!(&mut file, "{:?}", inst).expect("Failed to write data to file");
    }
}
