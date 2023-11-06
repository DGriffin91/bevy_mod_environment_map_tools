use std::path::Path;

use bevy::{prelude::Image, render::render_resource::Extent3d};
use ktx2::SupercompressionScheme;
use ktx2_writer::{Header, KTX2Writer, WriterLevel};
use rgb9e5::float3_to_rgb9e5;

pub mod ktx2_writer;
pub mod rgb9e5;

pub fn to_vec_f16_from_byte_slice(vecs: &[u8]) -> &[half::f16] {
    unsafe { std::slice::from_raw_parts(vecs.as_ptr() as *const _, vecs.len() / 2) }
}

pub fn u32_to_bytes(vecs: &[u32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(vecs.as_ptr() as *const _, vecs.len() * 4) }
}

pub fn write_ktx2(image: &Image, output_path: &Path) {
    if image.is_compressed() {
        panic!("Only uncompressed images supported");
    }

    let mut mips = Vec::new();
    for mip_level in 0..image.texture_descriptor.mip_level_count {
        let mut rgb9e5 = Vec::new();
        for face in 0..6 {
            let mip_data = extract_mip_level(image, mip_level, face);
            let f16data = to_vec_f16_from_byte_slice(&mip_data.data);

            for v in f16data.chunks(4) {
                rgb9e5.push(float3_to_rgb9e5(&[
                    v[0].to_f32(),
                    v[1].to_f32(),
                    v[2].to_f32(),
                ]));
            }
        }

        let rgb9e5_bytes = u32_to_bytes(&rgb9e5).to_vec();
        mips.push(WriterLevel {
            uncompressed_length: rgb9e5_bytes.len(),
            bytes: zstd::bulk::compress(&rgb9e5_bytes, 0).unwrap(),
        });
    }

    // https://github.khronos.org/KTX-Specification/
    let writer = KTX2Writer {
        header: Header {
            format: Some(ktx2::Format::E5B9G9R9_UFLOAT_PACK32),
            type_size: 4,
            pixel_width: image.texture_descriptor.size.width,
            pixel_height: image.texture_descriptor.size.height,
            pixel_depth: 1,
            layer_count: 1,
            face_count: 6,
            supercompression_scheme: Some(SupercompressionScheme::Zstandard),
        },
        dfd_bytes: u32_to_bytes(&[0u32, 0, 2]),
        levels_descending: mips,
    };

    writer
        .write(&mut std::fs::File::create(output_path).unwrap())
        .unwrap();
}

/// Extract a specific individual mip level as a new image.
pub fn extract_mip_level(image: &Image, mip_level: u32, face: u32) -> Image {
    let descriptor = &image.texture_descriptor;

    if descriptor.mip_level_count < mip_level {
        panic!(
            "Mip level {mip_level} requested, but only {} are avaliable.",
            descriptor.mip_level_count
        );
    }

    let block_size = descriptor.format.block_size(None).unwrap() as usize;

    let mut byte_offset = 0usize;

    for _ in 0..face {
        let mut width = descriptor.size.width as usize;
        let mut height = descriptor.size.height as usize;
        for _ in 0..descriptor.mip_level_count {
            byte_offset += width * block_size * height;
            width /= 2;
            height /= 2;
        }
    }

    let mut width = descriptor.size.width as usize;
    let mut height = descriptor.size.height as usize;

    for _ in 0..mip_level {
        byte_offset += width * block_size * height;
        width /= 2;
        height /= 2;
    }

    let mut new_descriptor = descriptor.clone();

    new_descriptor.mip_level_count = 1;
    new_descriptor.size = Extent3d {
        width: width as u32,
        height: height as u32,
        depth_or_array_layers: 1,
    };

    Image {
        data: image.data[byte_offset..byte_offset + (width * block_size * height)].to_vec(),
        texture_descriptor: new_descriptor,
        sampler: image.sampler.clone(),
        texture_view_descriptor: image.texture_view_descriptor.clone(),
    }
}
