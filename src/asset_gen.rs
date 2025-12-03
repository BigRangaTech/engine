use image::{ImageBuffer, Rgba};
use std::fs;
use std::path::{Path, PathBuf};

use ::rand::SeedableRng;
use ::rand::rngs::StdRng;
use ::rand::Rng;
use std::f32::consts::PI;

use crate::generator::GameRules;

pub fn generate_placeholder_assets(
    seed: u64,
    rules: &GameRules,
) -> Result<(), String> {
    let base = Path::new("assets");
    let mut rng = StdRng::seed_from_u64(seed);

    let palettes = pick_theme_colors(&rules.theme);

    ensure_dir(base.join("sprites/player")).map_err(to_string)?;
    ensure_dir(base.join("sprites/enemies")).map_err(to_string)?;
    ensure_dir(base.join("sprites/collectibles")).map_err(to_string)?;
    ensure_dir(base.join("sprites/goals")).map_err(to_string)?;
    ensure_dir(base.join("tiles/platforms")).map_err(to_string)?;
    ensure_dir(base.join("backgrounds")).map_err(to_string)?;
    ensure_dir(base.join("sounds")).map_err(to_string)?;

    // Clamp dimensions to avoid invalid sizes
    let player_size = rules.asset_player_size.max(4);
    let enemy_size = rules.asset_enemy_size.max(4);
    let collectible_size = rules.asset_collectible_size.max(4);
    let goal_size = rules.asset_goal_size.max(4);
    let platform_width = rules.asset_platform_width.max(4);
    let platform_height = rules.asset_platform_height.max(4);
    let bg_width = rules.background_width.max(16);
    let bg_height = rules.background_height.max(16);

    // Players
    save_solid_sprite(
        base.join("sprites/player/player_1.png"),
        player_size,
        player_size,
        random_color(&mut rng, palettes.player_primary, 60),
    )?;
    save_solid_sprite(
        base.join("sprites/player/player_2.png"),
        player_size,
        player_size,
        random_color(&mut rng, palettes.player_secondary, 60),
    )?;

    // Enemies
    save_solid_sprite(
        base.join("sprites/enemies/enemy_1.png"),
        enemy_size,
        enemy_size,
        random_color(&mut rng, palettes.enemy_primary, 50),
    )?;
    save_solid_sprite(
        base.join("sprites/enemies/enemy_2.png"),
        enemy_size,
        enemy_size,
        random_color(&mut rng, palettes.enemy_secondary, 50),
    )?;

    // Collectibles
    save_solid_sprite(
        base.join("sprites/collectibles/collectible_1.png"),
        collectible_size,
        collectible_size,
        random_color(&mut rng, palettes.collectible_primary, 40),
    )?;
    save_solid_sprite(
        base.join("sprites/collectibles/collectible_2.png"),
        collectible_size,
        collectible_size,
        random_color(&mut rng, palettes.collectible_secondary, 40),
    )?;

    // Goal collectibles (for win condition)
    save_solid_sprite(
        base.join("sprites/goals/goal_1.png"),
        goal_size,
        goal_size,
        random_color(&mut rng, palettes.goal_primary, 40),
    )?;
    save_solid_sprite(
        base.join("sprites/goals/goal_2.png"),
        goal_size,
        goal_size,
        random_color(&mut rng, palettes.goal_secondary, 40),
    )?;

    // Platforms
    save_solid_sprite(
        base.join("tiles/platforms/platform_1.png"),
        platform_width,
        platform_height,
        random_color(&mut rng, palettes.platform_primary, 40),
    )?;
    save_solid_sprite(
        base.join("tiles/platforms/platform_2.png"),
        platform_width,
        platform_height,
        random_color(&mut rng, palettes.platform_secondary, 40),
    )?;

    // Backgrounds
    let bg_count = rules.background_variants.max(1);
    for i in 0..bg_count {
        let top = random_color(&mut rng, palettes.bg_top, 30);
        let bottom = random_color(&mut rng, palettes.bg_bottom, 30);
        let filename = format!("backgrounds/bg_{}.png", i + 1);
        save_vertical_gradient(
            base.join(filename),
            bg_width,
            bg_height,
            top,
            bottom,
        )?;
    }

    generate_placeholder_sounds(&base.join("sounds"), rules)?;

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
) -> Result<(), String> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(width, height, |_x, _y| Rgba(color));
    img.save(path).map_err(to_string)?;
    Ok(())
}

fn save_vertical_gradient(
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
    top_color: [u8; 4],
    bottom_color: [u8; 4],
) -> Result<(), String> {
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
    img.save(path).map_err(to_string)?;
    Ok(())
}

fn random_color(rng: &mut StdRng, base: [u8; 3], variance: u8) -> [u8; 4] {
    let mut out = [0u8; 4];
    for i in 0..3 {
        let offset: i16 = rng.gen_range(-(variance as i16)..=(variance as i16));
        let v = base[i] as i16 + offset;
        out[i] = v.clamp(0, 255) as u8;
    }
    out[3] = 255;
    out
}

