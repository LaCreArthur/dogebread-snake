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

        if ft.timer.just_finished() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
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

        if particle.timer.just_finished() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

/// Spawn a "+1" floating text at a world position
pub fn spawn_score_popup(commands: &mut Commands, world_pos: Vec2) {
    commands.spawn((
        Text2d::new("+1"),
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

/// Spawn a centered "SPEED UP!" text that fades over 1.5s
pub fn spawn_speed_up_text(commands: &mut Commands) {
    commands.spawn((
        Text::new("SPEED UP!"),
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

        if speed_text.timer.just_finished() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}
