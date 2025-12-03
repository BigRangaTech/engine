use macroquad::prelude::*;
use macroquad::audio::{self, Sound, PlaySoundParams};

pub enum EntityKind {
    Player,
    Enemy,
    Collectible,
    Projectile,
}

pub struct Entity {
    pub kind: EntityKind,
    pub texture: Texture2D,
    pub position: Vec2,
    pub velocity: Vec2,
    pub value: u32,
    pub health_value: u32,
    pub base_position: Vec2,
    pub phase: f32,
    pub jumping: bool,
}

pub struct Sounds {
    pub jump: Option<Sound>,
    pub hit: Option<Sound>,
    pub pickup: Option<Sound>,
    pub music: Option<Sound>,
}

pub struct InputConfig {
    pub move_left_primary: KeyCode,
    pub move_left_alt: Option<KeyCode>,
    pub move_right_primary: KeyCode,
    pub move_right_alt: Option<KeyCode>,
    pub jump_primary: KeyCode,
    pub jump_alt: Option<KeyCode>,
}

pub struct Platform {
    pub texture: Texture2D,
    pub position: Vec2,
    pub base_position: Vec2,
    pub phase: f32,
    pub moving: bool,
    pub vertical: bool,
}

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
}

pub struct Scene {
    pub background: Option<Texture2D>,
    pub entities: Vec<Entity>,
    pub platforms: Vec<Platform>,
    pub score: u32,
    pub move_speed: f32,
    pub jump_strength: f32,
    pub gravity: f32,
    pub enemy_speed: f32,
    pub enemy_gravity_scale: f32,
    pub sprite_scale: f32,
    pub player_health: u32,
    pub player_max_health: u32,
    pub enemy_contact_damage: u32,
    pub hit_invincibility_duration: f32,
    pub hit_flash_enabled: bool,
    pub hit_timer: f32,
    pub player_dead: bool,
    pub input: InputConfig,
    pub world_width: f32,
    pub world_height: f32,
    pub time: f32,
    pub fall_respawn_offset: f32,
    pub jump_sfx_volume: f32,
    pub hit_sfx_volume: f32,
    pub pickup_sfx_volume: f32,
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
    pub enemy_jump_timer: f32,
    pub enemy_shoot_enabled: bool,
    pub enemy_shoot_interval: f32,
    pub enemy_shoot_timer: f32,
    pub enemy_shoot_range: f32,
    pub projectile_speed: f32,
    pub projectile_damage: u32,
    pub particles_enabled: bool,
    pub jump_particle_count: u32,
    pub hit_particle_count: u32,
    pub pickup_particle_count: u32,
    pub particles: Vec<Particle>,
    pub enemy_behavior_mode: String,
    pub enemy_chase_range: f32,
    pub enemy_circle_radius: f32,
    pub enemy_circle_speed: f32,
}

