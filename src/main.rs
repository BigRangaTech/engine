mod assets;
mod asset_gen;
mod generator;
mod scene;
mod export;

use crate::assets::Assets;
use crate::export::{
    export_run_config,
    save_preset,
    load_preset,
    export_run_cartridge,
    list_run_cartridges,
    load_run_cartridge,
};
use crate::generator::{generate_scene, load_rules, save_rules, spawn_collectibles, GameRules};
use crate::scene::{Scene, Sounds, EntityKind};
use macroquad::prelude::*;
use ::rand::SeedableRng;
use ::rand::rngs::StdRng;

const ASSETS_ROOT: &str = "assets";
const RULES_PATH: &str = "assets/config/rules.json";

#[derive(Copy, Clone)]
enum GameState {
    MainMenu,
    CartridgeMenu,
    Playing,
    Paused,
    Settings,
    RulesEditor,
    LevelEditor,
    RebindingControls,
    PresetsMenu,
    Help,
    GameOver,
    Won,
}

const RESOLUTIONS: &[(f32, f32)] = &[
    (800.0, 600.0),
    (1280.0, 720.0),
    (1600.0, 900.0),
    (1920.0, 1080.0),
];

#[macroquad::main("Random Asset Game")]
async fn main() {
    let mut rules = load_rules(RULES_PATH);

    let mut seed = rules
        .seed
        .unwrap_or_else(random_seed_from_time);
    println!("Using rules: {:?}, seed: {}", rules, seed);

    let mut resolution_index: i32 = rules
        .resolution_index
        .min(RESOLUTIONS.len().saturating_sub(1))
        as i32;
    let (init_w, init_h) = RESOLUTIONS[resolution_index as usize];
    macroquad::window::request_new_screen_size(init_w, init_h);

    if let Err(e) = asset_gen::generate_placeholder_assets(
        seed,
        &rules,
    ) {
        eprintln!("Failed to generate placeholder assets: {e}");
    }

    let assets = match assets::load_assets(ASSETS_ROOT).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to load assets: {e}");
            return;
        }
    };

    let sounds = load_sounds(&rules).await;
    update_music_volume(&sounds, &rules);

    let mut level: u32 = 1;
    let mut editor_level: u32 = 1;
    let mut total_collected: u32 = 0;
    let mut run_score: u32 = 0;
    let mut run_time: f32 = 0.0;
    let mut state = GameState::MainMenu;
    let mut menu_index: i32 = 0;
    let mut pause_index: i32 = 0;
    let mut settings_index: i32 = 0;
    let mut rules_menu_index: i32 = 0;
    let mut rebind_step: i32 = 0;
    let mut presets_index: i32 = 0;
    let mut settings_return_to = GameState::Paused;
    let mut cartridge_files: Vec<String> = Vec::new();
    let mut cartridge_index: i32 = 0;
    let mut level_rng = StdRng::seed_from_u64(seed ^ 0x9E3779B97F4A7C15);
    let mut scene = make_scene(&assets, &rules, level, &mut level_rng);
    let mut editor_level_data: Option<crate::generator::CustomLevel> = None;
    let mut editor_tool_index: i32 = 1; // 0: Player, 1: Platform, 2: Enemy, 3: Collectible, 4: Eraser
    let mut editor_player_index: i32 = 0;
    let mut editor_platform_index: i32 = 0;
    let mut editor_enemy_index: i32 = 0;
    let mut editor_collectible_index: i32 = 0;
    let mut editor_camera: Vec2 = Vec2::ZERO;
    let mut editor_preview_scale: f32 = 1.0;

    loop {
        let frame_start = std::time::Instant::now();
        let dt = get_frame_time();

        match state {
            GameState::MainMenu => {
                let up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
                let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);

                if up {
                    menu_index = (menu_index - 1).rem_euclid(7);
                }
                if down {
                    menu_index = (menu_index + 1).rem_euclid(7);
                }

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    match menu_index {
                        0 => {
                            level = 1;
                            total_collected = 0;
                            level_rng =
                                StdRng::seed_from_u64(seed ^ 0x9E3779B97F4A7C15);
                            scene = make_scene(&assets, &rules, level, &mut level_rng);
                            state = GameState::Playing;
                        }
                        1 => {
                            // Custom Levels: play using custom mode
                            level = 1;
                            total_collected = 0;
                            rules.mode = "custom".to_string();
                            level_rng =
                                StdRng::seed_from_u64(seed ^ 0x9E3779B97F4A7C15);
                            scene = make_scene(&assets, &rules, level, &mut level_rng);
                            state = GameState::Playing;
                        }
                        2 => {
                            // Level Editor
                            editor_level = 1;
                            editor_level_data =
                                crate::generator::load_custom_level(editor_level, &rules);
                            state = GameState::LevelEditor;
                        }
                        3 => {
                            // Play Cartridge
                            cartridge_files = list_run_cartridges().unwrap_or_else(|e| {
                                eprintln!("{e}");
                                Vec::new()
                            });
                            cartridge_index = 0;
                            state = GameState::CartridgeMenu;
                        }
                        4 => {
                            settings_return_to = GameState::MainMenu;
                            state = GameState::Settings;
                        }
                        5 => {
                            state = GameState::Help;
                        }
                        6 => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
            GameState::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Paused;
                } else {
                    // Edit current level while in custom mode
                    if rules.mode.to_lowercase() == "custom"
                        && is_key_pressed(KeyCode::F2)
                    {
                        editor_level = level;
                        editor_level_data =
                            crate::generator::load_custom_level(editor_level, &rules);
                        if let Some(player_pos) = scene.player_position() {
                            let sw = screen_width();
                            let sh = screen_height();
                            editor_camera = vec2(
                                player_pos.x - sw / 2.0,
                                player_pos.y - sh / 2.0,
                            );
                        } else {
                            editor_camera = Vec2::ZERO;
                        }
                        state = GameState::LevelEditor;
                    }

                    if is_key_pressed(KeyCode::R) {
                        scene = make_scene(&assets, &rules, level, &mut level_rng);
                    }

                    let prev_score = scene.score;
                    scene.update(dt, &sounds, rules.sfx_enabled);

                    if scene.player_dead {
                        state = GameState::GameOver;
                    } else {
                        let gained = scene.score.saturating_sub(prev_score);
                        total_collected = total_collected.saturating_add(gained as u32);
                        run_score = run_score.saturating_add(gained as u32);
                        run_time += dt;

                        if total_collected >= rules.collectibles_for_level_up {
                            if level < rules.max_level {
                                level += 1;
                                total_collected -= rules.collectibles_for_level_up;
                                scene = make_scene(&assets, &rules, level, &mut level_rng);
                            } else {
                                state = GameState::Won;
                            }
                        } else {
                            let has_collectibles = scene
                                .entities
                                .iter()
                                .any(|e| matches!(e.kind, EntityKind::Collectible));
                            if rules.auto_respawn_collectibles && !has_collectibles {
                                spawn_collectibles(
                                    &mut scene,
                                    &assets,
                                    &rules,
                                    level,
                                    &mut level_rng,
                                    1.0,
                                );
                            }
                        }
                    }
                }
            }
            GameState::Paused => {
                let up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
                let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);

                if up {
                    pause_index = (pause_index - 1).rem_euclid(4);
                }
                if down {
                    pause_index = (pause_index + 1).rem_euclid(4);
                }

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    match pause_index {
                        0 => state = GameState::Playing,      // Resume
                        1 => {
                            settings_return_to = GameState::Paused;
                            state = GameState::Settings;       // Settings
                        }
                        2 => state = GameState::MainMenu,     // Main menu
                        3 => break,                           // Quit
                        _ => {}
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Playing;
                }
            }
            GameState::Settings => {
                let up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
                let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
                let left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
                let right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);

                if up {
                    settings_index = (settings_index - 1).rem_euclid(14);
                }
                if down {
                    settings_index = (settings_index + 1).rem_euclid(14);
                }

                if settings_index == 0 {
                    if left {
                        resolution_index =
                            (resolution_index - 1).rem_euclid(RESOLUTIONS.len() as i32);
                    }
                    if right {
                        resolution_index =
                            (resolution_index + 1).rem_euclid(RESOLUTIONS.len() as i32);
                    }

                    let (w, h) = RESOLUTIONS[resolution_index as usize];
                    rules.resolution_index = resolution_index as usize;
                    macroquad::window::request_new_screen_size(w, h);
                }

                if settings_index == 1 {
                    let schemes = ["both", "wasd", "arrows", "custom"];
                    let mut idx = schemes
                        .iter()
                        .position(|s| s.eq_ignore_ascii_case(&rules.control_scheme))
                        .unwrap_or(0) as i32;
                    if left {
                        idx = (idx - 1).rem_euclid(schemes.len() as i32);
                    }
                    if right {
                        idx = (idx + 1).rem_euclid(schemes.len() as i32);
                    }
                    let new_scheme = schemes[idx as usize].to_string();
                    if new_scheme != rules.control_scheme {
                        rules.control_scheme = new_scheme;
                        // Regenerate scene so new control scheme takes effect
                        scene = make_scene(&assets, &rules, level, &mut level_rng);
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    if let Err(e) = save_rules(RULES_PATH, &rules) {
                        eprintln!("{e}");
                    }
                    state = settings_return_to;
                } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    match settings_index {
                        0 => {} // resolution handled with left/right
                        1 => {} // controls handled with left/right
                        2 => {
                            // Rebind controls flow
                            rules.control_scheme = "custom".to_string();
                            rebind_step = 0;
                            state = GameState::RebindingControls;
                        }
                        3 => {
                            rules.enemy_enabled = !rules.enemy_enabled;
                        }
                        4 => {
                            rules.collectible_enabled = !rules.collectible_enabled;
                        }
                        5 => {
                            rules.sfx_enabled = !rules.sfx_enabled;
                        }
                        6 => {
                            rules.music_enabled = !rules.music_enabled;
                            update_music_volume(&sounds, &rules);
                        }
                        7 => {
                            rules.vsync_enabled = !rules.vsync_enabled;
                        }
                        8 => {
                            rules.show_fps = !rules.show_fps;
                        }
                        9 => {
                            rules.debug_overlay = !rules.debug_overlay;
                        }
                        10 => {
                            // Open advanced rules editor
                            rules_menu_index = 0;
                            state = GameState::RulesEditor;
                        }
                        11 => {
                            // Presets submenu
                            presets_index = 0;
                            state = GameState::PresetsMenu;
                        }
                        12 => {
                            if let Err(e) = export_run_config(&rules, seed) {
                                eprintln!("{e}");
                            }
                        }
                        13 => {
                            if let Err(e) = save_rules(RULES_PATH, &rules) {
                                eprintln!("{e}");
                            }
                            state = settings_return_to;
                        }
                        _ => {}
                    }
                }
            }
            GameState::RebindingControls => {
                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Settings;
                } else if let Some(key) = get_last_key_pressed() {
                    if let Some(name) = keycode_to_rules_string(key) {
                        match rebind_step {
                            0 => rules.key_left_primary = name,
                            1 => rules.key_left_alt = name,
                            2 => rules.key_right_primary = name,
                            3 => rules.key_right_alt = name,
                            4 => rules.key_jump_primary = name,
                            5 => rules.key_jump_alt = name,
                            _ => {}
                        }
                        rebind_step += 1;
                        if rebind_step >= 6 {
                            // Rebuild scene so new bindings take effect
                            scene = make_scene(&assets, &rules, level, &mut level_rng);
                            state = GameState::Settings;
                        }
                    }
                }
            }
            GameState::PresetsMenu => {
                let up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
                let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);

                if up {
                    presets_index = (presets_index - 1).rem_euclid(7);
                }
                if down {
                    presets_index = (presets_index + 1).rem_euclid(7);
                }

                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Settings;
                } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    match presets_index {
                        0 | 1 | 2 => {
                            let slot = match presets_index {
                                0 => "slot1",
                                1 => "slot2",
                                _ => "slot3",
                            };
                            if let Ok(loaded) = load_preset(slot) {
                                rules = loaded;
                                // Update resolution and window size
                                resolution_index = rules
                                    .resolution_index
                                    .min(RESOLUTIONS.len().saturating_sub(1)) as i32;
                                let (w, h) =
                                    RESOLUTIONS[resolution_index as usize];
                                macroquad::window::request_new_screen_size(w, h);
                                update_music_volume(&sounds, &rules);
                                scene = make_scene(&assets, &rules, level, &mut level_rng);
                            }
                        }
                        3 | 4 | 5 => {
                            let slot = match presets_index {
                                3 => "slot1",
                                4 => "slot2",
                                _ => "slot3",
                            };
                            if let Err(e) = save_preset(&rules, slot) {
                                eprintln!("{e}");
                            }
                        }
                        6 => {
                            state = GameState::Settings;
                        }
                        _ => {}
                    }
                }
            }
            GameState::RulesEditor => {
                let up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
                let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
                let left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
                let right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);

                // Fixed list of tunable fields
                const RULE_ITEMS: usize = 12;

                if up {
                    rules_menu_index = (rules_menu_index - 1).rem_euclid(RULE_ITEMS as i32);
                }
                if down {
                    rules_menu_index = (rules_menu_index + 1).rem_euclid(RULE_ITEMS as i32);
                }

                let step_small_f32 = 10.0;
                let step_small_u32 = 1;

                if left || right {
                    let dir = if left { -1.0 } else { 1.0 };
                    match rules_menu_index {
                        0 => {
                            // Mode: normal / custom
                            let modes = ["normal", "custom"];
                            let mut idx = modes
                                .iter()
                                .position(|m| m.eq_ignore_ascii_case(&rules.mode))
                                .unwrap_or(0) as i32;
                            idx = (idx + if left { -1 } else { 1 })
                                .rem_euclid(modes.len() as i32);
                            rules.mode = modes[idx as usize].to_string();
                        }
                        1 => {
                            // Collectibles per level up
                            let v = rules.collectibles_for_level_up as i32
                                + if left { -1 } else { 1 } * step_small_u32 as i32;
                            rules.collectibles_for_level_up = v.max(1) as u32;
                        }
                        2 => {
                            // Max level
                            let v =
                                rules.max_level as i32 + if left { -1 } else { 1 } * step_small_u32 as i32;
                            rules.max_level = v.max(1) as u32;
                        }
                        3 => {
                            // Player start health
                            let v = rules.player_start_health as i32
                                + if left { -1 } else { 1 } * step_small_u32 as i32;
                            rules.player_start_health = v.max(1) as u32;
                        }
                        4 => {
                            // Player max health
                            let v = rules.player_max_health as i32
                                + if left { -1 } else { 1 } * step_small_u32 as i32;
                            rules.player_max_health =
                                v.max(rules.player_start_health as i32).max(1) as u32;
                        }
                        5 => {
                            // Enemy contact damage
                            let v = rules.enemy_contact_damage as i32
                                + if left { -1 } else { 1 } * step_small_u32 as i32;
                            rules.enemy_contact_damage = v.max(1) as u32;
                        }
                        6 => {
                            // Player move speed
                            rules.player_move_speed =
                                (rules.player_move_speed + dir * step_small_f32).max(10.0);
                        }
                        7 => {
                            // Player jump strength
                            rules.player_jump_strength =
                                (rules.player_jump_strength + dir * step_small_f32).max(10.0);
                        }
                        8 => {
                            // Gravity
                            rules.gravity =
                                (rules.gravity + dir * step_small_f32).max(10.0);
                        }
                        9 => {
                            // Enemy speed
                            rules.enemy_speed =
                                (rules.enemy_speed + dir * step_small_f32).max(0.0);
                        }
                        10 => {
                            // Boss level interval
                            let v = rules.boss_level_interval as i32
                                + if left { -1 } else { 1 } * step_small_u32 as i32;
                            rules.boss_level_interval = v.max(0) as u32;
                        }
                        11 => {
                            // Enemy shooting toggle
                            rules.enemy_shoot_enabled = !rules.enemy_shoot_enabled;
                        }
                        _ => {}
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    if let Err(e) = save_rules(RULES_PATH, &rules) {
                        eprintln!("{e}");
                    }
                    state = GameState::Settings;
                }
            }
            GameState::LevelEditor => {
                // Ensure we have some data to edit
                if editor_level_data.is_none() {
                    editor_level_data = Some(crate::generator::CustomLevel {
                        player_start: None,
                        platforms: Vec::new(),
                        enemies: Vec::new(),
                        collectibles: Vec::new(),
                    });
                }

                let up = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
                let down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);
                let left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
                let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);

                let pan_speed = 400.0;
                if left {
                    editor_camera.x -= pan_speed * dt;
                }
                if right {
                    editor_camera.x += pan_speed * dt;
                }
                if up {
                    editor_camera.y -= pan_speed * dt;
                }
                if down {
                    editor_camera.y += pan_speed * dt;
                }

                // Scroll wheel adjusts preview scale
                let (_wheel_x, wheel_y) = mouse_wheel();
                if wheel_y.abs() > f32::EPSILON {
                    editor_preview_scale =
                        (editor_preview_scale + wheel_y * 0.1).clamp(0.25, 4.0);
                }

                // Cycle tools with Q/E
                if is_key_pressed(KeyCode::Q) {
                    editor_tool_index = (editor_tool_index - 1).rem_euclid(5);
                }
                if is_key_pressed(KeyCode::E) {
                    editor_tool_index = (editor_tool_index + 1).rem_euclid(5);
                }

                // Change level with Z/X (save current, then load new)
                if is_key_pressed(KeyCode::Z) && editor_level > 1 {
                    if let Some(ref data) = editor_level_data {
                        if let Err(e) =
                            crate::generator::save_custom_level(editor_level, &rules, data)
                        {
                            eprintln!("{e}");
                        }
                    }
                    editor_level = editor_level.saturating_sub(1).max(1);
                    editor_level_data =
                        crate::generator::load_custom_level(editor_level, &rules);
                }
                if is_key_pressed(KeyCode::X) {
                    if let Some(ref data) = editor_level_data {
                        if let Err(e) =
                            crate::generator::save_custom_level(editor_level, &rules, data)
                        {
                            eprintln!("{e}");
                        }
                    }
                    editor_level = editor_level.saturating_add(1);
                    editor_level_data =
                        crate::generator::load_custom_level(editor_level, &rules);
                }

                // Sprite selection indices
                let player_sprites = assets.sprites_of_kind(crate::assets::SpriteKind::Player);
                let platform_sprites = assets.sprites_of_kind(crate::assets::SpriteKind::Platform);
                let enemy_sprites = assets.sprites_of_kind(crate::assets::SpriteKind::Enemy);
                let collectible_sprites =
                    assets.sprites_of_kind(crate::assets::SpriteKind::Collectible);

                if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::X) {
                    // already handled for level switching above
                }

                // Use A/D style keys for sprite cycling per category
                if is_key_pressed(KeyCode::Comma) {
                    match editor_tool_index {
                        0 => {
                            if !player_sprites.is_empty() {
                                editor_player_index =
                                    (editor_player_index - 1).rem_euclid(player_sprites.len() as i32);
                            }
                        }
                        1 => {
                            if !platform_sprites.is_empty() {
                                editor_platform_index =
                                    (editor_platform_index - 1).rem_euclid(platform_sprites.len() as i32);
                            }
                        }
                        2 => {
                            if !enemy_sprites.is_empty() {
                                editor_enemy_index =
                                    (editor_enemy_index - 1).rem_euclid(enemy_sprites.len() as i32);
                            }
                        }
                        3 => {
                            if !collectible_sprites.is_empty() {
                                editor_collectible_index =
                                    (editor_collectible_index - 1)
                                        .rem_euclid(collectible_sprites.len() as i32);
                            }
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Period) {
                    match editor_tool_index {
                        0 => {
                            if !player_sprites.is_empty() {
                                editor_player_index =
                                    (editor_player_index + 1).rem_euclid(player_sprites.len() as i32);
                            }
                        }
                        1 => {
                            if !platform_sprites.is_empty() {
                                editor_platform_index =
                                    (editor_platform_index + 1).rem_euclid(platform_sprites.len() as i32);
                            }
                        }
                        2 => {
                            if !enemy_sprites.is_empty() {
                                editor_enemy_index =
                                    (editor_enemy_index + 1).rem_euclid(enemy_sprites.len() as i32);
                            }
                        }
                        3 => {
                            if !collectible_sprites.is_empty() {
                                editor_collectible_index =
                                    (editor_collectible_index + 1)
                                        .rem_euclid(collectible_sprites.len() as i32);
                            }
                        }
                        _ => {}
                    }
                }

                // Save current level with S
                if is_key_pressed(KeyCode::S) {
                    if let Some(ref data) = editor_level_data {
                        if let Err(e) =
                            crate::generator::save_custom_level(editor_level, &rules, data)
                        {
                            eprintln!("{e}");
                        }
                    }
                }

                // Place / remove items with mouse (snapped to grid)
                if let Some(ref mut data) = editor_level_data {
                    let mouse = mouse_position();
                    let grid = 64.0f32;
                    let raw_pos = vec2(mouse.0 + editor_camera.x, mouse.1 + editor_camera.y);
                    let world_pos = vec2(
                        (raw_pos.x / grid).round() * grid,
                        (raw_pos.y / grid).round() * grid,
                    );

                    if is_mouse_button_pressed(MouseButton::Left) {
                        match editor_tool_index {
                            0 => {
                                // Player
                                if let Some(sprite) =
                                    player_sprites.get(editor_player_index.rem_euclid(
                                        player_sprites.len().max(1) as i32,
                                    ) as usize)
                                {
                                    data.player_start = Some(crate::generator::CustomLevelEntity {
                                        sprite: sprite.name.clone(),
                                        x: world_pos.x,
                                        y: world_pos.y,
                                    });
                                }
                            }
                            1 => {
                                // Platform
                                if !platform_sprites.is_empty() {
                                    let idx = editor_platform_index
                                        .rem_euclid(platform_sprites.len() as i32)
                                        as usize;
                                    let sprite = platform_sprites[idx];
                                    data.platforms.push(
                                        crate::generator::CustomLevelEntity {
                                            sprite: sprite.name.clone(),
                                            x: world_pos.x,
                                            y: world_pos.y,
                                        },
                                    );
                                }
                            }
                            2 => {
                                // Enemy
                                if !enemy_sprites.is_empty() {
                                    let idx = editor_enemy_index
                                        .rem_euclid(enemy_sprites.len() as i32)
                                        as usize;
                                    let sprite = enemy_sprites[idx];
                                    data.enemies.push(
                                        crate::generator::CustomLevelEntity {
                                            sprite: sprite.name.clone(),
                                            x: world_pos.x,
                                            y: world_pos.y,
                                        },
                                    );
                                }
                            }
                            3 => {
                                // Collectible
                                if !collectible_sprites.is_empty() {
                                    let idx = editor_collectible_index
                                        .rem_euclid(collectible_sprites.len() as i32)
                                        as usize;
                                    let sprite = collectible_sprites[idx];
                                    data.collectibles.push(
                                        crate::generator::CustomLevelCollectible {
                                            sprite: sprite.name.clone(),
                                            x: world_pos.x,
                                            y: world_pos.y,
                                            value: rules.collectible_value,
                                            health: rules.collectible_health_value,
                                        },
                                    );
                                }
                            }
                            4 => {
                                // Eraser: remove nearest from any category
                                remove_nearest_in_level(data, world_pos, 32.0);
                            }
                            _ => {}
                        }
                    }

                    if is_mouse_button_pressed(MouseButton::Right) {
                        // Remove nearest in current category only
                        remove_nearest_in_level_category(
                            data,
                            world_pos,
                            32.0,
                            editor_tool_index,
                        );
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    if let Some(ref data) = editor_level_data {
                        if let Err(e) =
                            crate::generator::save_custom_level(editor_level, &rules, data)
                        {
                            eprintln!("{e}");
                        }
                    }
                    state = GameState::MainMenu;
                }
            }
            GameState::CartridgeMenu => {
                let up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
                let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);

                let entry_count = (cartridge_files.len() as i32).saturating_add(1).max(1);

                if up {
                    cartridge_index = (cartridge_index - 1).rem_euclid(entry_count);
                }
                if down {
                    cartridge_index = (cartridge_index + 1).rem_euclid(entry_count);
                }

                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::MainMenu;
                } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    // Last entry is always "Back"
                    let back_index = cartridge_files.len() as i32;
                    if cartridge_files.is_empty() || cartridge_index == back_index {
                        state = GameState::MainMenu;
                    } else if let Some(name) =
                        cartridge_files.get(cartridge_index as usize)
                    {
                        match load_run_cartridge(name) {
                            Ok(cart) => {
                                rules = cart.rules;
                                seed = cart.seed;
                                rules.seed = Some(seed);

                                // Update resolution and window size
                                resolution_index = rules
                                    .resolution_index
                                    .min(RESOLUTIONS.len().saturating_sub(1))
                                    as i32;
                                let (w, h) =
                                    RESOLUTIONS[resolution_index as usize];
                                macroquad::window::request_new_screen_size(w, h);

                                update_music_volume(&sounds, &rules);

                                // Start a fresh run with this configuration
                                level = 1;
                                total_collected = 0;
                                run_score = 0;
                                run_time = 0.0;
                                level_rng =
                                    StdRng::seed_from_u64(seed ^ 0x9E3779B97F4A7C15);
                                scene =
                                    make_scene(&assets, &rules, level, &mut level_rng);
                                state = GameState::Playing;
                            }
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                    }
                }
            }
            GameState::Help => {
                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                {
                    state = GameState::MainMenu;
                }
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::C) {
                    if let Err(e) =
                        export_run_cartridge(&rules, seed, level, run_score, run_time)
                    {
                        eprintln!("{e}");
                    }
                } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    level = 1;
                    total_collected = 0;
                    run_score = 0;
                    run_time = 0.0;
                    level_rng = StdRng::seed_from_u64(seed ^ 0x9E3779B97F4A7C15);
                    scene = make_scene(&assets, &rules, level, &mut level_rng);
                    state = GameState::Playing;
                }
            }
            GameState::Won => {
                if is_key_pressed(KeyCode::C) {
                    if let Err(e) =
                        export_run_cartridge(&rules, seed, level, run_score, run_time)
                    {
                        eprintln!("{e}");
                    }
                } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    level = 1;
                    total_collected = 0;
                    run_score = 0;
                    run_time = 0.0;
                    level_rng = StdRng::seed_from_u64(seed ^ 0x9E3779B97F4A7C15);
                    scene = make_scene(&assets, &rules, level, &mut level_rng);
                    state = GameState::Playing;
                }
            }
        }

        clear_background(BLACK);

        if let GameState::LevelEditor = state {
            // Simple 2D editor draw with default camera
            set_default_camera();

            // Draw grid
            let sw = screen_width();
            let sh = screen_height();
            let grid = 64.0f32;
            let start_x = (editor_camera.x / grid).floor() * grid - editor_camera.x;
            let start_y = (editor_camera.y / grid).floor() * grid - editor_camera.y;

            let vertical_lines = (sw / grid).ceil() as i32 + 2;
            let horizontal_lines = (sh / grid).ceil() as i32 + 2;

            for i in 0..vertical_lines {
                let x = start_x + i as f32 * grid;
                draw_line(x, 0.0, x, sh, 1.0, Color::new(0.2, 0.2, 0.2, 0.7));
            }
            for i in 0..horizontal_lines {
                let y = start_y + i as f32 * grid;
                draw_line(0.0, y, sw, y, 1.0, Color::new(0.2, 0.2, 0.2, 0.7));
            }

            // Draw level entities from editor_level_data
            if let Some(ref data) = editor_level_data {
                let draw_entity = |sprite_name: &str, x: f32, y: f32| {
                    if let Some(sprite) = assets
                        .sprite_by_kind_and_name(crate::assets::SpriteKind::Platform, sprite_name)
                    {
                        let tex = &sprite.texture;
                        let dest_size =
                            vec2(tex.width() * rules.sprite_scale, tex.height() * rules.sprite_scale);
                        let sx = x - editor_camera.x - dest_size.x / 2.0;
                        let sy = y - editor_camera.y - dest_size.y / 2.0;
                        draw_texture_ex(
                            tex,
                            sx,
                            sy,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(dest_size),
                                ..Default::default()
                            },
                        );
                    }
                };

                // Platforms
                for p in &data.platforms {
                    if let Some(sprite) = assets.sprite_by_kind_and_name(
                        crate::assets::SpriteKind::Platform,
                        &p.sprite,
                    ) {
                        let tex = &sprite.texture;
                        let dest_size =
                            vec2(tex.width() * rules.sprite_scale, tex.height() * rules.sprite_scale);
                        let sx = p.x - editor_camera.x - dest_size.x / 2.0;
                        let sy = p.y - editor_camera.y - dest_size.y / 2.0;
                        draw_texture_ex(
                            tex,
                            sx,
                            sy,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(dest_size),
                                ..Default::default()
                            },
                        );
                    }
                }

                // Enemies
                for e in &data.enemies {
                    if let Some(sprite) = assets.sprite_by_kind_and_name(
                        crate::assets::SpriteKind::Enemy,
                        &e.sprite,
                    ) {
                        let tex = &sprite.texture;
                        let dest_size =
                            vec2(tex.width() * rules.sprite_scale, tex.height() * rules.sprite_scale);
                        let sx = e.x - editor_camera.x - dest_size.x / 2.0;
                        let sy = e.y - editor_camera.y - dest_size.y / 2.0;
                        draw_texture_ex(
                            tex,
                            sx,
                            sy,
                            RED,
                            DrawTextureParams {
                                dest_size: Some(dest_size),
                                ..Default::default()
                            },
                        );
                    }
                }

                // Collectibles
                for c in &data.collectibles {
                    if let Some(sprite) = assets.sprite_by_kind_and_name(
                        crate::assets::SpriteKind::Collectible,
                        &c.sprite,
                    ) {
                        let tex = &sprite.texture;
                        let dest_size =
                            vec2(tex.width() * rules.sprite_scale, tex.height() * rules.sprite_scale);
                        let sx = c.x - editor_camera.x - dest_size.x / 2.0;
                        let sy = c.y - editor_camera.y - dest_size.y / 2.0;
                        draw_texture_ex(
                            tex,
                            sx,
                            sy,
                            BLUE,
                            DrawTextureParams {
                                dest_size: Some(dest_size),
                                ..Default::default()
                            },
                        );
                    }
                }

                // Player start
                if let Some(ref start) = data.player_start {
                    if let Some(sprite) = assets.sprite_by_kind_and_name(
                        crate::assets::SpriteKind::Player,
                        &start.sprite,
                    ) {
                        let tex = &sprite.texture;
                        let dest_size =
                            vec2(tex.width() * rules.sprite_scale, tex.height() * rules.sprite_scale);
                        let sx = start.x - editor_camera.x - dest_size.x / 2.0;
                        let sy = start.y - editor_camera.y - dest_size.y / 2.0;
                        draw_texture_ex(
                            tex,
                            sx,
                            sy,
                            GREEN,
                            DrawTextureParams {
                                dest_size: Some(dest_size),
                                ..Default::default()
                            },
                        );
                    }
                }
            }

            // HUD for editor
            let tool_name = match editor_tool_index {
                0 => "Player",
                1 => "Platform",
                2 => "Enemy",
                3 => "Collectible",
                4 => "Eraser",
                _ => "Unknown",
            };
            let hud = format!(
                "Level Editor - Level {} | Tool: {} | Scale: {:.2}x",
                editor_level, tool_name, editor_preview_scale
            );
            draw_text(&hud, 16.0, 28.0, 30.0, YELLOW);
            let hint = "Move: WASD/arrows  | Q/E: tool  | ,/. : sprite  | Mouse wheel: preview size  | Z/X: level  | LMB: place  | RMB: erase  | S: save  | Esc: back";
            draw_text(hint, 16.0, 64.0, 20.0, GRAY);

            // Preview sprite under mouse
            if let Some(ref data) = editor_level_data {
                let mouse = mouse_position();
                let preview_pos = vec2(mouse.0, mouse.1);
                let color = match editor_tool_index {
                    0 => GREEN,
                    1 => WHITE,
                    2 => RED,
                    3 => BLUE,
                    _ => GRAY,
                };

                let draw_preview = |tex: &Texture2D| {
                    let base_w = tex.width();
                    let base_h = tex.height();
                    let scale = rules.sprite_scale * editor_preview_scale;
                    let dest_size = vec2(base_w * scale, base_h * scale);
                    draw_texture_ex(
                        tex,
                        preview_pos.x - dest_size.x / 2.0,
                        preview_pos.y - dest_size.y / 2.0,
                        color,
                        DrawTextureParams {
                            dest_size: Some(dest_size),
                            ..Default::default()
                        },
                    );
                };

                match editor_tool_index {
                    0 => {
                        if let Some(sprite) = assets
                            .sprites_of_kind(crate::assets::SpriteKind::Player)
                            .get(editor_player_index.rem_euclid(
                                assets
                                    .sprites_of_kind(crate::assets::SpriteKind::Player)
                                    .len()
                                    .max(1) as i32,
                            ) as usize)
                        {
                            draw_preview(&sprite.texture);
                        }
                    }
                    1 => {
                        let list = assets.sprites_of_kind(crate::assets::SpriteKind::Platform);
                        if !list.is_empty() {
                            let idx = editor_platform_index.rem_euclid(list.len() as i32) as usize;
                            let sprite = list[idx];
                            draw_preview(&sprite.texture);
                        }
                    }
                    2 => {
                        let list = assets.sprites_of_kind(crate::assets::SpriteKind::Enemy);
                        if !list.is_empty() {
                            let idx = editor_enemy_index.rem_euclid(list.len() as i32) as usize;
                            let sprite = list[idx];
                            draw_preview(&sprite.texture);
                        }
                    }
                    3 => {
                        let list =
                            assets.sprites_of_kind(crate::assets::SpriteKind::Collectible);
                        if !list.is_empty() {
                            let idx =
                                editor_collectible_index.rem_euclid(list.len() as i32) as usize;
                            let sprite = list[idx];
                            draw_preview(&sprite.texture);
                        }
                    }
                    _ => {}
                }
            }
        } else {
            // Normal game rendering
            // World camera following the player horizontally and vertically
            let sw = screen_width();
            let sh = screen_height();
            let player_pos = scene
                .player_position()
                .unwrap_or(vec2(scene.world_width / 2.0, sh / 2.0));
            let half_w = sw / 2.0;
            let half_h = sh / 2.0;
            let min_cam_x = half_w;
            let max_cam_x = (scene.world_width - half_w).max(min_cam_x);
            let min_cam_y = half_h;
            let world_h = scene.world_height.max(sh);
            let max_cam_y = (world_h - half_h).max(min_cam_y);
            let cam_x = player_pos.x.clamp(min_cam_x, max_cam_x);
            let cam_y = player_pos.y.clamp(min_cam_y, max_cam_y);

            // Parallax background: move slower than the world (0.5x), but keep within bounds
            let parallax_cam_x = half_w + (cam_x - half_w) * 0.5;
            let parallax_cam_y = half_h + (cam_y - half_h) * 0.5;

            let parallax_camera = Camera2D {
                target: vec2(parallax_cam_x, parallax_cam_y),
                zoom: vec2(2.0 / sw, 2.0 / sh),
                ..Default::default()
            };
            set_camera(&parallax_camera);
            scene.draw_background();

            // World camera
            let camera = Camera2D {
                target: vec2(cam_x, cam_y),
                zoom: vec2(2.0 / sw, 2.0 / sh),
                ..Default::default()
            };
            set_camera(&camera);
            scene.draw_world();
            if rules.debug_overlay {
                scene.debug_draw();
            }

            set_default_camera();

            let hud_text = format!(
                "Level: {} | HP: {}/{} | Progress: {}/{}",
                level,
                scene.player_health,
                scene.player_max_health,
                total_collected,
                rules.collectibles_for_level_up
            );

            let controls_hint = match rules.control_scheme.to_lowercase().as_str() {
                "wasd" => "Controls: A/D move, Space/W jump, R to regenerate",
                "arrows" => "Controls: Left/Right move, Up jump, R to regenerate",
                "custom" => "Controls: custom bindings (see rules.json), R to regenerate",
                _ => "Controls: A/D or arrows move, Space jump, R to regenerate",
            };
            draw_text(
                controls_hint,
                16.0,
                24.0,
                24.0,
                YELLOW,
            );

            draw_text(
                &hud_text,
                16.0,
                52.0,
                24.0,
                YELLOW,
            );

            // Boss / event level label
            let is_boss_level = rules.boss_level_interval > 0
                && level > 0
                && level % rules.boss_level_interval == 0;
            if is_boss_level {
                let label = match rules.boss_mode.to_lowercase().as_str() {
                    "collect_fest" | "collectfest" | "collect" => "EVENT: COLLECT FEST",
                    "boss_enemy" | "boss" => "BOSS LEVEL",
                    _ => "EVENT LEVEL",
                };
                let sw = screen_width();
                draw_text(
                    label,
                    sw * 0.5 - 140.0,
                    32.0,
                    28.0,
                    ORANGE,
                );
            }

            if rules.show_fps {
                let fps_text = format!("FPS: {}", get_fps());
                draw_text(
                    &fps_text,
                    screen_width() - 140.0,
                    24.0,
                    20.0,
                    GREEN,
                );
            }

            if rules.debug_overlay {
                let dbg = format!("Seed: {} | Level: {}", seed, level);
                draw_text(
                    &dbg,
                    16.0,
                    screen_height() - 16.0,
                    18.0,
                    GREEN,
                );
            }
        }

        match state {
            GameState::MainMenu => {
                let title = "Random Platformer Engine";
                let opt1 = "Start Game";
                let opt2 = "Custom Levels";
                let opt3 = "Level Editor";
                let opt4 = "Play Cartridge";
                let opt5 = "Settings";
                let opt6 = "Help";
                let opt7 = "Quit";

                let center_x = screen_width() * 0.5;
                let center_y = screen_height() * 0.5;

                draw_text(
                    title,
                    center_x - 220.0,
                    center_y - 80.0,
                    36.0,
                    YELLOW,
                );

                let color1 = if menu_index == 0 { GREEN } else { GRAY };
                let color2 = if menu_index == 1 { GREEN } else { GRAY };
                let color3 = if menu_index == 2 { GREEN } else { GRAY };
                let color4 = if menu_index == 3 { GREEN } else { GRAY };
                let color5 = if menu_index == 4 { GREEN } else { GRAY };
                let color6 = if menu_index == 5 { GREEN } else { GRAY };
                let color7 = if menu_index == 6 { GREEN } else { GRAY };

                draw_text(
                    opt1,
                    center_x - 80.0,
                    center_y,
                    28.0,
                    color1,
                );
                draw_text(
                    opt2,
                    center_x - 80.0,
                    center_y + 40.0,
                    28.0,
                    color2,
                );
                draw_text(
                    opt3,
                    center_x - 80.0,
                    center_y + 80.0,
                    28.0,
                    color3,
                );
                draw_text(
                    opt4,
                    center_x - 80.0,
                    center_y + 120.0,
                    28.0,
                    color4,
                );
                draw_text(
                    opt5,
                    center_x - 80.0,
                    center_y + 160.0,
                    28.0,
                    color5,
                );
                draw_text(
                    opt6,
                    center_x - 80.0,
                    center_y + 200.0,
                    28.0,
                    color6,
                );
                draw_text(
                    opt7,
                    center_x - 80.0,
                    center_y + 240.0,
                    28.0,
                    color7,
                );
            }
            GameState::Paused => {
                let title = "Paused";
                let options = ["Resume", "Settings", "Main Menu", "Quit"];
                let center_x = screen_width() * 0.5;
                let center_y = screen_height() * 0.5;

                draw_text(
                    title,
                    center_x - 80.0,
                    center_y - 80.0,
                    36.0,
                    YELLOW,
                );

                for (i, label) in options.iter().enumerate() {
                    let color = if pause_index == i as i32 { GREEN } else { GRAY };
                    draw_text(
                        label,
                        center_x - 90.0,
                        center_y + i as f32 * 32.0,
                        26.0,
                        color,
                    );
                }
            }
            GameState::Settings => {
                let title = "Settings";
                let center_x = screen_width() * 0.5;
                let center_y = screen_height() * 0.5;

                draw_text(
                    title,
                    center_x - 80.0,
                    center_y - 80.0,
                    36.0,
                    YELLOW,
                );

                let (res_w, res_h) =
                    RESOLUTIONS[resolution_index.clamp(0, RESOLUTIONS.len() as i32 - 1) as usize];

                let controls_label = match rules.control_scheme.to_lowercase().as_str() {
                    "wasd" => "WASD",
                    "arrows" => "Arrows",
                    "custom" => "Custom",
                    _ => "Both",
                };

                let entries = [
                    format!("Resolution: {}x{}", res_w as i32, res_h as i32),
                    format!("Controls: {}", controls_label),
                    "Rebind Controls".to_string(),
                    format!("Enemies: {}", if rules.enemy_enabled { "On" } else { "Off" }),
                    format!(
                        "Collectibles: {}",
                        if rules.collectible_enabled { "On" } else { "Off" }
                    ),
                    format!("Sound FX: {}", if rules.sfx_enabled { "On" } else { "Off" }),
                    format!("Music: {}", if rules.music_enabled { "On" } else { "Off" }),
                    format!("VSync: {}", if rules.vsync_enabled { "On" } else { "Off" }),
                    format!("FPS Counter: {}", if rules.show_fps { "On" } else { "Off" }),
                    format!(
                        "Debug Overlay: {}",
                        if rules.debug_overlay { "On" } else { "Off" }
                    ),
                    "Advanced Rules...".to_string(),
                    "Presets...".to_string(),
                    "Export Run Config".to_string(),
                    "Back".to_string(),
                ];

                for (i, text) in entries.iter().enumerate() {
                    let color = if settings_index == i as i32 { GREEN } else { GRAY };
                    draw_text(
                        text,
                        center_x - 140.0,
                        center_y + i as f32 * 32.0,
                        24.0,
                        color,
                    );
                }
            }
            GameState::RulesEditor => {
                let cx = screen_width() * 0.5;
                let cy = screen_height() * 0.5;

                let title = "Advanced Rules Editor";
                draw_text(title, cx - 200.0, cy - 120.0, 32.0, YELLOW);

                let note =
                    "Changes here are saved to rules.json.\nPlease restart the game after changing these settings.";
                draw_text(note, cx - 260.0, cy - 80.0, 18.0, ORANGE);

                let mode_label = format!("Mode: {}", rules.mode);
                let items = [
                    mode_label,
                    format!("Collectibles/Level Up: {}", rules.collectibles_for_level_up),
                    format!("Max Level: {}", rules.max_level),
                    format!("Player Start HP: {}", rules.player_start_health),
                    format!("Player Max HP: {}", rules.player_max_health),
                    format!("Enemy Contact Damage: {}", rules.enemy_contact_damage),
                    format!("Move Speed: {:.0}", rules.player_move_speed),
                    format!("Jump Strength: {:.0}", rules.player_jump_strength),
                    format!("Gravity: {:.0}", rules.gravity),
                    format!("Enemy Speed: {:.0}", rules.enemy_speed),
                    format!("Boss Level Interval: {}", rules.boss_level_interval),
                    format!(
                        "Enemy Shooting: {}",
                        if rules.enemy_shoot_enabled { "On" } else { "Off" }
                    ),
                ];

                for (i, text) in items.iter().enumerate() {
                    let color = if rules_menu_index == i as i32 { GREEN } else { GRAY };
                    draw_text(
                        text,
                        cx - 260.0,
                        cy - 20.0 + i as f32 * 26.0,
                        22.0,
                        color,
                    );
                }

                let hint = "Use Up/Down to select, Left/Right to change, Esc to go back.";
                draw_text(hint, cx - 260.0, cy + 220.0, 18.0, GRAY);
            }
            GameState::RebindingControls => {
                // Input handling for rebinding is done in the main state loop;
                // here we just draw the overlay.
                let cx = screen_width() * 0.5;
                let cy = screen_height() * 0.5;

                let step_label = match rebind_step {
                    0 => "Move Left (primary)",
                    1 => "Move Left (alt)",
                    2 => "Move Right (primary)",
                    3 => "Move Right (alt)",
                    4 => "Jump (primary)",
                    5 => "Jump (alt)",
                    _ => "Done",
                };

                let title = "Rebind Controls";
                let prompt = format!("Press a key for {}", step_label);
                let hint = "Press Esc to cancel";

                draw_text(title, cx - 140.0, cy - 80.0, 36.0, YELLOW);
                draw_text(&prompt, cx - 220.0, cy, 24.0, WHITE);
                draw_text(hint, cx - 150.0, cy + 40.0, 20.0, GRAY);
            }
            GameState::Help => {
                let cx = screen_width() * 0.5;
                let cy = screen_height() * 0.5;

                let title = "Help / Controls";
                draw_text(title, cx - 140.0, cy - 100.0, 36.0, YELLOW);

                let scheme = rules.control_scheme.to_lowercase();
                let lines: [&str; 4] = match scheme.as_str() {
                    "wasd" => [
                        "Movement: A/D to move",
                        "Jump: Space or W",
                        "Pause: Esc",
                        "Press Enter or Esc to return",
                    ],
                    "arrows" => [
                        "Movement: Left/Right arrows",
                        "Jump: Up arrow",
                        "Pause: Esc",
                        "Press Enter or Esc to return",
                    ],
                    "custom" => [
                        "Movement & jump use custom bindings",
                        "See Settings > Rebind and rules.json",
                        "Pause: Esc",
                        "Press Enter or Esc to return",
                    ],
                    _ => [
                        "Movement: A/D or Left/Right arrows",
                        "Jump: Space or W / Up arrow",
                        "Pause: Esc",
                        "Press Enter or Esc to return",
                    ],
                };

                for (i, text) in lines.iter().enumerate() {
                    draw_text(
                        text,
                        cx - 260.0,
                        cy - 40.0 + i as f32 * 30.0,
                        24.0,
                        WHITE,
                    );
                }
            }
            GameState::PresetsMenu => {
                let cx = screen_width() * 0.5;
                let cy = screen_height() * 0.5;
                let title = "Presets";
                let options = [
                    "Load Preset 1",
                    "Load Preset 2",
                    "Load Preset 3",
                    "Save Preset 1",
                    "Save Preset 2",
                    "Save Preset 3",
                    "Back",
                ];

                draw_text(title, cx - 80.0, cy - 80.0, 36.0, YELLOW);
                for (i, label) in options.iter().enumerate() {
                    let color = if presets_index == i as i32 { GREEN } else { GRAY };
                    draw_text(
                        label,
                        cx - 140.0,
                        cy + i as f32 * 32.0,
                        24.0,
                        color,
                    );
                }
            }
            GameState::CartridgeMenu => {
                let cx = screen_width() * 0.5;
                let cy = screen_height() * 0.5;
                let title = "Play Cartridge";
                draw_text(title, cx - 140.0, cy - 80.0, 36.0, YELLOW);

                if cartridge_files.is_empty() {
                    let msg = "No run cartridges found.";
                    let hint = "Press Enter or Esc to return";
                    draw_text(msg, cx - 200.0, cy, 24.0, GRAY);
                    draw_text(hint, cx - 260.0, cy + 40.0, 20.0, GRAY);
                } else {
                    for (i, name) in cartridge_files.iter().enumerate() {
                        let color = if cartridge_index == i as i32 {
                            GREEN
                        } else {
                            GRAY
                        };
                        draw_text(
                            name,
                            cx - 260.0,
                            cy - 20.0 + i as f32 * 28.0,
                            22.0,
                            color,
                        );
                    }
                    let back_y = cy - 20.0 + cartridge_files.len() as f32 * 28.0 + 24.0;
                    let back_color = if cartridge_index == cartridge_files.len() as i32 {
                        GREEN
                    } else {
                        GRAY
                    };
                    draw_text("Back", cx - 60.0, back_y, 22.0, back_color);
                }
            }
            GameState::GameOver => {
                let cx = screen_width() * 0.5;
                let cy = screen_height() * 0.5;
                let msg = "GAME OVER";
                let sub = "Press Enter or Space to restart";
                draw_text(msg, cx - 140.0, cy, 40.0, RED);
                draw_text(sub, cx - 220.0, cy + 40.0, 24.0, YELLOW);

                let summary1 = format!("Score: {}", run_score);
                let summary2 = format!("Level reached: {}", level);
                let summary3 = format!("Time: {:.1}s", run_time);
                draw_text(&summary1, cx - 140.0, cy + 80.0, 24.0, WHITE);
                draw_text(&summary2, cx - 140.0, cy + 110.0, 24.0, WHITE);
                draw_text(&summary3, cx - 140.0, cy + 140.0, 24.0, WHITE);
            }
            GameState::Won => {
                let cx = screen_width() * 0.5;
                let cy = screen_height() * 0.5;
                let msg = "YOU WIN!";
                let sub = "Press Enter or Space to play again";
                draw_text(msg, cx - 130.0, cy, 40.0, GREEN);
                draw_text(sub, cx - 260.0, cy + 40.0, 24.0, YELLOW);

                let summary1 = format!("Score: {}", run_score);
                let summary2 = format!("Level reached: {}", level);
                let summary3 = format!("Time: {:.1}s", run_time);
                draw_text(&summary1, cx - 140.0, cy + 80.0, 24.0, WHITE);
                draw_text(&summary2, cx - 140.0, cy + 110.0, 24.0, WHITE);
                draw_text(&summary3, cx - 140.0, cy + 140.0, 24.0, WHITE);
            }
            GameState::Playing => {}
            GameState::LevelEditor => {}
        }

        if rules.vsync_enabled {
            let target = std::time::Duration::from_micros(16_666);
            let elapsed = frame_start.elapsed();
            if elapsed < target {
                std::thread::sleep(target - elapsed);
            }
        }

        next_frame().await;
    }
}

