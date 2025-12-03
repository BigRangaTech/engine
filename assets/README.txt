Place your game assets in this folder.

Expected structure for the current platformer prototype:

- Player sprites (PNG):
  assets/sprites/player/*.png

- Enemy sprites (PNG):
  assets/sprites/enemies/*.png

- Collectible sprites (PNG):
  assets/sprites/collectibles/*.png

- Goal collectible sprites (PNG, for win condition if you choose to use them):
  assets/sprites/goals/*.png

- Platform tiles (PNG):
  assets/tiles/platforms/*.png

- Backgrounds (PNG):
  assets/backgrounds/*.png

- Optional sounds (OGG/WAV supported by macroquad):
  assets/sounds/jump.ogg or jump.wav
  assets/sounds/hit.ogg or hit.wav
  assets/sounds/pickup.ogg or pickup.wav

Optional rules file (JSON) at:
  assets/config/rules.json

Example rules.json:
{
  "seed": null,
  "mode": "normal",
  "control_scheme": "both",
  "enemy_enabled": true,
  "collectible_enabled": true,
  "sfx_enabled": true,
  "music_enabled": true,
  "show_fps": false,
  "vsync_enabled": true,
  "collectibles_for_level_up": 100,
  "max_level": 1000,
  "player_start_health": 5,
  "player_max_health": 100,
  "enemy_contact_damage": 1,
  "hit_invincibility_duration": 0.5,
  "player_move_speed": 220.0,
  "player_jump_strength": 420.0,
  "gravity": 900.0,
  "enemy_speed": 80.0,
  "enemy_gravity_scale": 0.0,
  "enemy_spawn_rows": 2,
  "min_enemies": 3,
  "max_enemies": 8,
  "min_collectibles": 3,
  "max_collectibles": 6,
  "platform_rows": 4,
  "min_platforms_per_row": 3,
  "max_platforms_per_row": 6,
  "platform_min_gap_x": 80.0,
  "platform_max_gap_x": 200.0,
  "platform_min_y": 0.3,
  "platform_max_y": 0.85,
  "ground_row_enabled": true,
  "enemy_on_platform_chance": 0.5,
  "collectible_value": 1,
  "rare_collectible_chance": 0.1,
  "rare_collectible_value": 5,
  "collectible_health_value": 1,
  "rare_collectible_health_value": 5,
  "collectibles_follow_path": false,
  "collectible_cluster_size_min": 1,
  "collectible_cluster_size_max": 3,
  "theme": "default",
  "background_variants": 2,
  "sprite_scale": 1.0
}

At runtime the engine will:
- Load any PNGs in those folders,
- Randomly pick one player sprite and one background,
- Build several rows of platforms from the platform tiles,
- Spawn a random number of enemies between min/max,
- Apply platformer physics (gravity, left/right movement, jumping), tuned by rules.json,
- Press R in-game to regenerate a new random level layout,
- Progress to the next level whenever your progress reaches `collectibles_for_level_up`, up to `max_level`,
- Lose health when colliding with enemies, and pick up health from collectibles up to `player_max_health`,
- Reach `max_level` and fill the progress bar to trigger a simple win screen,
- On death, see a game over screen and press Enter/Space to restart the run.

Controls:
- Move: A/D or Left/Right arrows
- Jump: Space, W, or Up arrow
- Regenerate level: R

Seed behaviour:
- If `seed` in `assets/config/rules.json` is `null` or missing, a new random seed is chosen every time you run the game, so art + levels look completely different each run.
- If you set `seed` to a number (for example `12345`), the generated placeholder art and the sequence of random levels will be reproducible for that seed.

Generating placeholder assets with Rust:
- After installing Rust and in the project root, run:
  cargo run --bin gen_assets
- This will create a set of simple colored PNGs in the correct folders so you can immediately test the engine.
 - When running the main game binary, placeholder sounds (simple beeps) are also generated in `assets/sounds` as WAV files if not already present, and you can replace them with your own sound effects.
