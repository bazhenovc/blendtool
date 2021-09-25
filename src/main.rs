// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use structopt::*;

mod dump_blend_structure;
mod dump_irradiance_volumes;

#[derive(Debug, StructOpt)]
struct CommandLineOptions {
    #[structopt(short = "i", long = "input", help = "Input .blend file path", parse(from_os_str))]
    input_file: std::path::PathBuf,

    #[structopt(
        long = "dump-blend-structure",
        help = "Text file to dump .blend file structure",
        parse(from_os_str)
    )]
    dump_blend_structure: Option<std::path::PathBuf>,

    #[structopt(
        long = "dump-irradiance-volumes",
        help = "Folder where to dump irradiance volume textures",
        parse(from_os_str)
    )]
    dump_irradiance_volumes: Option<std::path::PathBuf>,
}

fn main() {
    let command_line = CommandLineOptions::from_args();
    let blend = blend::Blend::from_path(command_line.input_file);

    if let Some(dump_blend_structure) = command_line.dump_blend_structure {
        crate::dump_blend_structure::dump_blend_structure(&blend, &dump_blend_structure);
    }

    if let Some(dump_irradiance_volumes) = command_line.dump_irradiance_volumes {
        crate::dump_irradiance_volumes::dump_irradiance_volumes(&blend, &dump_irradiance_volumes);
    }
}
