use bevy::prelude::*;
use shared::game::*;

use crate::audio;
use crate::effects;
use crate::rendering;
use crate::{GameTick, NUM_FOOD, SimpleRng};

#[derive(Resource)]
pub(crate) struct ArenaShrinkTimer {
    pub timer: Timer,
}

impl ArenaShrinkTimer {
    pub fn new(interval: f32) -> Self {
        Self {
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
pub(crate) struct SpeedTimer {
    pub timer: Timer,
}

impl SpeedTimer {
    pub fn new(interval: f32) -> Self {
        Self {
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
pub(crate) struct MatchTimer {
    pub elapsed: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn game_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut tick: ResMut<GameTick>,
    mut rng: ResMut<SimpleRng>,
    mut snake_query: Query<(Entity, &mut Snake, &SnakeId, &SnakeColor, Option<&PlayerControlled>)>,
    food_query: Query<(Entity, &Food)>,
    mut match_state: ResMut<MatchState>,
    mut next_state: ResMut<NextState<GameState>>,
    bounds: Res<ArenaBounds>,
    mut shake: ResMut<rendering::ScreenShake>,
    sfx: Option<Res<audio::SoundEffects>>,
) {
    tick.timer.tick(time.delta());
    if !tick.timer.just_finished() {
        return;
    }
    tick.tick_count += 1;

    let mut new_heads: Vec<(Entity, SnakeId, GridPos)> = Vec::new();
    for (entity, mut snake, id, _, _) in &mut snake_query {
        if !snake.alive {
            continue;
        }
        let head = snake.step();
        new_heads.push((entity, *id, head));
    }

    let body_map: Vec<(SnakeId, Vec<GridPos>)> = snake_query
        .iter()
        .filter(|(_, s, _, _, _)| s.alive)
        .map(|(_, s, id, _, _)| (*id, s.segments.iter().skip(1).copied().collect()))
        .collect();

    let mut kills: Vec<(Entity, Option<SnakeId>)> = Vec::new();

    for (entity, my_id, head) in &new_heads {
        if !bounds.contains(*head) {
            kills.push((*entity, None));
            continue;
        }

        if let Ok((_, snake, _, _, _)) = snake_query.get(*entity)
            && snake.self_collision()
        {
            kills.push((*entity, None));
            continue;
        }

        for (other_id, body_segments) in &body_map {
            if *other_id == *my_id {
                continue;
            }
            if body_segments.contains(head) {
                kills.push((*entity, Some(*other_id)));
                break;
            }
        }

        for (other_entity, other_id, other_head) in &new_heads {
            if *other_id == *my_id {
                continue;
            }
            if *head == *other_head {
                kills.push((*entity, None));
                kills.push((*other_entity, None));
            }
        }
    }

    let mut seen = Vec::new();
    let mut unique_kills = Vec::new();
    for (entity, killer) in &kills {
        if !seen.contains(entity) {
            seen.push(*entity);
            unique_kills.push((*entity, *killer));
        }
    }

    let kill_credits: Vec<SnakeId> = unique_kills.iter().filter_map(|(_, killer)| *killer).collect();

    for killer_id in &kill_credits {
        for (_, mut snake, id, _, _) in &mut snake_query {
            if *id == *killer_id {
                snake.kills += 1;
            }
        }
    }

    let player_kills = snake_query
        .iter()
        .find(|(_, _, id, _, _)| id.0 == 0)
        .map(|(_, snake, _, _, _)| snake.kills)
        .unwrap_or(0);

    for (entity, killer) in &unique_kills {
        if let Ok((_, mut snake, dead_id, color, _)) = snake_query.get_mut(*entity)
            && snake.alive
        {
            let death_pos = snake.head().to_world();
            snake.alive = false;
            commands.entity(*entity).insert(DeathTimer {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
            });

            if let Some(killer_id) = killer {
                if killer_id.0 == 0 {
                    shake.intensity = 8.0 + (player_kills as f32 * 2.0).min(12.0);
                } else {
                    shake.intensity = 8.0;
                }
            } else {
                shake.intensity = 8.0;
            }

            rendering::spawn_death_particles(&mut commands, death_pos, color.head, time.elapsed_secs());

            let dead_name = if dead_id.0 == 0 {
                "You".to_string()
            } else {
                rendering::get_snake_color_name(dead_id.0).to_string()
            };
            let (message, feed_color) = if let Some(killer_id) = killer {
                let killer_name = if killer_id.0 == 0 {
                    "You".to_string()
                } else {
                    rendering::get_snake_color_name(killer_id.0).to_string()
                };
                (format!("{} bonked by {}! wow", dead_name, killer_name), color.head)
            } else {
                (format!("{} ded! much rip!", dead_name), color.head)
            };
            rendering::spawn_kill_feed_entry(&mut commands, message, feed_color);

            if let Some(ref sfx) = sfx {
                audio::play_sfx(&mut commands, &sfx.death);
            }
        }
    }

    let alive = snake_query.iter().filter(|(_, s, _, _, _)| s.alive).count() as u32;
    match_state.alive_count = alive;

    if alive <= 1 && match_state.total_snakes > 1 {
        if let Some(ref sfx) = sfx {
            audio::play_sfx(&mut commands, &sfx.game_over);
        }
        next_state.set(GameState::GameOver);
    }

    let eaten_positions: Vec<GridPos> = snake_query
        .iter()
        .filter(|(_, s, _, _, _)| s.alive)
        .map(|(_, s, _, _, _)| s.head())
        .collect();

    let mut foods_eaten = 0;
    for (food_entity, food) in &food_query {
        if eaten_positions.contains(&food.pos) {
            if let Ok(mut ec) = commands.get_entity(food_entity) {
                ec.despawn();
            }
            foods_eaten += 1;

            for (_, mut snake, _, _, player) in &mut snake_query {
                if snake.alive && snake.head() == food.pos {
                    snake.grow_pending += 2;
                    snake.score += 1;

                    if player.is_some() {
                        let eat_pos = food.pos.to_world();
                        rendering::spawn_score_popup(&mut commands, eat_pos);
                        rendering::spawn_eat_particles(&mut commands, eat_pos, time.elapsed_secs());
                        shake.intensity = 2.0;
                        if let Some(ref sfx) = sfx {
                            audio::play_sfx(&mut commands, &sfx.eat);
                        }
                    }
                }
            }
        }
    }

    let remaining_food = food_query.iter().count() - foods_eaten;
    let target_food = NUM_FOOD;
    if remaining_food < target_food {
        for _ in 0..(target_food - remaining_food) {
            let x = rng.range(bounds.min_x + 1, bounds.max_x - 1);
            let y = rng.range(bounds.min_y + 1, bounds.max_y - 1);
            commands.spawn(Food::new(x, y));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn arena_shrink(
    mut commands: Commands,
    time: Res<Time>,
    mut shrink_timer: ResMut<ArenaShrinkTimer>,
    mut bounds: ResMut<ArenaBounds>,
    mut snake_query: Query<(Entity, &mut Snake, &SnakeColor, &SnakeId)>,
    food_query: Query<(Entity, &Food)>,
    mut shake: ResMut<rendering::ScreenShake>,
    mut warning: ResMut<rendering::ShrinkWarning>,
    sfx: Option<Res<audio::SoundEffects>>,
) {
    shrink_timer.timer.tick(time.delta());

    let elapsed = shrink_timer.timer.elapsed_secs();
    let duration = shrink_timer.timer.duration().as_secs_f32();
    let was_warning = warning.active;
    warning.active = bounds.can_shrink() && elapsed > (duration - 2.0);

    if warning.active
        && !was_warning
        && let Some(ref sfx) = sfx
    {
        audio::play_sfx(&mut commands, &sfx.shrink_warning);
    }

    if !shrink_timer.timer.just_finished() {
        return;
    }

    if !bounds.can_shrink() {
        return;
    }

    bounds.shrink();

    if let Some(ref sfx) = sfx {
        audio::play_sfx(&mut commands, &sfx.shrink_impact);
    }
    shake.intensity = 4.0;

    for (entity, mut snake, color, snake_id) in &mut snake_query {
        if !snake.alive {
            continue;
        }
        if !bounds.contains(snake.head()) {
            let death_pos = snake.head().to_world();
            snake.alive = false;
            commands.entity(entity).insert(DeathTimer {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
            });
            rendering::spawn_death_particles(&mut commands, death_pos, color.head, time.elapsed_secs());
            shake.intensity = 8.0;

            let dead_name = if snake_id.0 == 0 {
                "You".to_string()
            } else {
                rendering::get_snake_color_name(snake_id.0).to_string()
            };
            rendering::spawn_kill_feed_entry(
                &mut commands,
                format!("{} crushed! such squish!", dead_name),
                color.head,
            );
        }
    }

    for (entity, food) in &food_query {
        if !bounds.contains(food.pos)
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }
}

pub fn speed_increase(
    mut commands: Commands,
    time: Res<Time>,
    mut speed_timer: ResMut<SpeedTimer>,
    mut tick: ResMut<GameTick>,
    sfx: Option<Res<audio::SoundEffects>>,
) {
    speed_timer.timer.tick(time.delta());
    if !speed_timer.timer.just_finished() {
        return;
    }

    let current = tick.timer.duration().as_secs_f32();
    let new_interval = (current * 0.85).max(0.06);
    tick.timer
        .set_duration(std::time::Duration::from_secs_f32(new_interval));

    if let Some(ref sfx) = sfx {
        audio::play_sfx(&mut commands, &sfx.speed_up);
    }
    effects::spawn_speed_up_text(&mut commands);
}

pub fn track_match_time(time: Res<Time>, mut match_timer: ResMut<MatchTimer>) {
    match_timer.elapsed += time.delta_secs();
}

pub fn update_timer_text(match_timer: Res<MatchTimer>, mut text_query: Query<&mut Text, With<rendering::TimerText>>) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    let secs = match_timer.elapsed as u32;
    let mins = secs / 60;
    let secs = secs % 60;
    **text = format!("{}:{:02}", mins, secs);
}

pub fn handle_esc_quit(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}