fn update_music_volume(sounds: &Sounds, rules: &GameRules) {
    if let Some(m) = sounds.music.as_ref() {
        let volume = if rules.music_enabled { rules.music_volume } else { 0.0 };
        macroquad::audio::set_sound_volume(m, volume);
    }
}

fn keycode_to_rules_string(key: KeyCode) -> Option<String> {
    use KeyCode::*;
    let name = match key {
        A => "A",
        B => "B",
        C => "C",
        D => "D",
        E => "E",
        F => "F",
        G => "G",
        H => "H",
        I => "I",
        J => "J",
        K => "K",
        L => "L",
        M => "M",
        N => "N",
        O => "O",
        P => "P",
        Q => "Q",
        R => "R",
        S => "S",
        T => "T",
        U => "U",
        V => "V",
        W => "W",
        X => "X",
        Y => "Y",
        Z => "Z",
        Left => "Left",
        Right => "Right",
        Up => "Up",
        Down => "Down",
        Space => "Space",
        Escape => "Escape",
        _ => return None,
    };
    Some(name.to_string())
}

fn make_scene(
    assets: &Assets,
    rules: &GameRules,
    level: u32,
    rng: &mut StdRng,
) -> Scene {
    let screen_w = screen_width();
    let screen_h = screen_height();
    let world_width_screens = rules.world_width_screens.max(1.0);
    let world_height_screens = rules.world_height_screens.max(1.0);
    let world_w = screen_w * world_width_screens;
    let world_h = screen_h * world_height_screens;
    let screen_size = vec2(world_w, world_h);
    generate_scene(assets, rules, level, screen_size, rng)
}

