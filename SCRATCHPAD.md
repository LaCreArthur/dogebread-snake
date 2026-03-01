# Scratchpad

## Current State
ALL P0 COMPLETE. 8/8 P1 COMPLETE (minus 2 deferred). 3 P2 COMPLETE (menu flow added). DEPLOYED TO GITHUB PAGES. FULL TEST SUITE + CI. 85 tests passing, clippy clean, CI pipeline active. 16 sprints, 33 commits. Live at https://lacrearthur.github.io/dogebread-snake/

## Testing Coverage
- **62 unit tests** (shared/src/game.rs) — Direction, GridPos, Snake, ArenaBounds, SnakeColor, Food, edge cases
- **10 simulation tests** (shared/tests/simulation.rs) — headless game simulation, runs 100+ games as pure data, no engine dependency
- **CI pipeline** (.github/workflows/ci.yml) — cargo test + clippy -D warnings + fmt check + WASM build + size regression (<25MB)
- **Clippy clean** — zero warnings, Rust 2024 let-chains

## QA Approach Comparison
| Approach | Tests | Bugs Found | Time to Write | Time to Run | Value |
|----------|-------|------------|---------------|-------------|-------|
| Unit tests | 62 | 0 (but prevents regressions) | ~2 min | 0.00s | Foundation |
| Headless simulation | 10 | 0 (validates invariants) | ~2 min | <10ms | **Novel — highest value** |
| CI pipeline | - | Catches 19 clippy warnings | ~1 min | ~5 min on GH | Safety net |

**Key insight:** Headless simulation testing is the most valuable approach for game QA. It tests the *behavior* of the entire game loop without any engine overhead, runs in milliseconds, and catches categories of bugs that unit tests miss (infinite loops, statistical bias, invariant violations across full game runs).

## Remaining P1
1. **Sound effects** — blocked by "no external assets" constraint
2. **Player name entry** — complex Bevy text input, low priority

## Live URL
https://lacrearthur.github.io/dogebread-snake/

## To Rebuild & Redeploy
1. `./build-wasm.sh`
2. `git checkout gh-pages && cp web/* . && git add -A && git commit -m "Deploy" && git push && git checkout master`

## Architecture Notes
- effects.rs: FloatingText, DeathParticle, SpeedUpText, TrailParticle
- rendering.rs: GameOverAnimation (4-phase), all grid/snake/food/HUD/minimap rendering, auto-spectate
- input.rs: Player input + AI (Hungry/Cautious/Aggressive)
- lib.rs: System registration, game logic (tick, collisions, scoring), 10 snakes, 35 food, screen shake scaling
- game.rs: Core types (Snake, Food, GameState, ArenaBounds, Direction, GridPos) + 62 unit tests
- constants.rs: 60x60 grid, 12px cells, 8 tick/s
- tests/simulation.rs: Headless game simulation engine + 10 invariant tests