impl Scene {
    pub fn new(
        move_speed: f32,
        jump_strength: f32,
        gravity: f32,
        enemy_speed: f32,
        enemy_gravity_scale: f32,
        sprite_scale: f32,
        player_start_health: u32,
        player_max_health: u32,
        enemy_contact_damage: u32,
        hit_invincibility_duration: f32,
        hit_flash_enabled: bool,
        fall_respawn_offset: f32,
        jump_sfx_volume: f32,
        hit_sfx_volume: f32,
        pickup_sfx_volume: f32,
        input: InputConfig,
        world_width: f32,
        world_height: f32,
        moving_platform_enabled: bool,
        moving_platform_vertical: bool,
        moving_platform_speed: f32,
        moving_platform_amplitude: f32,
        collectible_bob_enabled: bool,
        collectible_bob_amplitude: f32,
        collectible_bob_speed: f32,
        enemy_jump_enabled: bool,
        enemy_jump_interval: f32,
        enemy_jump_strength: f32,
        enemy_shoot_enabled: bool,
        enemy_shoot_interval: f32,
        enemy_shoot_range: f32,
        projectile_speed: f32,
        projectile_damage: u32,
        particles_enabled: bool,
        jump_particle_count: u32,
        hit_particle_count: u32,
        pickup_particle_count: u32,
        enemy_behavior_mode: String,
        enemy_chase_range: f32,
        enemy_circle_radius: f32,
        enemy_circle_speed: f32,
    ) -> Self {
        let clamped_max = player_max_health.max(1);
        let clamped_start = player_start_health.max(1).min(clamped_max);

        Self {
            background: None,
            entities: Vec::new(),
            platforms: Vec::new(),
            score: 0,
            move_speed,
            jump_strength,
            gravity,
            enemy_speed,
            enemy_gravity_scale,
            sprite_scale,
            player_health: clamped_start,
            player_max_health: clamped_max,
            enemy_contact_damage,
            hit_invincibility_duration,
            hit_flash_enabled,
            hit_timer: 0.0,
            player_dead: false,
            input,
            world_width,
            world_height,
            time: 0.0,
            fall_respawn_offset,
            jump_sfx_volume,
            hit_sfx_volume,
            pickup_sfx_volume,
            moving_platform_enabled,
            moving_platform_vertical,
            moving_platform_speed,
            moving_platform_amplitude,
            collectible_bob_enabled,
            collectible_bob_amplitude,
            collectible_bob_speed,
            enemy_jump_enabled,
            enemy_jump_interval,
            enemy_jump_strength,
            enemy_jump_timer: 0.0,
            enemy_shoot_enabled,
            enemy_shoot_interval,
            enemy_shoot_timer: 0.0,
            enemy_shoot_range,
            projectile_speed,
            projectile_damage,
            particles_enabled,
            jump_particle_count,
            hit_particle_count,
            pickup_particle_count,
            particles: Vec::new(),
            enemy_behavior_mode,
            enemy_chase_range,
            enemy_circle_radius,
            enemy_circle_speed,
        }
    }

