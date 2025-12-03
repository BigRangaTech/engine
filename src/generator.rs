use crate::assets::{Assets, SpriteKind};
use crate::scene::{Entity, EntityKind, Platform, Scene, InputConfig};
use macroquad::prelude::*;
use ::rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameRules {
    pub seed: Option<u64>,
    pub mode: String,
    pub control_scheme: String,
    pub enemy_enabled: bool,
    pub collectible_enabled: bool,
    pub sfx_enabled: bool,
    pub music_enabled: bool,
    pub show_fps: bool,
    pub vsync_enabled: bool,
    pub resolution_index: usize,
    pub debug_overlay: bool,
    pub collectibles_for_level_up: u32,
    pub max_level: u32,
    pub player_start_health: u32,
    pub player_max_health: u32,
    pub enemy_contact_damage: u32,
    pub hit_invincibility_duration: f32,
    pub hit_flash_enabled: bool,
    pub player_move_speed: f32,
    pub player_jump_strength: f32,
    pub gravity: f32,
    pub enemy_speed: f32,
    pub enemy_gravity_scale: f32,
    pub enemy_spawn_rows: usize,
    pub min_enemies: usize,
    pub max_enemies: usize,
    pub min_collectibles: usize,
    pub max_collectibles: usize,
    pub platform_rows: usize,
    pub min_platforms_per_row: usize,
    pub max_platforms_per_row: usize,
    pub platform_min_gap_x: f32,
    pub platform_max_gap_x: f32,
    pub platform_min_y: f32,
    pub platform_max_y: f32,
    pub ground_row_enabled: bool,
    pub enemy_on_platform_chance: f32,
    pub collectible_value: u32,
    pub rare_collectible_chance: f32,
    pub rare_collectible_value: u32,
    pub collectible_health_value: u32,
    pub rare_collectible_health_value: u32,
    pub collectibles_follow_path: bool,
    pub collectible_cluster_size_min: usize,
    pub collectible_cluster_size_max: usize,
    pub theme: String,
    pub background_variants: usize,
    pub sprite_scale: f32,
    pub music_volume: f32,
    pub jump_sfx_volume: f32,
    pub hit_sfx_volume: f32,
    pub pickup_sfx_volume: f32,
    pub player_fall_respawn_offset: f32,
    pub auto_respawn_collectibles: bool,
    pub ground_row_y_factor: f32,
    pub key_left_primary: String,
    pub key_left_alt: String,
    pub key_right_primary: String,
    pub key_right_alt: String,
    pub key_jump_primary: String,
    pub key_jump_alt: String,
    pub asset_player_size: u32,
    pub asset_enemy_size: u32,
    pub asset_collectible_size: u32,
    pub asset_goal_size: u32,
    pub asset_platform_width: u32,
    pub asset_platform_height: u32,
    pub background_width: u32,
    pub background_height: u32,
    pub jump_sound_freq: f32,
    pub hit_sound_freq: f32,
    pub pickup_sound_start_freq: f32,
    pub pickup_sound_end_freq: f32,
    pub music_sound_start_freq: f32,
    pub music_sound_end_freq: f32,
    pub jump_sound_duration: f32,
    pub hit_sound_duration: f32,
    pub pickup_sound_duration: f32,
    pub music_sound_duration: f32,
    pub enemy_speed_level_scale: f32,
    pub max_enemies_level_scale: f32,
    pub min_enemies_level_scale: f32,
    pub min_collectibles_level_scale: f32,
    pub max_collectibles_level_scale: f32,
    pub custom_level_folder: String,
    pub world_width_screens: f32,
    pub world_height_screens: f32,
    pub platform_density_bottom: f32,
    pub platform_density_top: f32,
    pub moving_platform_enabled: bool,
    pub moving_platform_vertical: bool,
    pub moving_platform_speed: f32,
    pub moving_platform_amplitude: f32,
    pub collectible_bob_enabled: bool,
    pub collectible_bob_amplitude: f32,
    pub collectible_bob_speed: f32,
    pub enemy_jump_enabled: bool,
    pub enemy_jump_interval: f32,
    pub enemy_jump_strength: f32,
    pub boss_level_interval: u32,
    pub boss_mode: String,
    pub boss_collectibles_multiplier: f32,
    pub boss_enemy_speed_multiplier: f32,
    pub boss_enemy_damage_multiplier: f32,
    pub boss_enemy_count_multiplier: f32,
    pub enemy_shoot_enabled: bool,
    pub enemy_shoot_interval: f32,
    pub enemy_shoot_range: f32,
    pub projectile_speed: f32,
    pub projectile_damage: u32,
    pub particles_enabled: bool,
    pub jump_particle_count: u32,
    pub hit_particle_count: u32,
    pub pickup_particle_count: u32,
    pub editor: EditorOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorOptions {
    pub default_platform_moving: bool,
    pub default_platform_vertical: bool,
    pub default_enemy_jumping: bool,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            default_platform_moving: false,
            default_platform_vertical: false,
            default_enemy_jumping: false,
        }
    }
}

