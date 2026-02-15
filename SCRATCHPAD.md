# Scratchpad

## Current State
ALL P0 COMPLETE. Juicy 6-snake battle royale with countdown, kill feed, screen shake, particles, food pulse, speed indicator, safe entity despawn, distinct snake heads. 60x60 grid, effects.rs module extracted.

## Last Sprint
Sprint 4: Safe despawn (25 sites), bubble head, SPEED UP! text, effects.rs extraction.

## Next Steps (P1 — Should Have)
1. **10 snakes** — increase NUM_SNAKES, add more spawn positions
2. **Sound effects** — eat, die, kill, arena shrink, countdown
3. **Player name entry** — text input before match
4. **Better game over** — dramatic reveal, ranking animation
5. **Camera zoom** — zoom out as arena shrinks
6. **Smooth camera** — reduce lerp snap
7. **WASM deploy** — verify build, deploy to GitHub Pages

## Blockers
None.

## Notes
- effects.rs now handles all transient visual effects (SOLID)
- rendering.rs re-exports effects functions for backward compat
- GameState has 4 variants: WaitingToStart, Countdown, Playing, GameOver
- Bevy audio: probably needs bevy::audio feature and AudioSource/AudioPlayer
- Sound is highest-impact P1 — transforms the entire feel
