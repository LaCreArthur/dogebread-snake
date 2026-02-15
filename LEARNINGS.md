# Learnings — DogeBread Snake (Bevy 0.18 / Rust)

[2026-02-15] #gamedev #theming: Visual theming is purely cosmetic — only modify rendering.rs constants and text, leave game logic untouched
[2026-02-15] #gamedev #theming: Doge palette: #e8b04b gold, #1a1a2e background, bright saturated accent colors for AI snakes
[2026-02-15] #bevy #api: Bevy 0.18 breaks heavily from 0.15 — always read migration guide first
[2026-02-15] #bevy #api: EventWriter removed from prelude in 0.18; use std::process::exit as fallback
[2026-02-15] #bevy #api: System tuples max 8 elements for .run_if() — split into multiple add_systems calls
[2026-02-15] #bevy #api: OrthographicProjection not queryable as Component in 0.18
[2026-02-15] #bevy #api: Timer::finished() is private — use Timer::just_finished() instead
[2026-02-15] #bevy #api: WindowResolution accepts (u32,u32) not (f32,f32) in 0.18
[2026-02-15] #bevy #ecs: Query conflict B0001 — use Without<T> filter to make disjoint queries
[2026-02-15] #bevy #screenshot: Screenshot::primary_window() + save_to_disk() observer pattern for captures
[2026-02-15] #bevy #screenshot: AUTO_SCREENSHOT=1 env var pattern for autonomous visual verification
[2026-02-15] #bevy #rendering: Sprite::from_color(), Color::srgb(), Camera2d (no bundle) in 0.18
[2026-02-15] #bevy #ui: Text + TextFont + TextColor + Node for UI text; no TextBundle in 0.18
[2026-02-15] #gamedev #ai: 2-step look-ahead prevents dead-end trapping; personality variation (Hungry/Cautious/Aggressive) makes AI feel alive
[2026-02-15] #gamedev #design: Arena shrinking is the single highest-impact battle royale mechanic — implement first
[2026-02-15] #gamedev #ux: Minimap is essential when camera follows player — provides spatial awareness
[2026-02-15] #rust #workflow: Compile-screenshot-iterate loop works well; Bevy hot-reload not needed for this scale
[2026-02-15] #rust #speed: PCG-style RNG (wrapping_mul + wrapping_add) avoids rand crate dependency