impl Default for GameRules {
    fn default() -> Self {
        Self {
            seed: None,
            mode: "normal".to_string(),
            control_scheme: "both".to_string(),
            enemy_enabled: true,
            collectible_enabled: true,
            sfx_enabled: true,
            music_enabled: true,
            show_fps: false,
            vsync_enabled: true,
            resolution_index: 1,
            debug_overlay: false,
            collectibles_for_level_up: 100,
            max_level: 1000,
            player_start_health: 5,
            player_max_health: 3,
            enemy_contact_damage: 1,
            hit_invincibility_duration: 0.5,
            hit_flash_enabled: true,
            player_move_speed: 220.0,
            player_jump_strength: 420.0,
            gravity: 900.0,
            enemy_speed: 80.0,
            enemy_gravity_scale: 0.0,
            enemy_spawn_rows: 2,
            min_enemies: 3,
            max_enemies: 8,
            min_collectibles: 3,
            max_collectibles: 6,
            platform_rows: 4,
            min_platforms_per_row: 3,
            max_platforms_per_row: 6,
            platform_min_gap_x: 80.0,
            platform_max_gap_x: 200.0,
            platform_min_y: 0.3,
            platform_max_y: 0.85,
            ground_row_enabled: true,
            enemy_on_platform_chance: 0.5,
            collectible_value: 1,
            rare_collectible_chance: 0.1,
            rare_collectible_value: 5,
            collectible_health_value: 1,
            rare_collectible_health_value: 5,
            collectibles_follow_path: false,
            collectible_cluster_size_min: 1,
            collectible_cluster_size_max: 3,
            theme: "default".to_string(),
            background_variants: 2,
            sprite_scale: 1.0,
            music_volume: 0.3,
            jump_sfx_volume: 0.35,
            hit_sfx_volume: 0.5,
            pickup_sfx_volume: 0.4,
            player_fall_respawn_offset: 200.0,
            auto_respawn_collectibles: true,
            ground_row_y_factor: 0.92,
            key_left_primary: "A".to_string(),
            key_left_alt: "Left".to_string(),
            key_right_primary: "D".to_string(),
            key_right_alt: "Right".to_string(),
            key_jump_primary: "Space".to_string(),
            key_jump_alt: "W".to_string(),
            asset_player_size: 32,
            asset_enemy_size: 32,
            asset_collectible_size: 20,
            asset_goal_size: 24,
            asset_platform_width: 64,
            asset_platform_height: 16,
            background_width: 320,
            background_height: 180,
            jump_sound_freq: 880.0,
            hit_sound_freq: 220.0,
            pickup_sound_start_freq: 660.0,
            pickup_sound_end_freq: 990.0,
            music_sound_start_freq: 220.0,
            music_sound_end_freq: 440.0,
            jump_sound_duration: 0.15,
            hit_sound_duration: 0.12,
            pickup_sound_duration: 0.2,
            music_sound_duration: 3.0,
            enemy_speed_level_scale: 0.0,
            max_enemies_level_scale: 0.0,
            min_enemies_level_scale: 0.0,
            min_collectibles_level_scale: 0.0,
            max_collectibles_level_scale: 0.0,
            custom_level_folder: "assets/config/levels".to_string(),
            world_width_screens: 2.0,
            world_height_screens: 3.0,
            platform_density_bottom: 1.5,
            platform_density_top: 0.5,
            moving_platform_enabled: false,
            moving_platform_vertical: false,
            moving_platform_speed: 1.0,
            moving_platform_amplitude: 32.0,
            collectible_bob_enabled: false,
            collectible_bob_amplitude: 6.0,
            collectible_bob_speed: 2.0,
            enemy_jump_enabled: false,
            enemy_jump_interval: 2.0,
            enemy_jump_strength: 220.0,
            boss_level_interval: 5,
            boss_mode: "collect_fest".to_string(),
            boss_collectibles_multiplier: 3.0,
            boss_enemy_speed_multiplier: 1.5,
            boss_enemy_damage_multiplier: 2.0,
            boss_enemy_count_multiplier: 2.0,
            enemy_shoot_enabled: false,
            enemy_shoot_interval: 2.5,
            enemy_shoot_range: 320.0,
            projectile_speed: 260.0,
            projectile_damage: 1,
            particles_enabled: true,
            jump_particle_count: 10,
            hit_particle_count: 18,
            pickup_particle_count: 12,
            editor: EditorOptions::default(),
        }
    }
}

