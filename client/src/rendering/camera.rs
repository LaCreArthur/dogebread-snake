use bevy::prelude::*;
use shared::game::*;

use super::ScreenShake;

/// Camera follows the player's snake head during gameplay, centers during menus.
/// Adds screen shake offset when ScreenShake intensity > 0.
/// Zooms out as arena shrinks.
pub fn camera_follow(
    player_query: Query<&Snake, With<PlayerControlled>>,
    spectate_query: Query<(&Snake, &SnakeId), Without<PlayerControlled>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    state: Res<State<GameState>>,
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    bounds: Res<ArenaBounds>,
) {
    let Ok((mut cam_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let target = match state.get() {
        GameState::WaitingToStart | GameState::Countdown | GameState::GameOver => Vec2::ZERO,
        GameState::Playing => {
            if let Ok(snake) = player_query.single() {
                if snake.alive {
                    snake.head().to_world()
                } else {
                    spectate_query
                        .iter()
                        .filter(|(s, _)| s.alive)
                        .max_by_key(|(s, _)| s.score)
                        .map(|(s, _)| s.head().to_world())
                        .unwrap_or(Vec2::ZERO)
                }
            } else {
                Vec2::ZERO
            }
        }
    };

    let current = cam_transform.translation.truncate();
    let smoothing = 1.0 - (-5.0 * time.delta_secs()).exp();
    let mut smoothed = current.lerp(target, smoothing);

    if shake.intensity > 0.1 {
        let t = time.elapsed_secs();
        let offset_x = (t * 137.0).sin() * shake.intensity;
        let offset_y = (t * 251.0).cos() * shake.intensity;
        smoothed.x += offset_x;
        smoothed.y += offset_y;
        shake.intensity *= shake.decay;
    } else {
        shake.intensity = 0.0;
    }

    cam_transform.translation.x = smoothed.x;
    cam_transform.translation.y = smoothed.y;

    if let Projection::Orthographic(ortho) = projection.as_mut() {
        let bounds_width = (bounds.max_x - bounds.min_x) as f32;
        let arena_fraction = bounds_width / (shared::constants::GRID_WIDTH as f32 - 2.0);
        let mut target_scale = 1.0 + (1.0 - arena_fraction) * 0.5;

        if *state.get() == GameState::GameOver {
            target_scale = 2.0;
        }

        let scale_smoothing = 1.0 - (-3.0 * time.delta_secs()).exp();
        ortho.scale = ortho.scale + (target_scale - ortho.scale) * scale_smoothing;
    }
}