fn to_string<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn generate_placeholder_sounds(dir: &Path, rules: &GameRules) -> Result<(), String> {
    let jump_path = dir.join("jump.wav");
    let hit_path = dir.join("hit.wav");
    let pickup_path = dir.join("pickup.wav");
    let music_path = dir.join("music.wav");

    write_tone(
        &jump_path,
        rules.jump_sound_freq,
        rules.jump_sound_duration,
        0.6,
    )?;
    write_tone(
        &hit_path,
        rules.hit_sound_freq,
        rules.hit_sound_duration,
        0.7,
    )?;
    write_tone_glissando(
        &pickup_path,
        rules.pickup_sound_start_freq,
        rules.pickup_sound_end_freq,
        rules.pickup_sound_duration,
        0.6,
    )?;
    write_tone_glissando(
        &music_path,
        rules.music_sound_start_freq,
        rules.music_sound_end_freq,
        rules.music_sound_duration,
        0.3,
    )?;

    Ok(())
}

fn write_tone(path: &PathBuf, freq_hz: f32, duration_s: f32, volume: f32) -> Result<(), String> {
    let sample_rate = 44_100u32;
    let total_samples = (duration_s * sample_rate as f32) as u32;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec).map_err(to_string)?;

    for n in 0..total_samples {
        let t = n as f32 / sample_rate as f32;
        let sample = (volume * i16::MAX as f32 * (2.0 * PI * freq_hz * t).sin()) as i16;
        writer.write_sample(sample).map_err(to_string)?;
    }

    writer.finalize().map_err(to_string)?;
    Ok(())
}

fn write_tone_glissando(
    path: &PathBuf,
    start_freq: f32,
    end_freq: f32,
    duration_s: f32,
    volume: f32,
) -> Result<(), String> {
    let sample_rate = 44_100u32;
    let total_samples = (duration_s * sample_rate as f32) as u32;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec).map_err(to_string)?;

    for n in 0..total_samples {
        let t = n as f32 / sample_rate as f32;
        let freq = start_freq + (end_freq - start_freq) * (t / duration_s);
        let sample = (volume * i16::MAX as f32 * (2.0 * PI * freq * t).sin()) as i16;
        writer.write_sample(sample).map_err(to_string)?;
    }

    writer.finalize().map_err(to_string)?;
    Ok(())
}

struct ThemePalettes {
    player_primary: [u8; 3],
    player_secondary: [u8; 3],
    enemy_primary: [u8; 3],
    enemy_secondary: [u8; 3],
    collectible_primary: [u8; 3],
    collectible_secondary: [u8; 3],
    goal_primary: [u8; 3],
    goal_secondary: [u8; 3],
    platform_primary: [u8; 3],
    platform_secondary: [u8; 3],
    bg_top: [u8; 3],
    bg_bottom: [u8; 3],
}

fn pick_theme_colors(theme: &str) -> ThemePalettes {
    match theme.to_lowercase().as_str() {
        "forest" => ThemePalettes {
            player_primary: [40, 160, 80],
            player_secondary: [80, 200, 120],
            enemy_primary: [220, 60, 60],       // red enemies
            enemy_secondary: [180, 40, 40],
            collectible_primary: [80, 160, 255], // blue collectibles
            collectible_secondary: [120, 200, 255],
            goal_primary: [80, 220, 80],        // green superhealth / goals
            goal_secondary: [140, 255, 140],
            platform_primary: [90, 60, 30],
            platform_secondary: [70, 50, 25],
            bg_top: [30, 80, 40],
            bg_bottom: [120, 200, 100],
        },
        "desert" => ThemePalettes {
            player_primary: [200, 160, 80],
            player_secondary: [220, 200, 120],
            enemy_primary: [220, 60, 60],
            enemy_secondary: [180, 40, 40],
            collectible_primary: [80, 160, 255],
            collectible_secondary: [120, 200, 255],
            goal_primary: [80, 220, 80],
            goal_secondary: [140, 255, 140],
            platform_primary: [180, 140, 80],
            platform_secondary: [150, 120, 70],
            bg_top: [240, 210, 140],
            bg_bottom: [220, 180, 110],
        },
        "neon" => ThemePalettes {
            player_primary: [80, 200, 255],
            player_secondary: [255, 80, 200],
            enemy_primary: [255, 80, 80],
            enemy_secondary: [220, 40, 40],
            collectible_primary: [80, 180, 255],
            collectible_secondary: [140, 220, 255],
            goal_primary: [80, 255, 120],
            goal_secondary: [160, 255, 200],
            platform_primary: [80, 80, 80],
            platform_secondary: [120, 120, 120],
            bg_top: [10, 10, 30],
            bg_bottom: [40, 0, 80],
        },
        _ => ThemePalettes {
            player_primary: [40, 120, 220],
            player_secondary: [40, 200, 140],
            enemy_primary: [220, 60, 60],
            enemy_secondary: [180, 40, 40],
            collectible_primary: [80, 160, 255],
            collectible_secondary: [120, 200, 255],
            goal_primary: [80, 220, 80],
            goal_secondary: [140, 255, 140],
            platform_primary: [150, 100, 60],
            platform_secondary: [120, 120, 120],
            bg_top: [30, 60, 160],
            bg_bottom: [150, 200, 240],
        },
    }
}