pub fn load_rules(path: &str) -> GameRules {
    let mut rules = match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("Failed to parse rules from {path}: {e}. Using defaults.");
            GameRules::default()
        }),
        Err(e) => {
            eprintln!("Could not read rules file {path}: {e}. Using defaults.");
            GameRules::default()
        }
    };

    match rules.mode.to_lowercase().as_str() {
        "chill" => {
            rules.max_enemies = ((rules.max_enemies as f32) * 0.5).max(1.0) as usize;
            rules.gravity *= 0.8;
        }
        "hardcore" => {
            rules.max_enemies = ((rules.max_enemies as f32) * 1.5).max(1.0) as usize;
            rules.player_move_speed *= 1.1;
            rules.gravity *= 1.1;
        }
        _ => {}
    }

    rules
}

pub fn save_rules(path: &str, rules: &GameRules) -> Result<(), String> {
    let text =
        serde_json::to_string_pretty(rules).map_err(|e| format!("Failed to serialize rules: {e}"))?;
    fs::write(path, text).map_err(|e| format!("Failed to write rules file {path}: {e}"))?;
    Ok(())
}

pub fn generate_scene(
    assets: &Assets,
    rules: &GameRules,
    level: u32,
    screen_size: Vec2,
    rng: &mut impl Rng,
) -> Scene {
    let input = build_input_config(rules);

    let level_index = level.saturating_sub(1) as f32;

    let enemy_speed_factor = 1.0 + rules.enemy_speed_level_scale * level_index;
    let max_enemies_factor = 1.0 + rules.max_enemies_level_scale * level_index;
    let min_enemies_factor = 1.0 + rules.min_enemies_level_scale * level_index;

    let mut effective_enemy_speed = (rules.enemy_speed * enemy_speed_factor).max(0.0);
    let mut effective_min_enemies =
        ((rules.min_enemies as f32) * min_enemies_factor).round().max(1.0) as usize;
    let mut effective_max_enemies =
        ((rules.max_enemies as f32) * max_enemies_factor)
            .round()
            .max(effective_min_enemies as f32) as usize;

    // Boss / event level modifiers
    let is_boss_level =
        rules.boss_level_interval > 0 && level > 0 && level % rules.boss_level_interval == 0;

    let mut enemy_enabled = rules.enemy_enabled;
    let mut collectible_multiplier: f32 = 1.0;
    let mut enemy_speed_mult: f32 = 1.0;
    let mut enemy_damage_mult: f32 = 1.0;
    let mut enemy_count_mult: f32 = 1.0;

    if is_boss_level {
        match rules.boss_mode.to_lowercase().as_str() {
            // Lots of collectibles, no enemies
            "collect_fest" | "collectfest" | "collect" => {
                enemy_enabled = false;
                collectible_multiplier = rules.boss_collectibles_multiplier.max(1.0);
            }
            // Tougher enemy levels
            "boss_enemy" | "boss" => {
                enemy_speed_mult = rules.boss_enemy_speed_multiplier.max(0.0);
                enemy_damage_mult = rules.boss_enemy_damage_multiplier.max(0.0);
                enemy_count_mult = rules.boss_enemy_count_multiplier.max(0.0);
            }
            _ => {}
        }
    }

    effective_enemy_speed *= enemy_speed_mult;
    effective_min_enemies =
        ((effective_min_enemies as f32) * enemy_count_mult).round().max(1.0) as usize;
    effective_max_enemies =
        ((effective_max_enemies as f32) * enemy_count_mult)
            .round()
            .max(effective_min_enemies as f32) as usize;

    let effective_enemy_contact_damage: u32 = {
        let base = rules.enemy_contact_damage.max(1);
        let scaled =
            (base as f32 * enemy_damage_mult).round().max(1.0) as u32;
        scaled
    };
    let mut scene = Scene::new(
        rules.player_move_speed,
        rules.player_jump_strength,
        rules.gravity,
        effective_enemy_speed,
        rules.enemy_gravity_scale,
        rules.sprite_scale,
        rules.player_start_health,
        rules.player_max_health,
        effective_enemy_contact_damage,
        rules.hit_invincibility_duration,
        rules.hit_flash_enabled,
        rules.player_fall_respawn_offset,
        rules.jump_sfx_volume,
        rules.hit_sfx_volume,
        rules.pickup_sfx_volume,
        input,
        screen_size.x,
        screen_size.y,
        rules.moving_platform_enabled,
        rules.moving_platform_vertical,
        rules.moving_platform_speed,
        rules.moving_platform_amplitude,
        rules.collectible_bob_enabled,
        rules.collectible_bob_amplitude,
        rules.collectible_bob_speed,
        rules.enemy_jump_enabled,
        rules.enemy_jump_interval,
        rules.enemy_jump_strength,
        rules.enemy_shoot_enabled,
        rules.enemy_shoot_interval,
        rules.enemy_shoot_range,
        rules.projectile_speed,
        rules.projectile_damage,
        rules.particles_enabled,
        rules.jump_particle_count,
        rules.hit_particle_count,
        rules.pickup_particle_count,
    );

    // Background
    let backgrounds = assets.sprites_of_kind(SpriteKind::Background);
    if let Some(bg_asset) = choose_random(&backgrounds, rng) {
        scene.background = Some(bg_asset.texture.clone());
    }

    // Custom level mode: try to load layout from JSON instead of random generation.
    if rules.mode.eq_ignore_ascii_case("custom") {
        if apply_custom_level(&mut scene, assets, rules, level, screen_size) {
            return scene;
        } else {
            eprintln!(
                "Custom mode is set but loading custom level failed; falling back to random generation."
            );
        }
    }

    // Platforms
    let platform_sprites = assets.sprites_of_kind(SpriteKind::Platform);
    if !platform_sprites.is_empty() {
        let rows = rules.platform_rows.max(1);

        let min_y = (rules.platform_min_y.clamp(0.0, 1.0) * screen_size.y).min(screen_size.y);
        let max_y = (rules.platform_max_y.clamp(0.0, 1.0) * screen_size.y).max(min_y + 1.0);
        let height_span = (max_y - min_y).max(1.0);

        for row in 0..rows {
            let t = if rows == 1 {
                0.5
            } else {
                row as f32 / (rows.saturating_sub(1) as f32)
            };
            let y = max_y - t * height_span;

            let max_by_gap = if rules.platform_min_gap_x > 0.0 {
                (screen_size.x / rules.platform_min_gap_x).max(1.0) as usize
            } else {
                rules.max_platforms_per_row
            };

            let base_max = rules.max_platforms_per_row.min(max_by_gap.max(1));
            let base_min = rules.min_platforms_per_row.min(base_max).max(1);

            let density = (rules.platform_density_bottom
                + (rules.platform_density_top - rules.platform_density_bottom) * t)
                .max(0.1);

            let max_for_row =
                ((base_max as f32) * density).round().max(1.0) as usize;
            let min_for_row =
                base_min.min(max_for_row).max(1);

            let count = rng.gen_range(min_for_row..=max_for_row);

            for _ in 0..count {
                if let Some(sprite) = choose_random(&platform_sprites, rng) {
                    let tex_w = sprite.texture.width() * rules.sprite_scale;
                    if tex_w <= 0.0 {
                        continue;
                    }
                    let margin = tex_w / 2.0;
                    let x = rng.gen_range(margin..(screen_size.x - margin));

                    scene.platforms.push(Platform {
                        texture: sprite.texture.clone(),
                        position: vec2(x, y),
                        base_position: vec2(x, y),
                        phase: rng.gen_range(0.0..std::f32::consts::TAU),
                        moving: rules.moving_platform_enabled,
                        vertical: rules.moving_platform_vertical,
                    });
                }
            }
        }

        if rules.ground_row_enabled {
            let y = screen_size.y * rules.ground_row_y_factor.clamp(0.0, 2.0);
            let max_by_gap = if rules.platform_min_gap_x > 0.0 {
                (screen_size.x / rules.platform_min_gap_x).max(1.0) as usize
            } else {
                rules.max_platforms_per_row
            };
            let max_for_row = rules.max_platforms_per_row.min(max_by_gap.max(1));
            let min_for_row = rules.min_platforms_per_row.min(max_for_row).max(1);
            let count = rng.gen_range(min_for_row..=max_for_row);

            for _ in 0..count {
                if let Some(sprite) = choose_random(&platform_sprites, rng) {
                    let tex_w = sprite.texture.width() * rules.sprite_scale;
                    if tex_w <= 0.0 {
                        continue;
                    }
                    let margin = tex_w / 2.0;
                    let x = rng.gen_range(margin..(screen_size.x - margin));

                    scene.platforms.push(Platform {
                        texture: sprite.texture.clone(),
                        position: vec2(x, y),
                        base_position: vec2(x, y),
                        phase: rng.gen_range(0.0..std::f32::consts::TAU),
                        moving: rules.moving_platform_enabled,
                        vertical: rules.moving_platform_vertical,
                    });
                }
            }
        }
    }

    // Player
    let players = assets.sprites_of_kind(SpriteKind::Player);
    if let Some(player_asset) = choose_random(&players, rng) {
        let player_pos = vec2(screen_size.x / 2.0, 0.0);
        scene.entities.push(Entity {
            kind: EntityKind::Player,
            texture: player_asset.texture.clone(),
            position: player_pos,
            velocity: Vec2::ZERO,
            value: 0,
            health_value: 0,
            base_position: player_pos,
            phase: 0.0,
            jumping: false,
        });
    }

    // Enemies
    let enemies = assets.sprites_of_kind(SpriteKind::Enemy);
    if enemy_enabled && !enemies.is_empty() {
        let enemy_count = rng.gen_range(effective_min_enemies..=effective_max_enemies);

        for _ in 0..enemy_count {
            if let Some(enemy_asset) = choose_random(&enemies, rng) {
                let use_platform = !scene.platforms.is_empty()
                    && rng.gen::<f32>() < rules.enemy_on_platform_chance;

                let (x, y) = if use_platform {
                    let idx = rng.gen_range(0..scene.platforms.len());
                    let p = &scene.platforms[idx];
                    (p.position.x, p.position.y - enemy_asset.texture.height() * rules.sprite_scale)
                } else {
                    let rows = rules.enemy_spawn_rows.max(1);
                    let row = rng.gen_range(0..rows);
                    let min_y = (rules.platform_min_y.clamp(0.0, 1.0) * screen_size.y)
                        .min(screen_size.y);
                    let max_y = (rules.platform_max_y.clamp(0.0, 1.0) * screen_size.y)
                        .max(min_y + 1.0);
                    let span = (max_y - min_y).max(1.0);
                    let t = if rows == 1 {
                        0.5
                    } else {
                        row as f32 / (rows.saturating_sub(1) as f32)
                    };
                    let y = max_y - t * span;
                    let x = rng.gen_range(0.0..screen_size.x);
                    (x, y)
                };

                let dir = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };

                scene.entities.push(Entity {
                    kind: EntityKind::Enemy,
                    texture: enemy_asset.texture.clone(),
                    position: vec2(x, y),
                    velocity: vec2(dir * effective_enemy_speed, 0.0),
                    value: 0,
                    health_value: 0,
                    base_position: vec2(x, y),
                    phase: rng.gen_range(0.0..std::f32::consts::TAU),
                    jumping: rules.enemy_jump_enabled,
                });
            }
        }
    }

    spawn_collectibles(
        &mut scene,
        assets,
        rules,
        level,
        rng,
        collectible_multiplier,
    );

    scene
}