fn remove_nearest_in_level(
    level: &mut crate::generator::CustomLevel,
    pos: Vec2,
    radius: f32,
) {
    remove_nearest_in_level_category(level, pos, radius, -1);
}

fn remove_nearest_in_level_category(
    level: &mut crate::generator::CustomLevel,
    pos: Vec2,
    radius: f32,
    category: i32,
) {
    let r2 = radius * radius;

    let mut best: Option<(usize, i32)> = None;
    let mut best_dist2 = r2;

    // platforms (category 1 or any if category < 0)
    if category < 0 || category == 1 {
        for (i, p) in level.platforms.iter().enumerate() {
            let d2 = (vec2(p.x, p.y) - pos).length_squared();
            if d2 <= best_dist2 {
                best_dist2 = d2;
                best = Some((i, 1));
            }
        }
    }

    // enemies (category 2 or any)
    if category < 0 || category == 2 {
        for (i, e) in level.enemies.iter().enumerate() {
            let d2 = (vec2(e.x, e.y) - pos).length_squared();
            if d2 <= best_dist2 {
                best_dist2 = d2;
                best = Some((i, 2));
            }
        }
    }

    // collectibles (category 3 or any)
    if category < 0 || category == 3 {
        for (i, c) in level.collectibles.iter().enumerate() {
            let d2 = (vec2(c.x, c.y) - pos).length_squared();
            if d2 <= best_dist2 {
                best_dist2 = d2;
                best = Some((i, 3));
            }
        }
    }

    // player start (category 0 or any)
    if (category < 0 || category == 0) && level.player_start.is_some() {
        if let Some(ref start) = level.player_start {
            let d2 = (vec2(start.x, start.y) - pos).length_squared();
            if d2 <= best_dist2 {
                best = Some((0, 0));
            }
        }
    }

    if let Some((index, cat)) = best {
        match cat {
            0 => level.player_start = None,
            1 => {
                level.platforms.remove(index);
            }
            2 => {
                level.enemies.remove(index);
            }
            3 => {
                level.collectibles.remove(index);
            }
            _ => {}
        }
    }
}