    pub fn update(&mut self, dt: f32, sounds: &Sounds, sfx_enabled: bool) {
        self.time += dt;

        // Update particles
        if self.particles_enabled {
            for p in &mut self.particles {
                p.position += p.velocity * dt;
                p.lifetime -= dt;
            }
            self.particles.retain(|p| p.lifetime > 0.0);
        } else {
            self.particles.clear();
        }

        if self.enemy_jump_timer > 0.0 {
            self.enemy_jump_timer = (self.enemy_jump_timer - dt).max(0.0);
        }

        if self.enemy_shoot_timer > 0.0 {
            self.enemy_shoot_timer = (self.enemy_shoot_timer - dt).max(0.0);
        }

        // Move platforms if enabled (per-platform)
        if self.moving_platform_enabled {
            let t = self.time * self.moving_platform_speed;
            for platform in &mut self.platforms {
                if !platform.moving {
                    continue;
                }
                let offset = (t + platform.phase).sin() * self.moving_platform_amplitude;
                if platform.vertical {
                    platform.position.y = platform.base_position.y + offset;
                } else {
                    platform.position.x = platform.base_position.x + offset;
                }
            }
        }

        if self.hit_timer > 0.0 {
            self.hit_timer = (self.hit_timer - dt).max(0.0);
        }

        let mut any_enemy_jumped = false;
        let mut any_enemy_shot = false;
        let platforms = &self.platforms;

        let player_pos = self
            .entities
            .iter()
            .find(|e| matches!(e.kind, EntityKind::Player))
            .map(|e| e.position);

        let mut new_projectiles: Vec<Entity> = Vec::new();
        let mut new_particles: Vec<Particle> = Vec::new();

        for entity in &mut self.entities {
            match entity.kind {
                EntityKind::Player => {
                    let mut dir = 0.0;
                    let left_down = is_key_down(self.input.move_left_primary)
                        || self
                            .input
                            .move_left_alt
                            .map_or(false, |k| is_key_down(k));
                    let right_down = is_key_down(self.input.move_right_primary)
                        || self
                            .input
                            .move_right_alt
                            .map_or(false, |k| is_key_down(k));
                    if left_down {
                        dir -= 1.0;
                    }
                    if right_down {
                        dir += 1.0;
                    }
                    entity.velocity.x = dir * self.move_speed;

                    let on_ground = is_on_ground(entity, platforms, self.sprite_scale);

                    let mut jump_pressed = false;
                    if is_key_pressed(self.input.jump_primary) {
                        jump_pressed = true;
                    }
                    if self
                        .input
                        .jump_alt
                        .map_or(false, |k| is_key_pressed(k))
                    {
                        jump_pressed = true;
                    }

                    if on_ground && jump_pressed {
                        entity.velocity.y = -self.jump_strength;
                        play_sound_opt(&sounds.jump, self.jump_sfx_volume, sfx_enabled);
                        if self.particles_enabled {
                            let foot_y = entity.position.y
                                + entity.texture.height() * self.sprite_scale / 2.0;
                            emit_particles(
                                &mut new_particles,
                                vec2(entity.position.x, foot_y),
                                self.jump_particle_count,
                                std::f32::consts::PI,
                                std::f32::consts::TAU,
                                80.0,
                                0.35,
                            );
                        }
                    }

                    entity.velocity.y += self.gravity * dt;

                    entity.position += entity.velocity * dt;

                    resolve_platform_collisions(entity, platforms, self.sprite_scale);

                    let half_w = entity.texture.width() * self.sprite_scale / 2.0;
                    if entity.position.x - half_w < 0.0 {
                        entity.position.x = half_w;
                    }
                    if entity.position.x + half_w > self.world_width {
                        entity.position.x = self.world_width - half_w;
                    }

                    let half_h = entity.texture.height() * self.sprite_scale / 2.0;
                    if entity.position.y - half_h > self.world_height + self.fall_respawn_offset {
                        entity.position = vec2(self.world_width / 2.0, 0.0);
                        entity.velocity = Vec2::ZERO;
                    }
                }
                EntityKind::Enemy => {
                    entity.velocity.y += self.gravity * self.enemy_gravity_scale * dt;

                    // Horizontal behavior: patrol / chase / circle
                    match self.enemy_behavior_mode.to_lowercase().as_str() {
                        "chase" => {
                            if let Some(player_pos) = player_pos {
                                let dx = player_pos.x - entity.position.x;
                                if dx.abs() <= self.enemy_chase_range {
                                    let dir = if dx > 0.0 { 1.0 } else { -1.0 };
                                    entity.velocity.x = dir * self.enemy_speed;
                                }
                            }
                        }
                        "circle" => {
                            // Simple circular motion around base_position
                            let angle = self.time * self.enemy_circle_speed + entity.phase;
                            let r = self.enemy_circle_radius;
                            entity.position.x =
                                entity.base_position.x + angle.cos() * r;
                            entity.position.y =
                                entity.base_position.y + angle.sin() * r;
                        }
                        _ => {
                            // "patrol" or unknown: current velocity.x is used
                        }
                    }

                    if !matches!(self.enemy_behavior_mode.to_lowercase().as_str(), "circle") {
                        entity.position += entity.velocity * dt;
                    }

                    resolve_platform_collisions(entity, platforms, self.sprite_scale);

                    let half_w = entity.texture.width() * self.sprite_scale / 2.0;
                    if entity.position.x - half_w < 0.0 || entity.position.x + half_w > self.world_width {
                        if !matches!(self.enemy_behavior_mode.to_lowercase().as_str(), "circle") {
                            entity.velocity.x = -entity.velocity.x;
                        }
                    }

                    if self.enemy_jump_enabled
                        && entity.jumping
                        && self.enemy_jump_interval > 0.0
                        && self.enemy_jump_timer <= 0.0
                    {
                        if is_on_ground(entity, platforms, self.sprite_scale) {
                            entity.velocity.y = -self.enemy_jump_strength;
                            any_enemy_jumped = true;
                        }
                    }

                    if self.enemy_shoot_enabled
                        && self.enemy_shoot_interval > 0.0
                        && self.enemy_shoot_timer <= 0.0
                    {
                        if let Some(player_pos) = player_pos {
                            let to_player = player_pos - entity.position;
                            let dist = to_player.length();
                            if dist > 0.0 && dist <= self.enemy_shoot_range {
                                let dir = to_player / dist;
                                let vel = dir * self.projectile_speed;
                                new_projectiles.push(Entity {
                                    kind: EntityKind::Projectile,
                                    texture: entity.texture.clone(),
                                    position: entity.position,
                                    velocity: vel,
                                    value: 0,
                                    health_value: 0,
                                    base_position: entity.position,
                                    phase: 0.0,
                                    jumping: false,
                                });
                                any_enemy_shot = true;
                            }
                        }
                    }
                }
                EntityKind::Collectible => {
                    if self.collectible_bob_enabled {
                        let angle = self.time * self.collectible_bob_speed + entity.phase;
                        let offset = angle.sin() * self.collectible_bob_amplitude;
                        entity.position.y = entity.base_position.y + offset;
                    }
                }
                EntityKind::Projectile => {
                    entity.position += entity.velocity * dt;
                }
            }
        }

        if any_enemy_jumped {
            self.enemy_jump_timer = self.enemy_jump_interval;
        }

        if any_enemy_shot {
            self.enemy_shoot_timer = self.enemy_shoot_interval;
        }

        // Add newly spawned projectiles
        if !new_projectiles.is_empty() {
            self.entities.extend(new_projectiles);
        }

        // Cull projectiles that leave the world bounds
        let world_w = self.world_width;
        let world_h = self.world_height;
        self.entities.retain(|e| {
            if matches!(e.kind, EntityKind::Projectile) {
                let pos = e.position;
                pos.x >= -32.0
                    && pos.x <= world_w + 32.0
                    && pos.y >= -32.0
                    && pos.y <= world_h + 32.0
            } else {
                true
            }
        });

        // Player bounding box after movement
        let player_rect = self
            .entities
            .iter()
            .find(|e| matches!(e.kind, EntityKind::Player))
            .map(|e| entity_rect(e, self.sprite_scale));

        // Take damage when touching enemies or projectiles
        if let Some(ref rect) = player_rect {
            if self.player_health > 0 {
                let mut hit_enemy = false;
                let mut hit_projectile = false;

                if self.hit_timer <= 0.0 {
                    for e in &self.entities {
                        match e.kind {
                            EntityKind::Enemy => {
                                let enemy_rect = entity_rect(e, self.sprite_scale);
                                if enemy_rect.overlaps(rect) {
                                    hit_enemy = true;
                                }
                            }
                            EntityKind::Projectile => {
                                let proj_rect = entity_rect(e, self.sprite_scale);
                                if proj_rect.overlaps(rect) {
                                    hit_projectile = true;
                                }
                            }
                            _ => {}
                        }
                        if hit_enemy || hit_projectile {
                            break;
                        }
                    }
                }

                if self.hit_timer <= 0.0 && (hit_enemy || hit_projectile) {
                    self.hit_timer = self.hit_invincibility_duration;
                    let mut damage: u32 = 0;
                    if hit_enemy {
                        damage = damage.saturating_add(self.enemy_contact_damage.max(1));
                    }
                    if hit_projectile {
                        damage = damage.saturating_add(self.projectile_damage.max(1));
                    }
                    let damage = damage.max(1);

                    if damage >= self.player_health {
                        self.player_health = 0;
                        self.player_dead = true;
                    } else {
                        self.player_health -= damage;
                    }
                    play_sound_opt(&sounds.hit, self.hit_sfx_volume, sfx_enabled);

                    if self.particles_enabled {
                        let center = vec2(
                            rect.x + rect.w / 2.0,
                            rect.y + rect.h / 2.0,
                        );
                        emit_particles(
                            &mut new_particles,
                            center,
                            self.hit_particle_count,
                            0.0,
                            std::f32::consts::TAU,
                            90.0,
                            0.5,
                        );
                    }
                }

                // Remove projectiles that hit the player, even when invincible
                self.entities.retain(|e| {
                    if matches!(e.kind, EntityKind::Projectile) {
                        let r = entity_rect(e, self.sprite_scale);
                        if r.overlaps(rect) {
                            return false;
                        }
                    }
                    true
                });
            }
        }

        // Collect collectibles when the player touches them
        if let Some(player_rect) = player_rect {
            let mut collected_value: u32 = 0;
            let mut collected_health: u32 = 0;
            let mut pickup_bursts: Vec<Vec2> = Vec::new();
            self.entities.retain(|e| {
                if matches!(e.kind, EntityKind::Collectible) {
                    let r = entity_rect(e, self.sprite_scale);
                    if r.overlaps(&player_rect) {
                        collected_value = collected_value.saturating_add(e.value);
                        collected_health = collected_health.saturating_add(e.health_value);
                        play_sound_opt(&sounds.pickup, self.pickup_sfx_volume, sfx_enabled);
                        if self.particles_enabled {
                            pickup_bursts.push(e.position);
                        }
                        return false;
                    }
                }
                true
            });
            self.score = self.score.saturating_add(collected_value);
            if collected_health > 0 && self.player_health > 0 {
                let new_health = self
                    .player_health
                    .saturating_add(collected_health)
                    .min(self.player_max_health);
                self.player_health = new_health;
            }

            if self.particles_enabled {
                for pos in pickup_bursts {
                    emit_particles(
                        &mut new_particles,
                        pos,
                        self.pickup_particle_count,
                        0.0,
                        std::f32::consts::TAU,
                        60.0,
                        0.4,
                    );
                }
            }
        }

        if self.particles_enabled && !new_particles.is_empty() {
            self.particles.extend(new_particles);
        }
    }