pub fn spawn_collectibles(
    scene: &mut Scene,
    assets: &Assets,
    rules: &GameRules,
    level: u32,
    rng: &mut impl Rng,
    multiplier: f32,
) {
    let collectible_sprites = assets.sprites_of_kind(SpriteKind::Collectible);
    if !rules.collectible_enabled
        || collectible_sprites.is_empty()
        || scene.platforms.is_empty()
    {
        return;
    }

    let level_index = level.saturating_sub(1) as f32;
    let min_factor = 1.0 + rules.min_collectibles_level_scale * level_index;
    let max_factor = 1.0 + rules.max_collectibles_level_scale * level_index;

    let base_min = rules.min_collectibles.max(0);
    let base_max = rules.max_collectibles.max(base_min);

    let effective_min = ((base_min as f32) * min_factor).round().max(0.0) as usize;
    let effective_max = ((base_max as f32) * max_factor)
        .round()
        .max(effective_min as f32) as usize;

    let base_total = rng.gen_range(effective_min..=effective_max.max(effective_min));
    let total = ((base_total as f32) * multiplier.max(0.0))
        .round()
        .max(0.0) as usize;
    let cluster_min = rules.collectible_cluster_size_min.max(1);
    let cluster_max = rules.collectible_cluster_size_max.max(cluster_min);

    let mut remaining = total;

    let mut platform_indices: Vec<usize> = (0..scene.platforms.len()).collect();
    platform_indices.sort_by(|&a, &b| {
        scene.platforms[a]
            .position
            .x
            .partial_cmp(&scene.platforms[b].position.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    while remaining > 0 {
        let cluster_size =
            rng.gen_range(cluster_min..=cluster_max).min(remaining).max(1);
        remaining -= cluster_size;

        if scene.platforms.is_empty() {
            break;
        }

        if rules.collectibles_follow_path && platform_indices.len() >= cluster_size {
            let start_idx = rng.gen_range(0..=platform_indices.len() - cluster_size);
            for i in 0..cluster_size {
                let p_index = platform_indices[start_idx + i];
                let platform = &scene.platforms[p_index];
                if let Some(sprite) = choose_random(&collectible_sprites, rng) {
                    let x = platform.position.x;
                    let y =
                        platform.position.y - sprite.texture.height() * rules.sprite_scale;

                    let is_rare = rng.gen::<f32>() < rules.rare_collectible_chance;
                    let value = if is_rare {
                        rules.rare_collectible_value
                    } else {
                        rules.collectible_value
                    };
                    let health_value = if is_rare {
                        rules.rare_collectible_health_value
                    } else {
                        rules.collectible_health_value
                    };

                    scene.entities.push(Entity {
                        kind: EntityKind::Collectible,
                        texture: sprite.texture.clone(),
                        position: vec2(x, y),
                        velocity: Vec2::ZERO,
                        value,
                        health_value,
                        base_position: vec2(x, y),
                        phase: rng.gen_range(0.0..std::f32::consts::TAU),
                        jumping: false,
                    });
                }
            }
        } else {
            for _ in 0..cluster_size {
                let platform_index = rng.gen_range(0..scene.platforms.len());
                let platform = &scene.platforms[platform_index];
                if let Some(sprite) = choose_random(&collectible_sprites, rng) {
                    let x = platform.position.x;
                    let y =
                        platform.position.y - sprite.texture.height() * rules.sprite_scale;

                    let is_rare = rng.gen::<f32>() < rules.rare_collectible_chance;
                    let value = if is_rare {
                        rules.rare_collectible_value
                    } else {
                        rules.collectible_value
                    };
                    let health_value = if is_rare {
                        rules.rare_collectible_health_value
                    } else {
                        rules.collectible_health_value
                    };

                    scene.entities.push(Entity {
                        kind: EntityKind::Collectible,
                        texture: sprite.texture.clone(),
                        position: vec2(x, y),
                        velocity: Vec2::ZERO,
                        value,
                        health_value,
                        base_position: vec2(x, y),
                        phase: rng.gen_range(0.0..std::f32::consts::TAU),
                        jumping: false,
                    });
                }
            }
        }
    }
}

fn choose_random<'a, T>(items: &'a [&T], rng: &mut impl Rng) -> Option<&'a T> {
    if items.is_empty() {
        None
    } else {
        let idx = rng.gen_range(0..items.len());
        Some(items[idx])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLevelEntity {
    pub sprite: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub moving: bool,
    #[serde(default)]
    pub vertical: bool,
    #[serde(default)]
    pub jumping: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLevelCollectible {
    pub sprite: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub value: u32,
    #[serde(default)]
    pub health: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLevel {
    #[serde(default)]
    pub player_start: Option<CustomLevelEntity>,
    #[serde(default)]
    pub platforms: Vec<CustomLevelEntity>,
    #[serde(default)]
    pub enemies: Vec<CustomLevelEntity>,
    #[serde(default)]
    pub collectibles: Vec<CustomLevelCollectible>,
}

fn apply_custom_level(
    scene: &mut Scene,
    assets: &Assets,
    rules: &GameRules,
    level: u32,
    _screen_size: Vec2,
) -> bool {
    let def = match load_custom_level(level, rules) {
        Some(d) => d,
        None => return false,
    };

    apply_custom_level_def(scene, assets, rules, &def)
}

pub fn load_custom_level(level: u32, rules: &GameRules) -> Option<CustomLevel> {
    let folder = if rules.custom_level_folder.trim().is_empty() {
        "assets/config/levels"
    } else {
        rules.custom_level_folder.trim()
    };

    let path = format!("{folder}/level{level}.json");

    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to read custom level {path}: {e}");
            return None;
        }
    };

    let def: CustomLevel = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse custom level {path}: {e}");
            return None;
        }
    };

    Some(def)
}

pub fn save_custom_level(
    level: u32,
    rules: &GameRules,
    def: &CustomLevel,
) -> Result<(), String> {
    let folder = if rules.custom_level_folder.trim().is_empty() {
        "assets/config/levels"
    } else {
        rules.custom_level_folder.trim()
    };

    let path = format!("{folder}/level{level}.json");
    let text = serde_json::to_string_pretty(def)
        .map_err(|e| format!("Failed to serialize custom level {path}: {e}"))?;
    fs::write(&path, text)
        .map_err(|e| format!("Failed to write custom level {path}: {e}"))?;
    Ok(())
}

fn apply_custom_level_def(
    scene: &mut Scene,
    assets: &Assets,
    rules: &GameRules,
    def: &CustomLevel,
) -> bool {
    // Player
    let player_sprite = assets
        .sprites_of_kind(SpriteKind::Player)
        .get(0)
        .map(|s| *s);
    if let Some(base_sprite) = player_sprite {
        let (px, py) = if let Some(start) = def.player_start.as_ref() {
            (start.x, start.y)
        } else {
            (scene.world_width / 2.0, 0.0)
        };
        scene.entities.push(Entity {
            kind: EntityKind::Player,
            texture: base_sprite.texture.clone(),
            position: vec2(px, py),
            velocity: Vec2::ZERO,
            value: 0,
            health_value: 0,
            base_position: vec2(px, py),
            phase: 0.0,
            jumping: false,
        });
    } else {
        eprintln!("No player sprites loaded; cannot build custom level.");
        return false;
    }

    // Platforms
    for p in &def.platforms {
        let sprite = assets
            .sprite_by_kind_and_name(SpriteKind::Platform, &p.sprite)
            .or_else(|| {
                assets
                    .sprites_of_kind(SpriteKind::Platform)
                    .get(0)
                    .copied()
            });
        if let Some(s) = sprite {
            let pos = vec2(p.x, p.y);
            scene.platforms.push(Platform {
                texture: s.texture.clone(),
                position: pos,
                base_position: pos,
                phase: 0.0,
                moving: p.moving,
                vertical: p.vertical,
            });
        } else {
            eprintln!(
                "Custom level: no platform sprite found for '{}'",
                p.sprite
            );
        }
    };

    // Enemies
    for e_def in &def.enemies {
        let sprite = assets
            .sprite_by_kind_and_name(SpriteKind::Enemy, &e_def.sprite)
            .or_else(|| {
                assets.sprites_of_kind(SpriteKind::Enemy).get(0).copied()
            });
        if let Some(s) = sprite {
            let pos = vec2(e_def.x, e_def.y);
            let vel = vec2(rules.enemy_speed, 0.0);
            scene.entities.push(Entity {
                kind: EntityKind::Enemy,
                texture: s.texture.clone(),
                position: pos,
                velocity: vel,
                value: 0,
                health_value: 0,
                base_position: pos,
                phase: 0.0,
                jumping: e_def.jumping,
            });
        } else {
            eprintln!(
                "Custom level: no enemy sprite found for '{}'",
                e_def.sprite
            );
        }
    }

    // Collectibles
    for c in &def.collectibles {
        let sprite = assets
            .sprite_by_kind_and_name(SpriteKind::Collectible, &c.sprite)
            .or_else(|| {
                assets
                    .sprites_of_kind(SpriteKind::Collectible)
                    .get(0)
                    .copied()
            });
        if let Some(s) = sprite {
            let pos = vec2(c.x, c.y);
            let value = if c.value > 0 {
                c.value
            } else {
                rules.collectible_value
            };
            let health_value = if c.health > 0 {
                c.health
            } else {
                rules.collectible_health_value
            };

            scene.entities.push(Entity {
                kind: EntityKind::Collectible,
                texture: s.texture.clone(),
                position: pos,
                velocity: Vec2::ZERO,
                value,
                health_value,
                base_position: pos,
                phase: 0.0,
                jumping: false,
            });
        } else {
            eprintln!(
                "Custom level: no collectible sprite found for '{}'",
                c.sprite
            );
        }
    }

    true
}

fn build_input_config(rules: &GameRules) -> InputConfig {
    let scheme = rules.control_scheme.to_lowercase();

    match scheme.as_str() {
        "wasd" => InputConfig {
            move_left_primary: KeyCode::A,
            move_left_alt: None,
            move_right_primary: KeyCode::D,
            move_right_alt: None,
            jump_primary: KeyCode::Space,
            jump_alt: Some(KeyCode::W),
        },
        "arrows" => InputConfig {
            move_left_primary: KeyCode::Left,
            move_left_alt: None,
            move_right_primary: KeyCode::Right,
            move_right_alt: None,
            jump_primary: KeyCode::Up,
            jump_alt: None,
        },
        "custom" => InputConfig {
            move_left_primary: parse_key(&rules.key_left_primary).unwrap_or(KeyCode::A),
            move_left_alt: parse_optional_key(&rules.key_left_alt),
            move_right_primary: parse_key(&rules.key_right_primary).unwrap_or(KeyCode::D),
            move_right_alt: parse_optional_key(&rules.key_right_alt),
            jump_primary: parse_key(&rules.key_jump_primary).unwrap_or(KeyCode::Space),
            jump_alt: parse_optional_key(&rules.key_jump_alt),
        },
        _ => InputConfig {
            move_left_primary: KeyCode::A,
            move_left_alt: Some(KeyCode::Left),
            move_right_primary: KeyCode::D,
            move_right_alt: Some(KeyCode::Right),
            jump_primary: KeyCode::Space,
            jump_alt: Some(KeyCode::W),
        },
    }
}

fn parse_optional_key(name: &str) -> Option<KeyCode> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else {
        parse_key(trimmed)
    }
}

