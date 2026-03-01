//! Sprite asset loading — embeds PNG files at compile time via include_bytes!().
//! Available immediately in WASM (no filesystem needed).

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;

/// Pre-loaded sprite handles. Inserted as a resource during Startup.
#[derive(Resource)]
pub struct SpriteAssets {
    pub doge_head: Handle<Image>,
    pub coin: Handle<Image>,
}

/// Startup system: decode embedded PNGs and register them in Assets<Image>.
pub fn load_sprite_assets(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    let doge_bytes: &[u8] = include_bytes!("../../assets/doge_head.png");
    let coin_bytes: &[u8] = include_bytes!("../../assets/coin.png");

    let load = |bytes: &[u8]| {
        Image::from_buffer(
            bytes,
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true, // is_srgb
            ImageSampler::nearest(),
            RenderAssetUsages::default(),
        )
        .expect("failed to decode embedded sprite PNG")
    };

    commands.insert_resource(SpriteAssets {
        doge_head: images.add(load(doge_bytes)),
        coin: images.add(load(coin_bytes)),
    });
}