    pub fn draw_background(&self) {
        if let Some(bg) = &self.background {
            let screen_h = screen_height();
            let world_w = self.world_width.max(screen_width());
            let world_h = self.world_height.max(screen_h);
            let dest_size = vec2(world_w, world_h);

            draw_texture_ex(
                bg,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    ..Default::default()
                },
            );
        }
    }

    pub fn draw_world(&self) {
        // platforms
        for platform in &self.platforms {
            let tex = &platform.texture;
            let dest_size = vec2(tex.width() * self.sprite_scale, tex.height() * self.sprite_scale);

            draw_texture_ex(
                tex,
                platform.position.x - dest_size.x / 2.0,
                platform.position.y - dest_size.y / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    ..Default::default()
                },
            );
        }

        // entities (player, enemies, collectibles, projectiles)
        for entity in &self.entities {
            match entity.kind {
                EntityKind::Projectile => {
                    let size = 8.0 * self.sprite_scale;
                    draw_rectangle(
                        entity.position.x - size / 2.0,
                        entity.position.y - size / 2.0,
                        size,
                        size,
                        YELLOW,
                    );
                }
                _ => {
                    let tex = &entity.texture;
                    let dest_size = vec2(
                        tex.width() * self.sprite_scale,
                        tex.height() * self.sprite_scale,
                    );
                    let tint = if matches!(entity.kind, EntityKind::Player)
                        && self.hit_flash_enabled
                        && self.hit_timer > 0.0
                    {
                        RED
                    } else {
                        WHITE
                    };
                    draw_texture_ex(
                        tex,
                        entity.position.x - dest_size.x / 2.0,
                        entity.position.y - dest_size.y / 2.0,
                        tint,
                        DrawTextureParams {
                            dest_size: Some(dest_size),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        // particles
        if self.particles_enabled {
            for p in &self.particles {
                let alpha = (p.lifetime / 0.5).clamp(0.0, 1.0);
                let color = Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: alpha,
                };
                let size = 4.0 * self.sprite_scale;
                draw_rectangle(
                    p.position.x - size / 2.0,
                    p.position.y - size / 2.0,
                    size,
                    size,
                    color,
                );
            }
        }
    }

    pub fn draw(&self) {
        self.draw_background();
        self.draw_world();
    }

}

impl Scene {
    pub fn player_position(&self) -> Option<Vec2> {
        self.entities
            .iter()
            .find(|e| matches!(e.kind, EntityKind::Player))
            .map(|e| e.position)
    }

    pub fn debug_draw(&self) {
        let scale = self.sprite_scale;

        // Platforms
        for platform in &self.platforms {
            let r = platform_rect(platform, scale);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, GREEN);
        }

        // Entities
        for e in &self.entities {
            let r = entity_rect(e, scale);
            let color = match e.kind {
                EntityKind::Player => BLUE,
                EntityKind::Enemy => RED,
                EntityKind::Collectible => YELLOW,
                EntityKind::Projectile => ORANGE,
            };
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, color);
        }
    }
}

