use macroquad::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteKind {
    Player,
    Enemy,
    Background,
    Platform,
    Collectible,
    GoalCollectible,
}

#[derive(Clone)]
pub struct SpriteAsset {
    pub name: String,
    pub texture: Texture2D,
    pub kind: SpriteKind,
}

pub struct Assets {
    pub sprites: Vec<SpriteAsset>,
}

impl Assets {
    pub fn sprites_of_kind(&self, kind: SpriteKind) -> Vec<&SpriteAsset> {
        self.sprites
            .iter()
            .filter(|s| s.kind == kind)
            .collect()
    }

    pub fn sprite_by_kind_and_name(
        &self,
        kind: SpriteKind,
        name: &str,
    ) -> Option<&SpriteAsset> {
        self.sprites
            .iter()
            .find(|s| s.kind == kind && s.name.eq_ignore_ascii_case(name))
    }
}

pub async fn load_assets(root: &str) -> Result<Assets, String> {
    let mut sprites = Vec::new();

    load_sprites_for_kind(root, "sprites/player", SpriteKind::Player, &mut sprites).await?;
    load_sprites_for_kind(root, "sprites/enemies", SpriteKind::Enemy, &mut sprites).await?;
    load_sprites_for_kind(
        root,
        "sprites/collectibles",
        SpriteKind::Collectible,
        &mut sprites,
    )
    .await?;
    load_sprites_for_kind(
        root,
        "sprites/goals",
        SpriteKind::GoalCollectible,
        &mut sprites,
    )
    .await?;
    load_sprites_for_kind(root, "tiles/platforms", SpriteKind::Platform, &mut sprites).await?;
    load_sprites_for_kind(root, "backgrounds", SpriteKind::Background, &mut sprites).await?;

    Ok(Assets { sprites })
}

async fn load_sprites_for_kind(
    root: &str,
    subdir: &str,
    kind: SpriteKind,
    sprites: &mut Vec<SpriteAsset>,
) -> Result<(), String> {
    let dir = format!("{root}/{subdir}");

    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Warning: could not read directory {dir}: {e}");
            return Ok(());
        }
    };

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry in {dir}: {e}"))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }

        let path_str = path
            .to_str()
            .ok_or_else(|| "Non-UTF8 path in assets directory".to_string())?
            .to_string();

        let texture = load_texture(&path_str)
            .await
            .map_err(|e| format!("Failed to load texture {path_str}: {e:?}"))?;
        texture.set_filter(FilterMode::Nearest);

        let name = Path::new(&path_str)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        sprites.push(SpriteAsset { name, texture, kind });
    }

    Ok(())
}
