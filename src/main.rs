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

    loop {
        let frame_start = std::time::Instant::now();
        let dt = get_frame_time();

        match state {
            GameState::MainMenu => {
                let up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
                let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);

                if up {
                    menu_index = (menu_index - 1).rem_euclid(6);
                }
                if down {
                    menu_index = (menu_index + 1).rem_euclid(6);
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
                            // Level Editor
                            editor_level = 1;
                            editor_level_data =
                                crate::generator::load_custom_level(editor_level, &rules);
                            state = GameState::RulesEditor; // temporary placeholder
                        }
                        2 => {
                            // Play Cartridge
                            cartridge_files = list_run_cartridges().unwrap_or_else(|e| {
                                eprintln!("{e}");
                                Vec::new()
                            });
                            cartridge_index = 0;
                            state = GameState::CartridgeMenu;
                        }
                        3 => {
                            settings_return_to = GameState::MainMenu;
                            state = GameState::Settings;
                        }
                        4 => {
                            state = GameState::Help;
                        }
                        5 => {
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

        match state {
            GameState::MainMenu => {
                let title = "Random Platformer Engine";
                let opt1 = "Start Game";
                let opt2 = "Play Cartridge";
                let opt3 = "Settings";
                let opt4 = "Help";
                let opt5 = "Quit";

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
