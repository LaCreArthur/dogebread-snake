# Scratchpad

## Current State
ALL P0 COMPLETE. 7/8 P1 COMPLETE. 2 P2 COMPLETE. Polished 10-snake battle royale with smooth camera, arena zoom, trail particles, dramatic 4-phase game over, auto-spectate, screen shake scaling, WASM build pipeline (20MB optimized). 26 commits total.

## Remaining P1
1. **Sound effects** — blocked by "no external assets" constraint. Would need procedural audio or embedded WAV bytes.
2. **Player name entry** — text input in Bevy is non-trivial. Low priority for PvE demo.

## Remaining P2
- Full menu flow: Home → Play → Game → Leaderboard → Home
- Doge sprites/assets (actual Doge head, coin pickups)
- Match stats history (local storage in WASM)
- Mobile touch controls (WASM)
- GitHub Pages deployment via GitHub Actions (infrastructure)

## What's Shippable Now
The game is feature-complete for a PvE demo. To share:
1. `./build-wasm.sh`
2. `cd web && python3 -m http.server 8080`
3. Open http://localhost:8080

For public hosting: push web/ artifacts to a gh-pages branch or set up GitHub Actions.

## Architecture Notes
- effects.rs: FloatingText, DeathParticle, SpeedUpText, TrailParticle
- rendering.rs: GameOverAnimation (4-phase), all grid/snake/food/HUD/minimap rendering, auto-spectate
- input.rs: Player input + AI (Hungry/Cautious/Aggressive)
- lib.rs: System registration, game logic (tick, collisions, scoring), 10 snakes, 35 food, screen shake scaling
- game.rs: Core types (Snake, Food, GameState, ArenaBounds, Direction, GridPos)
- constants.rs: 60x60 grid, 12px cells, 8 tick/s
