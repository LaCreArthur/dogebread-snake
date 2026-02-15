# Scratchpad

## Current State
ALL P0 COMPLETE. 7/8 P1 COMPLETE. Polished 10-snake battle royale with smooth camera, arena zoom, trail particles, dramatic 4-phase game over, WASM build pipeline. 24 commits total.

## Remaining P1
1. **Sound effects** — blocked by "no external assets" constraint. Would need procedural audio or embedded WAV bytes.
2. **Player name entry** — text input in Bevy is non-trivial. Low priority for PvE demo.

## P2 Candidates (if continuing)
- Spectate: auto-follow strongest remaining snake (moderate)
- Screen shake scales with kills (easy)
- GitHub Pages deployment via GitHub Actions (infrastructure)
- WASM size optimization: wasm-opt, LTO, strip debug symbols (73M → <10M target)

## What's Shippable Now
The game is feature-complete for a PvE demo. To share:
1. `./build-wasm.sh`
2. `cd web && python3 -m http.server 8080`
3. Open http://localhost:8080

For public hosting: push web/ artifacts to a gh-pages branch or set up GitHub Actions.

## Architecture Notes
- effects.rs: FloatingText, DeathParticle, SpeedUpText, TrailParticle
- rendering.rs: GameOverAnimation (4-phase), all grid/snake/food/HUD/minimap rendering
- input.rs: Player input + AI (Hungry/Cautious/Aggressive)
- lib.rs: System registration, game logic (tick, collisions, scoring), 10 snakes, 35 food
- game.rs: Core types (Snake, Food, GameState, ArenaBounds, Direction, GridPos)
- constants.rs: 60x60 grid, 12px cells, 8 tick/s
