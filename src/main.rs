#![allow(unstable_features)]
#![feature(portable_simd)]
use image::{RgbImage};
use std::simd::prelude::*;
use std::time::Instant;
use std::env;
use std::path::Path;

pub fn blur_rgb_scalar(input: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut output = vec![0u8; input.len()];

    // Copy border pixels directly (no blur)
    for y in 0..height {
        for x in 0..width {
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                let idx = (y * width + x) * 3;
                output[idx..idx + 3].copy_from_slice(&input[idx..idx + 3]);
            }
        }
    }

    // Process inner pixels with blur
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut r = 0u16;
            let mut g = 0u16;
            let mut b = 0u16;

            for dy in -1..=1 {
                for dx in -1..=1 {
                    let px = ((y as isize + dy) * width as isize + (x as isize + dx)) as usize;
                    r += input[3 * px + 0] as u16;
                    g += input[3 * px + 1] as u16;
                    b += input[3 * px + 2] as u16;
                }
            }

            let out_idx = (y * width + x) * 3;
            output[out_idx + 0] = (r / 9) as u8;
            output[out_idx + 1] = (g / 9) as u8;
            output[out_idx + 2] = (b / 9) as u8;
        }
    }

    output
}

pub fn blur_rgb_simd(input: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut output = vec![0u8; input.len()];
    let lane = 16;

    // Copy border pixels directly (no blur)
    for y in 0..height {
        for x in 0..width {
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                let idx = (y * width + x) * 3;
                output[idx..idx + 3].copy_from_slice(&input[idx..idx + 3]);
            }
        }
    }

    // Process inner pixels with SIMD
    for y in 1..height - 1 {
        // SIMD processing for bulk of the row
        for x in (1..width - 1 - lane).step_by(lane) {
            let mut r_sum = Simd::<u16, 16>::splat(0);
            let mut g_sum = Simd::<u16, 16>::splat(0);
            let mut b_sum = Simd::<u16, 16>::splat(0);

            for dy in -1..=1 {
                for dx in -1isize..=1 {
                    let base_idx = ((y as isize + dy) * width as isize + (x as isize + dx)) as usize * 3;
                    
                    // Load 16 pixels at once for each channel
                    let mut r = [0u8; 16];
                    let mut g = [0u8; 16];
                    let mut b = [0u8; 16];

                    // Unroll the inner loop for better performance
                    #[allow(clippy::needless_range_loop)]
                    for i in 0..16 {
                        let px = base_idx + i * 3;
                        r[i] = input[px];
                        g[i] = input[px + 1];
                        b[i] = input[px + 2];
                    }

                    // Convert to SIMD and accumulate
                    r_sum += Simd::from_array(r).cast();
                    g_sum += Simd::from_array(g).cast();
                    b_sum += Simd::from_array(b).cast();
                }
            }

            // Calculate averages
            let r_avg: [u8; 16] = (r_sum / Simd::splat(9)).cast().to_array();
            let g_avg: [u8; 16] = (g_sum / Simd::splat(9)).cast().to_array();
            let b_avg: [u8; 16] = (b_sum / Simd::splat(9)).cast().to_array();

            // Store results
            for i in 0..lane {
                let out = (y * width + x + i) * 3;
                output[out] = r_avg[i];
                output[out + 1] = g_avg[i];
                output[out + 2] = b_avg[i];
            }
        }

        // Scalar fallback for remaining pixels
        for x in (width - 1 - lane)..width - 1 {
            let mut r = 0u16;
            let mut g = 0u16;
            let mut b = 0u16;

            for dy in -1..=1 {
                for dx in -1..=1 {
                    let px = ((y as isize + dy) * width as isize + (x as isize + dx)) as usize;
                    r += input[3 * px] as u16;
                    g += input[3 * px + 1] as u16;
                    b += input[3 * px + 2] as u16;
                }
            }

            let out = (y * width + x) * 3;
            output[out] = (r / 9) as u8;
            output[out + 1] = (g / 9) as u8;
            output[out + 2] = (b / 9) as u8;
        }
    }

    output
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <image_path>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }

    let img = image::open(input_path).expect("Failed to open image").to_rgb8();
    let (width, height) = img.dimensions();
    let width = width as usize;
    let height = height as usize;
    let input = img.as_raw();

    // Generate output filename
    let input_path = Path::new(input_path);
    let output_path = input_path.with_file_name(format!(
        "{}_blurred{}",
        input_path.file_stem().unwrap().to_str().unwrap(),
        input_path.extension().map_or("".to_string(), |ext| format!(".{}", ext.to_str().unwrap()))
    ));

    // Benchmark scalar vs SIMD
    let start = Instant::now();
    let scalar_result = blur_rgb_scalar(input, width, height);
    let scalar_time = start.elapsed();

    let start = Instant::now();
    let simd_result = blur_rgb_simd(input, width, height);
    let simd_time = start.elapsed();

    println!("Scalar time: {:?}", scalar_time);
    println!("SIMD time: {:?}", simd_time);
    println!("Speedup: {:.2}x", scalar_time.as_secs_f64() / simd_time.as_secs_f64());

    // Save SIMD result
    let out_img_scalar = RgbImage::from_raw(width as u32, height as u32, scalar_result).unwrap();
    out_img_scalar.save(input_path.with_file_name(format!(
        "{}_blurred_scalar{}",
        input_path.file_stem().unwrap().to_str().unwrap(),
        input_path.extension().map_or("".to_string(), |ext| format!(".{}", ext.to_str().unwrap()))
    ))).unwrap_or_else(|e| {
        eprintln!("Failed to save scalar output image: {}", e);
        std::process::exit(1);
    });
    let out_img_simd = RgbImage::from_raw(width as u32, height as u32, simd_result).unwrap();
    out_img_simd.save(&output_path).unwrap_or_else(|e| {
        eprintln!("Failed to save SIMD output image: {}", e);
        std::process::exit(1);
    });

    println!("Blurred image saved as: {}", output_path.display());
}
