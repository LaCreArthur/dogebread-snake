# Scratchpad — DogeBread Snake

## Current Focus: Goal 1 — Doge Theming (completing)

### Status Assessment
Doge theming is ~60% complete. The color palette and some key UI strings are done.

### What's Already Themed
- Color palette (DOGE_GOLD, dark background, golden food, golden-brown walls)
- Title screen: "DOGEBREAD SNAKE" + "such snake • very battle • wow"
- HUD: "much alive: X / Y • wow score: Z • kills: N"
- Game over: "such game over • wow", "VICTORY! very win! so champion!"
- Spectating: "much spectate • following X doge"
- Restart: "Press SPACE for much restart"

### What Needs Theming
1. **Snake names** — currently generic color names (Gold, Green, Red...). Should be doge-meme names like "Doge", "Cheems", "Shiba", etc.
2. **Kill feed messages** — "X killed by Y!" and "X eliminated!" are generic. Need doge-speak.
3. **Score popup** — "+1" is boring. Should be "wow" or "such coin"
4. **SPEED UP!** — generic. Needs doge flair
5. **Countdown** — 3, 2, 1, GO! could have doge text
6. **Timer text** — plain gray, could be more themed with doge gold color

### Plan
- Update snake names to doge-meme names
- Rewrite kill feed with doge speak
- Theme score popups, speed-up text, countdown
- Make timer text doge-gold
- Verify `cargo build -p client` compiles
- Commit

### Completed This Iteration
- All snake names updated to doge memes (Doge, Cheems, Bonk, Shibe, Floof, Bork, Snoot, Woofer)
- Kill feed fully doge-ified ("bonked by", "ded! much rip!", "crushed! such squish!")
- Score popups now cycle through 5 doge phrases
- Speed-up, countdown, start prompt, rankings, stats all themed
- Timer text color changed to doge-gold
- Build passes, all 72 tests pass
- Committed: 595713d

### Goal 1 Status: COMPLETE
Doge theming is now ~95% done. Every player-facing text uses doge-speak. Only missing actual Doge sprite assets, which would require an asset pipeline.

### Next iteration
- Goal 2: Sound effects (bevy_audio integration)

## Current Focus: Goal 2 — Sound Effects

### Approach
Procedural WAV generation in memory — no external audio assets needed.
- Generate sine waves, noise bursts, frequency sweeps as raw PCM
- Wrap in WAV header, create AudioSource, register as Bevy assets
- Play one-shot sounds at game events using AudioPlayer + PlaybackSettings::DESPAWN
- Enable `bevy/wav` feature in Cargo.toml

### Sound Design
| Event | Sound | Technique |
|-------|-------|-----------|
| Food eaten | Short happy bleep | Rising sine ~440→880Hz, 80ms |
| Snake death | Harsh crash | White noise burst + low sine, 200ms |
| Arena shrink warning | Alarm tone | Alternating 600/800Hz, 500ms |
| Arena shrink impact | Deep thud | Low sine 80Hz with decay, 300ms |
| Speed increase | Ascending sweep | Sine sweep 200→1200Hz, 300ms |
| Countdown beep | Short tick | Sine 880Hz, 100ms |
| Countdown GO! | Bright chord | Major chord burst, 200ms |
| Game over | Descending tone | Sine sweep 800→200Hz, 500ms |

### Plan
1. Enable `bevy/wav` feature in client/Cargo.toml
2. Create `client/src/audio.rs` with WAV generation + sound resource
3. Modify `lib.rs` to register audio module + play sounds at events
4. Build + test
5. Commit
