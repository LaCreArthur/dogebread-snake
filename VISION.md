# Vision — DogeBread Snake

## Goal
A polished, juicy snake battle royale playable in-browser that feels like a real game, not a tech demo. 10 snakes, satisfying feedback on every action, Doge-themed.

## Priorities (ordered)

### P0: Must Have (ship-blocking)
- [x] Core snake mechanics (move, eat, grow, die)
- [x] Battle royale (arena shrink, last alive wins)
- [x] Multiple AI opponents with personality
- [x] Game state flow (start → play → game over → restart)
- [x] Scoring + kill tracking
- [x] Minimap
- [x] Camera follow player
- [x] Bigger map — 60x60 grid, 12px cells, 900x780 window
- [x] Juice: screen shake on death/kill
- [x] Juice: score popup when eating food (+1 floating text)
- [x] Juice: food pulsing/glowing animation
- [x] Juice: death explosion (particles or expanding ring)
- [x] Juice: arena shrink warning (flash/shake before shrink)
- [x] Juice: speed lines or visual indicator when speed increases
- [x] Snake head distinct from body (eyes, shape, or size difference)
- [x] Kill feed (top-right, "Red killed Blue", fades out)
- [x] Countdown (3-2-1-GO) before match starts
- [x] Fix entity despawned warnings (race condition in cleanup)

### P1: Should Have
- [x] 10 snakes (Sprint 5)
- [ ] Sound effects (eat, die, kill, arena shrink, countdown)
- [ ] Player name entry before match
- [x] Better game over screen — phased reveal, colored rankings, crown (Sprint 6)
- [x] Camera zoom out slightly as arena shrinks (Sprint 5)
- [x] Smooth camera — delta-time exponential smoothing (Sprint 5)
- [x] Trail/afterimage effect on fast snakes (Sprint 7)
- [x] WASM build verified + build script (Sprint 8) — GitHub Pages deploy pending

### P2: Nice to Have
- [ ] Full menu flow: Home → Play → Game → Leaderboard → Home
- [ ] Doge sprites/assets (actual Doge head, coin pickups)
- [ ] Screen shake intensity scales with kills
- [ ] Spectate: auto-follow strongest remaining snake
- [ ] Match stats history (local storage in WASM)
- [ ] Mobile touch controls (WASM)

## Constraints
- Rust + Bevy 0.18 (no engine switch)
- No external asset files yet (procedural/code-generated visuals only)
- Single-player only (multiplayer is Phase 3, out of scope)
- Build must compile and run natively + WASM
- Auto-screenshot system must keep working for AI verification

## Stop Conditions
- All P0 items complete
- 3 consecutive sprint failures
- User says stop