fn entity_rect(entity: &Entity, scale: f32) -> Rect {
    match entity.kind {
        EntityKind::Projectile => {
            let size = 8.0 * scale;
            Rect::new(
                entity.position.x - size / 2.0,
                entity.position.y - size / 2.0,
                size,
                size,
            )
        }
        _ => {
            let w = entity.texture.width() * scale;
            let h = entity.texture.height() * scale;
            Rect::new(
                entity.position.x - w / 2.0,
                entity.position.y - h / 2.0,
                w,
                h,
            )
        }
    }
}

fn emit_particles(
    out: &mut Vec<Particle>,
    origin: Vec2,
    count: u32,
    angle_start: f32,
    angle_end: f32,
    speed: f32,
    lifetime: f32,
) {
    if count == 0 {
        return;
    }
    let total = count.max(1);
    let delta = if total <= 1 {
        0.0
    } else {
        (angle_end - angle_start) / (total as f32)
    };
    for i in 0..total {
        let angle = angle_start + delta * i as f32;
        let dir = vec2(angle.cos(), angle.sin());
        out.push(Particle {
            position: origin,
            velocity: dir * speed,
            lifetime,
        });
    }
}

fn platform_rect(platform: &Platform, scale: f32) -> Rect {
    let w = platform.texture.width() * scale;
    let h = platform.texture.height() * scale;
    Rect::new(platform.position.x - w / 2.0, platform.position.y - h / 2.0, w, h)
}

fn is_on_ground(entity: &Entity, platforms: &[Platform], scale: f32) -> bool {
    let mut rect = entity_rect(entity, scale);
    rect.y += 1.0;
    for platform in platforms {
        let plat = platform_rect(platform, scale);
        if rect.overlaps(&plat) {
            return true;
        }
    }
    false
}

fn resolve_platform_collisions(entity: &mut Entity, platforms: &[Platform], scale: f32) {
    if entity.velocity.y <= 0.0 {
        return;
    }

    let mut rect = entity_rect(entity, scale);

    for platform in platforms {
        let plat = platform_rect(platform, scale);
        if !rect.overlaps(&plat) {
            continue;
        }

        // landed on top of platform
        entity.position.y = plat.y - rect.h / 2.0;
        entity.velocity.y = 0.0;
        rect.y = plat.y - rect.h;
    }
}

fn play_sound_opt(sound: &Option<Sound>, volume: f32, enabled: bool) {
    if !enabled {
        return;
    }
    if let Some(s) = sound.as_ref() {
        audio::play_sound(
            s,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }
}