fn parse_key(name: &str) -> Option<KeyCode> {
    let s = name.trim();
    if s.is_empty() {
        return None;
    }

    if s.len() == 1 {
        let c = s.chars().next().unwrap().to_ascii_uppercase();
        return match c {
            'A' => Some(KeyCode::A),
            'B' => Some(KeyCode::B),
            'C' => Some(KeyCode::C),
            'D' => Some(KeyCode::D),
            'E' => Some(KeyCode::E),
            'F' => Some(KeyCode::F),
            'G' => Some(KeyCode::G),
            'H' => Some(KeyCode::H),
            'I' => Some(KeyCode::I),
            'J' => Some(KeyCode::J),
            'K' => Some(KeyCode::K),
            'L' => Some(KeyCode::L),
            'M' => Some(KeyCode::M),
            'N' => Some(KeyCode::N),
            'O' => Some(KeyCode::O),
            'P' => Some(KeyCode::P),
            'Q' => Some(KeyCode::Q),
            'R' => Some(KeyCode::R),
            'S' => Some(KeyCode::S),
            'T' => Some(KeyCode::T),
            'U' => Some(KeyCode::U),
            'V' => Some(KeyCode::V),
            'W' => Some(KeyCode::W),
            'X' => Some(KeyCode::X),
            'Y' => Some(KeyCode::Y),
            'Z' => Some(KeyCode::Z),
            _ => None,
        };
    }

    match s.to_ascii_uppercase().as_str() {
        "LEFT" => Some(KeyCode::Left),
        "RIGHT" => Some(KeyCode::Right),
        "UP" => Some(KeyCode::Up),
        "DOWN" => Some(KeyCode::Down),
        "SPACE" | "SPACEBAR" => Some(KeyCode::Space),
        "ESC" | "ESCAPE" => Some(KeyCode::Escape),
        _ => None,
    }
}
