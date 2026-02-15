use bevy::prelude::*;
use shared::constants::*;

/// Floating text that drifts up and fades (e.g. "+1" score popup)
#[derive(Component)]
pub struct FloatingText {
    pub timer: Timer,
}

/// Death particle: flies outward, shrinks, fades
#[derive(Component)]
pub struct DeathParticle {
    pub velocity: Vec2,
    pub timer: Timer,
}

/// Speed-up indicator text that fades over 1.5s
#[derive(Component)]
pub struct SpeedUpText {
    pub timer: Timer,
}

/// Eat particle: golden sparkle burst when food is eaten
#[derive(Component)]
pub struct EatParticle {
    pub velocity: Vec2,
    pub timer: Timer,
}

/// Trail particle: fading afterimage left behind by moving snakes
#[derive(Component)]
pub struct TrailParticle {
    pub timer: Timer,
}

/// Resource to control trail particle spawning rate
#[derive(bevy::prelude::Resource)]
pub struct TrailSpawner {
    pub timer: Timer,
}

/// Animate floating text: drift up, fade out, despawn when done
pub fn animate_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FloatingText, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut ft, mut transform, mut color) in &mut query {
        ft.timer.tick(time.delta());
        let frac = ft.timer.fraction();

        // Float upward
        transform.translation.y += 30.0 * time.delta_secs();

        // Fade out
        let alpha = 1.0 - frac;
        color.0 = Color::srgba(0.95, 0.85, 0.3, alpha);

        if ft.timer.just_finished()
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }
}

/// Animate death particles: move, shrink, fade, despawn
pub fn animate_death_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DeathParticle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.timer.tick(time.delta());
        let frac = particle.timer.fraction();

        // Move
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Shrink and fade
        let remaining = 1.0 - frac;
        transform.scale = Vec3::splat(remaining);
        if let Some(ref mut size) = sprite.custom_size {
            // Keep base size, scale handles shrinking
            let _ = size;
        }
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, remaining);

        if particle.timer.just_finished()
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }
}

/// Counter for cycling through score popup texts
static POPUP_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

const SCORE_POPUPS: &[&str] = &["wow", "+1", "such coin", "very nom", "many point"];

/// Spawn a doge-themed floating text at a world position
pub fn spawn_score_popup(commands: &mut Commands, world_pos: Vec2) {
    let idx = POPUP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as usize;
    let text = SCORE_POPUPS[idx % SCORE_POPUPS.len()];
    commands.spawn((
        Text2d::new(text),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgba(0.95, 0.85, 0.3, 1.0)),
        Transform::from_translation(world_pos.extend(10.0)),
        FloatingText {
            timer: Timer::from_seconds(0.8, TimerMode::Once),
        },
    ));
}

/// Spawn death explosion particles at a world position with the given color
pub fn spawn_death_particles(commands: &mut Commands, world_pos: Vec2, color: Color, time_secs: f32) {
    let num_particles = 10;
    for i in 0..num_particles {
        // Distribute evenly around circle, use time_secs for slight variation
        let angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU
            + (time_secs * 31.0).sin() * 0.3;
        let speed = 40.0 + (time_secs * (i as f32 + 1.0) * 17.0).sin().abs() * 40.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        commands.spawn((
            Sprite::from_color(color, Vec2::splat(CELL_SIZE * 0.5)),
            Transform::from_translation(world_pos.extend(5.0)),
            DeathParticle {
                velocity,
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        ));
    }
}

/// Spawn golden eat sparkle particles at a food position
pub fn spawn_eat_particles(commands: &mut Commands, world_pos: Vec2, time_secs: f32) {
    let num = 6;
    for i in 0..num {
        let angle = (i as f32 / num as f32) * std::f32::consts::TAU
            + (time_secs * 47.0).sin() * 0.5;
        let speed = 50.0 + (time_secs * (i as f32 + 1.0) * 23.0).sin().abs() * 30.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        commands.spawn((
            Sprite::from_color(
                Color::srgba(0.95, 0.80, 0.25, 1.0), // golden
                Vec2::splat(CELL_SIZE * 0.3),
            ),
            Transform::from_translation(world_pos.extend(6.0)),
            EatParticle {
                velocity,
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
        ));
    }
}

/// Animate eat particles: fly outward, shrink, fade, despawn
pub fn animate_eat_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EatParticle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.timer.tick(time.delta());
        let frac = particle.timer.fraction();

        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        let remaining = 1.0 - frac;
        transform.scale = Vec3::splat(remaining);
        sprite.color = Color::srgba(0.95, 0.80, 0.25, remaining);

        if particle.timer.just_finished()
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }
}

/// Spawn a centered doge-themed speed-up text that fades over 1.5s
pub fn spawn_speed_up_text(commands: &mut Commands) {
    commands.spawn((
        Text::new("much fast! wow!"),
        TextFont { font_size: 48.0, ..default() },
        TextColor(Color::srgba(1.0, 0.85, 0.2, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            left: Val::Percent(50.0),
            ..default()
        },
        SpeedUpText {
            timer: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));
}

/// Animate speed-up text: fade out and despawn
pub fn animate_speed_up_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SpeedUpText, &mut TextColor)>,
) {
    for (entity, mut speed_text, mut color) in &mut query {
        speed_text.timer.tick(time.delta());
        let frac = speed_text.timer.fraction();

        // Scale alpha from 1.0 to 0.0
        let alpha = 1.0 - frac;
        color.0 = Color::srgba(1.0, 0.85, 0.2, alpha);

        if speed_text.timer.just_finished()
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }
}

/// Spawn trail particles at snake positions each tick
pub fn spawn_trail_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<TrailSpawner>,
    snake_query: Query<(&shared::game::Snake, &shared::game::SnakeColor)>,
) {
    spawner.timer.tick(time.delta());
    if !spawner.timer.just_finished() {
        return;
    }

    for (snake, color) in &snake_query {
        if !snake.alive || snake.segments.len() < 2 {
            continue;
        }
        let trail_pos = snake.segments[1]; // where head just was
        let c = color.body.to_srgba();
        commands.spawn((
            Sprite::from_color(
                Color::srgba(c.red, c.green, c.blue, 0.4),
                Vec2::splat(CELL_SIZE * 0.6),
            ),
            Transform::from_translation(trail_pos.to_world().extend(0.5)),
            TrailParticle {
                timer: Timer::from_seconds(0.4, TimerMode::Once),
            },
        ));
    }
}

/// Animate trail particles: fade out and despawn
pub fn animate_trail_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TrailParticle, &mut Sprite)>,
) {
    for (entity, mut particle, mut sprite) in &mut query {
        particle.timer.tick(time.delta());
        let frac = particle.timer.fraction();

        // Fade from 0.4 to 0.0
        let alpha = 0.4 * (1.0 - frac);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);

        if particle.timer.just_finished()
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }
}
