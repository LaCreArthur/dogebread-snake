# Sprint Log

## Pre-orchestration — 2026-02-15 00:39–01:00
- **Task:** Build complete snake battle royale from scratch
- **Model:** opus (manual, no subagents)
- **Result:** success
- **Changes:** 11 commits (dabb541..8d6ee23), 4 source files, full game loop
- **Notes:** First Bevy 0.18 project. Heavy API learning curve early, accelerated fast. Entity despawned warnings remain.

## Sprint 1 — 2026-02-15 12:01
- **Task:** Expand map from 40x40 to 60x60 grid
- **Model:** sonnet
- **Result:** success
- **Changes:** 76f73f7 — constants, spawn positions, food count, window size, minimap
- **Notes:** Straightforward. Minor minimap dot clipping at boundary.

## Sprint 2 — 2026-02-15 12:03
- **Task:** Juice pass — 5 visual feedback systems
- **Model:** opus
- **Result:** success
- **Changes:** 283ff02 — screen shake, score popups, food pulse, death particles, shrink warning (272 insertions)
- **Notes:** Agent correctly used Text2d for world-space score popups. Clean build.

## Sprint 3 — 2026-02-15 12:08
- **Task:** Countdown + kill feed + Doge HUD
- **Model:** opus
- **Result:** success
- **Changes:** 0648a0f — Countdown game state, 3-2-1-GO!, kill feed with fade, arena crush deaths in feed (206 insertions)
- **Notes:** Added Countdown variant to GameState. Kill feed also covers arena shrink deaths. One compile error (Children iterator) caught and fixed. Sprint 3b (entity despawn fix, sonnet) ran in parallel but got overwritten — lesson: don't run parallel sprints on same files.

## Sprint 3b — 2026-02-15 12:08
- **Task:** Fix entity despawned warnings
- **Model:** sonnet
- **Result:** partial (overwritten)
- **Changes:** None committed — Sprint 3 overwrote changes
- **Notes:** Used commands.get_entity() pattern. Correct approach but got clobbered by parallel Sprint 3. LEARNING: parallel sprints on same files = merge conflicts. Use SOLID / separate modules.

## Sprint 4 — 2026-02-15 12:18
- **Task:** Safe despawn + snake head distinction + speed indicator + effects module extraction
- **Model:** opus
- **Result:** success
- **Changes:** 4da7011 — 25 safe despawn conversions, bubble head effect, SPEED UP! text, new effects.rs (242 insertions, 127 deletions)
- **Notes:** Extracted effects.rs following SOLID. Zero despawn warnings confirmed. All P0 items now complete.

## Sprint 5 — 2026-02-15 12:35
- **Task:** 10 snakes + smooth camera + arena zoom
- **Model:** sonnet
- **Result:** success
- **Changes:** 4c1393b — NUM_SNAKES 6→10, NUM_FOOD 25→35, 2 interior spawn positions, delta-time camera smoothing, Projection::Orthographic zoom on arena shrink (19 insertions, 5 deletions)
- **Notes:** Used exponential decay smoothing `1.0 - (-5.0 * delta_secs).exp()` for frame-rate-independent camera. Zoom via `Projection::Orthographic(ortho)` pattern.

## Sprint 6 — 2026-02-15 12:40
- **Task:** Dramatic game over screen with phased reveal
- **Model:** opus
- **Result:** success
- **Changes:** 0274af0 — 4-phase animated game over (overlay fade, title, winner announcement, colored rankings with crown/YOU markers, stats, restart prompt), GameOverAnimation resource, 5 new marker components (274 insertions, 43 deletions)
- **Notes:** Complex multi-phase animation system. Rankings sorted alive-first, then score/kills. Player row highlighted gold. Winner gets crown symbol. Stats footer shows total kills.

## Sprint 7 — 2026-02-15 12:39
- **Task:** Trail afterimage effect for snake movement
- **Model:** sonnet (parallel with Sprint 6)
- **Result:** success
- **Changes:** c65bf66 — TrailParticle component, TrailSpawner resource (0.15s interval), spawn at segment[1], 0.4s fade, 40% initial opacity, 60% cell size (75 insertions)
- **Notes:** Successfully ran parallel with Sprint 6 — different files (effects.rs vs rendering.rs). SOLID module separation prevented conflicts.

## Sprint 8 — 2026-02-15 12:43
- **Task:** WASM build pipeline and deployment setup
- **Model:** sonnet
- **Result:** success
- **Changes:** 4d8ee54 — build-wasm.sh script, .gitignore for WASM artifacts (21 insertions)
- **Notes:** WASM compiles in release mode. wasm-bindgen generates client.js (105K) + client_bg.wasm (73M). index.html import path matches. Local serving via python3 http.server verified. 73M WASM is large — needs wasm-opt or LTO for production.

