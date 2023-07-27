use image::{ImageBuffer, Rgba};

use crate::Error;

// Index directly corresponds to value in map
pub const TEXTURE_WIDTH: usize = 512;
pub const N_TEXTURES: usize = 9;
const TEXTURES_TO_LOAD: [&str; N_TEXTURES] = ["DUNGEONBRICKS.png", "DUNGEONCELL.png", "SPOOKYDOOR.png", "CROSSCUBE.png", "OFFICEDOOR.png", "PIPES.png", "ROUNDBRICKS.png", "LAVAROCKS.png", "GRAYWALL.png"];

fn load_resize_png_to_u32_array(file_name: &str) -> Result<Box<[u32]>, Error> {
    let env = std::env::current_dir().unwrap();
    let img_path = env.to_str().unwrap().to_owned() + "\\img\\" + file_name;

    // Load the image from the given file path
    let img = image::open(img_path)?;

    // Resize the image
    let resized_img = img.resize_exact(TEXTURE_WIDTH.try_into()?, TEXTURE_WIDTH.try_into()?, image::imageops::FilterType::Nearest);

    // Convert the image to a buffer of RGBA format
    let rgba_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = resized_img.to_rgba8();

    // Convert the RGBA buffer to an array of u32 in the format RGBA,
    // 1 byte per channel
    // We create a boxed array since it's possible for the size of
    // the texture to exceed the size of the stack.
    let mut u32_array = vec![0_u32; TEXTURE_WIDTH * TEXTURE_WIDTH].into_boxed_slice();
    for (i, pixel) in rgba_buffer.pixels().enumerate() {
        let [r, g, b, a] = pixel.0;
        let rgba_value: u32 = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32;
        u32_array[i] = rgba_value;
    }

    Ok(u32_array)
}

pub fn load_textures() -> Result<Box<[Box<[u32]>]>, Error>  {
    let mut textures = vec![
        vec![0_u32; TEXTURE_WIDTH * TEXTURE_WIDTH].into_boxed_slice(); N_TEXTURES
    ].into_boxed_slice();

    for (i, texture) in TEXTURES_TO_LOAD.iter().enumerate() {
        let result = load_resize_png_to_u32_array(texture);
        match result {
            Ok(u32_array) => {textures[i] = u32_array}
            Err(err) => {
                println!("WARN: Could not load texture '{}' - {:?}", texture, err)
            }
        }
    }

   Ok(textures)
}