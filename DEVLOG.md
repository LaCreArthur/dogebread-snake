# Devlog — DogeBread Snake

## 2026-02-15 00:39–01:00 — From scratch to playable battle royale

### What was built
Full snake battle royale game in Rust/Bevy 0.18, coded entirely by AI (Claude) with zero editor:
- 6-snake arena (1 player + 5 AI with 3 personality types)
- 40x40 grid, camera follow, minimap
- Arena shrinking every 12s (battle royale mechanic)
- Speed increase over time
- Score + kills tracking, game over scoreboard
- Death blink animation, spectating mode
- Full game state flow: WaitingToStart → Playing → GameOver

### 10 commits in ~21 min active time
1. Initial 4-snake with AI
2. Gameplay polish (longer snakes, dead cleanup)
3. Visible arena borders
4. **Arena shrinking + WaitingToStart** (biggest feature)
5. Scoring, larger grid (30→40), scoreboard
6. Kill credit tracking
7. Death animation, centered camera
8. 6 snakes, AI personalities, 2-step look-ahead
9. Minimap
10. QoL (spectating, ESC quit, start overlay)

### Key learnings (see LEARNINGS.md)
- Bevy 0.18 API breaks heavily from examples online — migration guide is mandatory
- Auto-screenshot via Bevy native API + env var toggle = autonomous visual verification
- ECS query conflicts solved with `Without<T>` filter
- Iteration speed accelerated: early commits had 3-4 compile errors, last 5 were clean first try

### What's NOT obvious
- The project was scaffolded across 2 sessions (first session: Rust install, API research, initial scaffolding; second session: this devlog's 21 min of rapid feature work)
- `shared/` crate holds all game logic (designed for future server-authoritative multiplayer with Lightyear)
- `server/` and `protocol.rs` are placeholder stubs for Phase 3 (networking)
- SimpleRng uses PCG algorithm to avoid external rand dependency
- Grid cells store their GridPos for dynamic recoloring during arena shrink
