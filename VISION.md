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
- [ ] Bigger map — current 40x40 (16px cells) feels cramped. Target: 60x60+ with 12-14px cells or zoom
- [ ] Juice: screen shake on death/kill
- [ ] Juice: score popup when eating food (+1 floating text)
- [ ] Juice: food pulsing/glowing animation
- [ ] Juice: death explosion (particles or expanding ring)
- [ ] Juice: arena shrink warning (flash/shake before shrink)
- [ ] Juice: speed lines or visual indicator when speed increases
- [ ] Snake head distinct from body (eyes, shape, or size difference)
- [ ] Kill feed (top-right, "Red killed Blue", fades out)
- [ ] Countdown (3-2-1-GO) before match starts
- [ ] Fix entity despawned warnings (race condition in cleanup)

### P1: Should Have
- [ ] 10 snakes (currently 6)
- [ ] Sound effects (eat, die, kill, arena shrink, countdown)
- [ ] Player name entry before match
- [ ] Better game over screen (dramatic reveal, stats, ranking animation)
- [ ] Camera zoom out slightly as arena shrinks
- [ ] Smooth camera (currently lerp is too snappy)
- [ ] Trail/afterimage effect on fast snakes
- [ ] WASM build verified + deployed to GitHub Pages

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