## Sprint 9 — 2026-02-15 12:50
- **Task:** Screen shake scaling + auto-spectate strongest snake
- **Model:** sonnet (parallel with Sprint 10)
- **Result:** success
- **Changes:** 027be6e — shake intensity scales with player kills (8.0 + kills*2.0, capped 20.0), auto-spectate follows strongest alive snake when player dead, "much spectate • following [Color] doge" text
- **Notes:** Hit borrow checker: `cannot borrow snake_query as immutable because also borrowed as mutable`. Fixed by extracting player_kills count before mutable death processing loop. Unused variable warning fixed with `_entity` prefix.

## Sprint 10 — 2026-02-15 12:50
- **Task:** WASM size optimization — release profile + wasm-opt
- **Model:** sonnet (parallel with Sprint 9)
- **Result:** success
- **Changes:** 13a7e02 — release profile (opt-level="z", lto=true, codegen-units=1, strip=true), wasm-opt -Oz in build-wasm.sh
- **Notes:** WASM reduced from 73MB → 20MB (73% reduction). Still large — could disable unused Bevy features for further reduction.

## Sprint 11 — 2026-02-15 13:55
- **Task:** GitHub repo creation + GitHub Pages deployment
- **Model:** opus (manual)
- **Result:** success
- **Changes:** d445c5a (untrack WASM artifacts), gh-pages branch deployed
- **Notes:** Created public repo LaCreArthur/dogebread-snake. Untracked WASM artifacts from main (were committed before .gitignore). Created orphan gh-pages branch with pre-built web/ contents. GitHub Pages live at https://lacrearthur.github.io/dogebread-snake/. Note: 85MB WASM in git history (pre-optimization commit) triggers GitHub warning but is not blocking.

## Sprint 12 — 2026-02-15 14:15
- **Task:** Comprehensive unit tests for shared/ game logic
- **Model:** general-purpose agent
- **Result:** success
- **Changes:** dbcd0cc — 62 unit tests in shared/src/game.rs
- **Notes:** Tests cover Direction (5), GridPos (13), Snake (22), ArenaBounds (12), SnakeColor (3), Food (1), cross-entity (2). All edge cases: boundaries, 180° turns, collision detection, arena shrink limits.

## Sprint 13 — 2026-02-15 14:15
- **Task:** Headless game simulation tests — novel E2E without engine
- **Model:** general-purpose agent (parallel with Sprint 12)
- **Result:** success
- **Changes:** 4de0db0 — 10 simulation tests in shared/tests/simulation.rs
- **Notes:** Novel approach: complete game simulation as pure data operations, no Bevy/ECS. GameSim engine (~200 lines) runs full battle royale games with seeded RNG. Tests: game termination (50 games), score validity (20 games), dead snake immutability, statistical fairness (100 games, <40% win rate per snake). All 10 tests pass in <10ms total. Key insight: decoupled game logic from engine makes fast deterministic testing trivial.

## Sprint 14 — 2026-02-15 14:15
- **Task:** CI pipeline — tests, clippy, WASM build check
- **Model:** general-purpose agent (parallel with Sprints 12-13)
- **Result:** success
- **Changes:** 20e3b7d — .github/workflows/ci.yml, rustfmt.toml
- **Notes:** 3 parallel CI jobs: test (cargo test --workspace), lint (clippy -D warnings + fmt check), wasm-build (compile + wasm-bindgen + size check <25MB). Triggered on push to master + PRs. Also added rustfmt.toml (edition 2024, max_width 120).

## Sprint 15 — 2026-02-15 14:30
- **Task:** Fix all clippy warnings (19 issues) to pass CI
- **Model:** opus (manual)
- **Result:** success
- **Changes:** c4901d1 — 4 files, 87 insertions, 91 deletions
- **Notes:** Collapsed nested if/if-let into Rust 2024 let-chains. Replaced iter().any() with contains(). Derived Default for ShrinkWarning. Combined identical if/else branches. Added #[allow(too_many_arguments)] for Bevy system functions (ECS parameters). All 72 tests pass + clippy clean.

## Sprint 16 — 2026-02-15 14:45
- **Task:** Event-driven visual testing — AUTO_TEST mode
- **Model:** general-purpose agent + opus (visual review)
- **Result:** success
- **Changes:** 8247c56 + 904d012 — AutoTestState resource, auto_test_system, 9 event-triggered screenshots
- **Notes:** Novel approach: captures screenshots at MEANINGFUL game events instead of random intervals. 9 checkpoints: countdown-3, countdown-GO, gameplay-start, first-death, arena-shrink, late-game, gameover-title, gameover-rankings, gameover-complete. Auto-starts, auto-exits. All visual systems verified working: grid, snakes, food, HUD, kill feed, spectate, arena shrink/danger zone, camera zoom, game over phased animation. Run with `AUTO_TEST=1 cargo run --release`.
