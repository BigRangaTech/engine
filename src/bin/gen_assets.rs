use image::{ImageBuffer, Rgba};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = Path::new("assets");

    // Ensure directories exist
    ensure_dir(base.join("sprites/player"))?;
    ensure_dir(base.join("sprites/enemies"))?;
    ensure_dir(base.join("tiles/platforms"))?;
    ensure_dir(base.join("backgrounds"))?;

    // Players
    save_solid_sprite(
        base.join("sprites/player/player_blue.png"),
        32,
        32,
        [60, 160, 255, 255],
    )?;
    save_solid_sprite(
        base.join("sprites/player/player_green.png"),
        32,
        32,
        [60, 220, 120, 255],
    )?;

    // Enemies
    save_solid_sprite(
        base.join("sprites/enemies/enemy_red.png"),
        32,
        32,
        [220, 60, 60, 255],
    )?;
    save_solid_sprite(
        base.join("sprites/enemies/enemy_purple.png"),
        32,
        32,
        [180, 60, 220, 255],
    )?;

    // Platforms
    save_solid_sprite(
        base.join("tiles/platforms/platform_brown.png"),
        64,
        16,
        [140, 90, 40, 255],
    )?;
    save_solid_sprite(
        base.join("tiles/platforms/platform_gray.png"),
        64,
        16,
        [120, 120, 120, 255],
    )?;

    // Backgrounds
    save_vertical_gradient(
        base.join("backgrounds/bg_blue_sky.png"),
        320,
        180,
        [40, 80, 200, 255],
        [180, 220, 255, 255],
    )?;
    save_vertical_gradient(
        base.join("backgrounds/bg_purple_night.png"),
        320,
        180,
        [30, 10, 40, 255],
        [120, 60, 160, 255],
    )?;

    println!("Sample assets generated into the assets/ folder.");
    Ok(())
}

fn ensure_dir(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
    fs::create_dir_all(path)
}

fn save_solid_sprite(
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
    color: [u8; 4],
) -> Result<(), Box<dyn std::error::Error>> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(width, height, |_x, _y| Rgba(color));
    img.save(path)?;
    Ok(())
}

fn save_vertical_gradient(
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
    top_color: [u8; 4],
    bottom_color: [u8; 4],
) -> Result<(), Box<dyn std::error::Error>> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |_x, y| {
        let t = y as f32 / (height.saturating_sub(1).max(1) as f32);
        let lerp = |a: u8, b: u8| -> u8 {
            ((a as f32 * (1.0 - t)) + (b as f32 * t)).round().clamp(0.0, 255.0) as u8
        };
        Rgba([
            lerp(top_color[0], bottom_color[0]),
            lerp(top_color[1], bottom_color[1]),
            lerp(top_color[2], bottom_color[2]),
            lerp(top_color[3], bottom_color[3]),
        ])
    });
    img.save(path)?;
    Ok(())
}