fn random_seed_from_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // mix seconds and nanos a bit
    now.as_secs() ^ (now.subsec_nanos() as u64).rotate_left(32)
}

async fn load_sounds(rules: &GameRules) -> Sounds {
    use macroquad::audio::load_sound;

    async fn load_optional(paths: &[&str]) -> Option<macroquad::audio::Sound> {
        for path in paths {
            match load_sound(path).await {
                Ok(s) => return Some(s),
                Err(e) => {
                    eprintln!("Failed to load sound {path}: {e}");
                }
            }
        }
        None
    }

    let jump = load_optional(&["assets/sounds/jump.ogg", "assets/sounds/jump.wav"]).await;
    let hit = load_optional(&["assets/sounds/hit.ogg", "assets/sounds/hit.wav"]).await;
    let pickup =
        load_optional(&["assets/sounds/pickup.ogg", "assets/sounds/pickup.wav"]).await;
    let music =
        load_optional(&["assets/sounds/music.ogg", "assets/sounds/music.wav"]).await;

    if rules.music_enabled {
        if let Some(m) = music.as_ref() {
            macroquad::audio::play_sound(
                m,
                macroquad::audio::PlaySoundParams {
                    looped: true,
                    volume: rules.music_volume,
                },
            );
        }
    }

    Sounds {
        jump,
        hit,
        pickup,
        music,
    }
}
