// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use scratch_dds::*;

pub fn dump_irradiance_volumes(blend: &blend::Blend, folder_path: &std::path::Path) {
    for inst in blend.get_by_code(*b"OB") {
        let inst_name = inst.get("id").get_string("name");
        let inst_data = inst.get("data");

        if inst_data.type_name == "LightProbe" {
            let probe_type = match inst_data.get_i8("type") {
                0 => "LIGHTPROBE_TYPE_CUBE",
                1 => "LIGHTPROBE_TYPE_PLANAR",
                2 => "LIGHTPROBE_TYPE_GRID",
                t => panic!("Unknown light probe type: {}", t),
            };

            let grid_x = inst_data.get_i32("grid_resolution_x");
            let grid_y = inst_data.get_i32("grid_resolution_y");
            let grid_z = inst_data.get_i32("grid_resolution_z");

            println!(
                "{:?} type: {} resolution: {}x{}x{}",
                inst_name, probe_type, grid_x, grid_y, grid_z,
            );
        }
    }

    for inst in blend.get_all_root_blocks() {
        if inst.type_name == "Scene" {
            let scene_name = inst.get("id").get_string("name");
            let eevee = inst.get("eevee");

            let gi_diffuse_bounces = eevee.get_i32("gi_diffuse_bounces");
            let gi_cubemap_resolution = eevee.get_i32("gi_cubemap_resolution");
            let gi_visibility_resolution = eevee.get_i32("gi_visibility_resolution");

            let output_path = folder_path.join(&scene_name);
            std::fs::create_dir_all(&output_path).expect("Failed to create output directory for irradiance volumes");

            println!(
                "{}: gi_diffuse_bounces: {}, gi_cubemap_resolution: {}, gi_visibility_resolution: {}",
                &scene_name, gi_diffuse_bounces, gi_cubemap_resolution, gi_visibility_resolution
            );

            let light_cache = eevee.get("light_cache_data");
            let light_cache_flag = light_cache.get_i32("flag");

            const LIGHTCACHE_BAKED: i32 = 1 << 0;
            const LIGHTCACHE_CUBE_READY: i32 = 1 << 2;
            const LIGHTCACHE_GRID_READY: i32 = 1 << 3;

            if light_cache_flag & LIGHTCACHE_BAKED == LIGHTCACHE_BAKED {
                if light_cache_flag & LIGHTCACHE_CUBE_READY == LIGHTCACHE_CUBE_READY {
                    let cube_tx = light_cache.get("cube_tx");
                    let tex_size = cube_tx.get_i32_vec("tex_size");
                    let data_type = cube_tx.get_i8("data_type");
                    let components = cube_tx.get_i8("components");

                    let cube_mips = light_cache.get_iter("cube_mips");
                    let mipmap_count = cube_mips.count() + 1;
                    println!(
                        "Baked cubemap texture: {}x{}x{} data_type: {}, components: {}, mips: {}",
                        tex_size[0], tex_size[1], tex_size[2], data_type, components, mipmap_count,
                    );

                    assert_eq!(tex_size[2] % 6, 0, "Not a cube map - slices not divisible by 6");

                    let mut dds = ScratchImage::new(
                        tex_size[0] as _,
                        tex_size[1] as _,
                        1,
                        mipmap_count as _,
                        (tex_size[2] / 6) as _,
                        lightcache_type_to_dxgi(data_type, components),
                        true,
                    );

                    let mut target_slice = dds.as_slice_mut();
                    {
                        let data = cube_tx.get_i8_vec("data");
                        let (dst, remaining) = target_slice.split_at_mut(data.len());
                        dst.copy_from_slice(bytemuck::cast_slice(&data));
                        target_slice = remaining;
                    }

                    let cube_mips = light_cache.get_iter("cube_mips");
                    for mip in cube_mips {
                        let data = mip.get_i8_vec("data");
                        let (dst, remaining) = target_slice.split_at_mut(data.len());
                        dst.copy_from_slice(bytemuck::cast_slice(&data));
                        target_slice = remaining;
                    }

                    if !target_slice.is_empty() {
                        panic!(
                            "Not all data was copied to the image, remaining bytes: {}",
                            target_slice.len()
                        );
                    }

                    dds.write_to_file(&output_path.join("cube_tx.dds"))
                        .expect("Failed to write cube_tx.dds");
                }

                if light_cache_flag & LIGHTCACHE_GRID_READY == LIGHTCACHE_GRID_READY {
                    let grid_tx = light_cache.get("grid_tx");
                    let tex_size = grid_tx.get_i32_vec("tex_size");
                    let data_type = grid_tx.get_i8("data_type");
                    let components = grid_tx.get_i8("components");
                    println!(
                        "Baked grid texture: {}x{}x{} data_type: {}, components: {}",
                        tex_size[0], tex_size[1], tex_size[2], data_type, components
                    );

                    let mut dds = ScratchImage::new(
                        tex_size[0] as _,
                        tex_size[1] as _,
                        1,
                        1,
                        tex_size[2] as _,
                        lightcache_type_to_dxgi(data_type, components),
                        false,
                    );

                    let data = grid_tx.get_i8_vec("data");
                    dds.as_slice_mut().copy_from_slice(bytemuck::cast_slice(&data));

                    dds.write_to_file(&output_path.join("grid_tx.dds"))
                        .expect("Failed to write grid_tx.dds");
                }
            }
        }
    }
}

fn lightcache_type_to_dxgi(data_type: i8, components: i8) -> u32 {
    const LIGHTCACHETEX_BYTE: i8 = 1 << 0;
    const LIGHTCACHETEX_FLOAT: i8 = 1 << 1;
    const LIGHTCACHETEX_UINT: i8 = 1 << 2;

    match data_type {
        LIGHTCACHETEX_BYTE => match components {
            1 => DXGI_FORMAT_R8_UNORM,
            2 => DXGI_FORMAT_R8G8_UNORM,
            4 => DXGI_FORMAT_R8G8B8A8_UNORM,
            x => panic!("Unsupported light cache format with {} components", x),
        },
        LIGHTCACHETEX_FLOAT => match components {
            1 => DXGI_FORMAT_R32_FLOAT,
            2 => DXGI_FORMAT_R32G32_FLOAT,
            3 => DXGI_FORMAT_R32G32B32_FLOAT,
            4 => DXGI_FORMAT_R32G32B32A32_FLOAT,
            x => panic!("Unsupported light cache format with {} components", x),
        },
        LIGHTCACHETEX_UINT => match components {
            1 => DXGI_FORMAT_R11G11B10_FLOAT,
            x => panic!("Unsupported LIGHTCACHETEX_UINT format with {} components", x),
        },
        x => panic!("Unsupported light cache format: {}", x),
    }
}
