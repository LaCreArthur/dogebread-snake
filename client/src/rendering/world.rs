use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

use super::sprites::SpriteAssets;
use super::*;

/// Sync all snake entities to their sprite representations
pub fn render_snakes(
    mut commands: Commands,
    sprite_assets: Res<SpriteAssets>,
    snake_query: Query<(Entity, &Snake, &SnakeColor, Option<&DeathTimer>)>,
    mut segment_query: Query<(Entity, &mut Transform, &mut Sprite, &SnakeSegmentSprite)>,
) {
    let alive_snake_entities: Vec<Entity> = snake_query.iter().map(|(e, _, _, _)| e).collect();

    for (snake_entity, snake, color, _) in &snake_query {
        let existing: Vec<(Entity, usize)> = segment_query
            .iter()
            .filter(|(_, _, _, seg)| seg.snake_entity == snake_entity)
            .map(|(e, _, _, seg)| (e, seg.index))
            .collect();

        let needed = snake.segments.len();
        let existing_count = existing.len();

        if needed > existing_count {
            for i in existing_count..needed {
                let pos = snake.segments[i];
                if i == 0 {
                    // Head: use Doge sprite
                    commands.spawn((
                        Sprite {
                            image: sprite_assets.doge_head.clone(),
                            custom_size: Some(Vec2::splat(CELL_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(pos.to_world().extend(2.0)),
                        SnakeSegmentSprite { snake_entity, index: i },
                    ));
                } else {
                    // Body: colored square (keeps per-snake color identity)
                    commands.spawn((
                        Sprite::from_color(color.body, Vec2::splat(CELL_SIZE - 2.0)),
                        Transform::from_translation(pos.to_world().extend(1.0)),
                        SnakeSegmentSprite { snake_entity, index: i },
                    ));
                }
            }
        }

        if needed < existing_count {
            let mut sorted = existing.clone();
            sorted.sort_by_key(|(_, i)| *i);
            for (entity, _) in sorted.iter().rev().take(existing_count - needed) {
                if let Ok(mut ec) = commands.get_entity(*entity) {
                    ec.despawn();
                }
            }
        }
    }

    for (_, mut transform, mut sprite, seg) in &mut segment_query {
        if !alive_snake_entities.contains(&seg.snake_entity) {
            continue;
        }

        let Ok((_, snake, color, death_timer)) = snake_query.get(seg.snake_entity) else {
            continue;
        };

        if seg.index < snake.segments.len() {
            let pos = snake.segments[seg.index];
            transform.translation = pos.to_world().extend(if seg.index == 0 { 2.0 } else { 1.0 });

            if !snake.alive {
                // Death blink: use alpha fade on all segments
                let blink = if let Some(dt) = death_timer {
                    let t = dt.timer.fraction();
                    (1.0 - t) * ((t * 12.0).sin() * 0.5 + 0.5)
                } else {
                    0.5
                };
                sprite.color = Color::srgba(0.4, 0.4, 0.4, blink);
                sprite.custom_size = Some(Vec2::splat(if seg.index == 0 {
                    CELL_SIZE
                } else {
                    CELL_SIZE - 3.0
                }));
            } else if seg.index == 0 {
                // Head: restore full-color (no tint) so Doge sprite looks natural
                sprite.color = Color::WHITE;
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE));
            } else {
                // Body segments: tint with snake color
                let c = color.body.to_srgba();
                sprite.color = Color::srgb(c.red * 0.8, c.green * 0.8, c.blue * 0.8);
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE - 3.0));
            }
        }
    }
}

/// Render food with pulsing animation using coin sprite
pub fn render_food(
    mut commands: Commands,
    time: Res<Time>,
    sprite_assets: Res<SpriteAssets>,
    food_query: Query<&Food>,
    mut food_sprite_query: Query<(Entity, &mut Transform, &mut Sprite), With<FoodSprite>>,
) {
    let foods: Vec<&Food> = food_query.iter().collect();

    if foods.is_empty() {
        for (entity, _, _) in &food_sprite_query {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
        return;
    }

    let elapsed = time.elapsed_secs();
    let base_size = CELL_SIZE - 4.0;

    let mut existing: Vec<_> = food_sprite_query.iter_mut().collect();
    for (i, food) in foods.iter().enumerate() {
        let phase_hash = (food.pos.x * 7 + food.pos.y * 13) as f32;
        let pulse = (elapsed * 3.0 + phase_hash).sin();
        let scale = 1.0 + pulse * 0.15;
        let pulsed_size = base_size * scale;

        if i < existing.len() {
            existing[i].1.translation = food.pos.to_world().extend(0.5);
            existing[i].2.custom_size = Some(Vec2::splat(pulsed_size));
            // Keep coin sprite color natural (gold coin needs no extra tint)
            existing[i].2.color = Color::WHITE;
        } else {
            commands.spawn((
                Sprite {
                    image: sprite_assets.coin.clone(),
                    custom_size: Some(Vec2::splat(pulsed_size)),
                    ..default()
                },
                Transform::from_translation(food.pos.to_world().extend(0.5)),
                FoodSprite,
            ));
        }
    }

    for (entity, _, _) in existing.iter().skip(foods.len()) {
        if let Ok(mut ec) = commands.get_entity(*entity) {
            ec.despawn();
        }
    }
}
