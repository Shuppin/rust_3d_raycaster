use image::{ImageBuffer, Rgba};

// Index directly corresponds to value in map
pub const TEXTURE_WIDTH: u32 = 512;
const TEXTURES_TO_LOAD: [&str; 9] = ["DUNGEONBRICKS.png", "DUNGEONCELL.png", "SPOOKYDOOR.png", "CROSSCUBE.png", "OFFICEDOOR.png", "PIPES.png", "ROUNDBRICKS.png", "LAVAROCKS.png", "GRAYWALL.png"];

fn load_resize_png_to_u32_array(file_name: &str) -> Option<Vec<u32>> {
    let env = std::env::current_dir().unwrap();
    let img_path = env.to_str().unwrap().to_owned() + "\\img\\" + file_name;

    // Load the image from the given file path
    let img = match image::open(img_path) {
        Ok(img) => img,
        Err(_) => return None,
    };

    // Resize the image
    let resized_img = img.resize_exact(TEXTURE_WIDTH, TEXTURE_WIDTH, image::imageops::FilterType::Nearest);

    // Convert the image to a buffer of RGBA format
    let rgba_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = resized_img.to_rgba8();

    // Convert the RGBA buffer to an array of u32 in the format RGBA, 1 byte per channel
    let mut u32_array: Vec<u32> = Vec::with_capacity(512 * 512);
    for pixel in rgba_buffer.pixels() {
        let [r, g, b, a] = pixel.0;
        let rgba_value: u32 = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32;
        u32_array.push(rgba_value);
    }

    Some(u32_array)
}

pub fn load_textures() -> Vec<Vec<u32>> {
    let mut textures = vec![];

    for texture in TEXTURES_TO_LOAD {
        let result = load_resize_png_to_u32_array(texture);
        match result {
            Some(u32_array) => {textures.push(u32_array)}
            None => {
                panic!("Could not load texture '{}'", texture)
            }
        }
    }

    return textures
}