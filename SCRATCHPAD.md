# Scratchpad

## Current State
ALL P0 + 6/8 P1 COMPLETE. 10-snake battle royale with smooth camera, arena zoom, trail particles, dramatic phased game over screen. Doge theming throughout.

## Last Sprint
Sprint 6 (opus): Dramatic game over — 4-phase animated reveal with overlay fade-in, winner announcement, colored ranked scoreboard with crown/YOU markers, stats footer, delayed restart prompt.
Sprint 7 (sonnet, parallel): Trail afterimage effect — fading particles at segment[1] every 0.15s, 0.4s lifespan.

## Next Steps (remaining P1)
1. **WASM build + deploy** — verify wasm32-unknown-unknown target compiles, trunk or wasm-pack, deploy to GitHub Pages
2. **Sound effects** — procedural audio or embedded WAV, eat/die/kill/shrink/countdown
3. **Player name entry** — text input before match

## Blockers
- Sound effects may be hard without asset files (constraint: no external assets yet). Could do procedural beeps via bevy_audio or skip.
- WASM: need to verify WebTransport certs not needed for PvE. wasm-bindgen already in lib.rs.

## Notes
- effects.rs: FloatingText, DeathParticle, SpeedUpText, TrailParticle (SOLID)
- rendering.rs: GameOverAnimation resource with 4-phase reveal system
- lib.rs: 10 snakes, 35 food, 10 spawn positions, TrailSpawner resource
- GameState: WaitingToStart → Countdown → Playing → GameOver
- All despawns use safe commands.get_entity() pattern
- Bevy 0.18: Projection::Orthographic(ortho) pattern for camera zoom
