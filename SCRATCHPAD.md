# Scratchpad

## Current State
Playable 6-snake battle royale. Core mechanics solid. User QA verdict: "great first stage" but map too small and needs juice/UX feedback. Entity despawned warnings in console from dead snake cleanup race condition.

## Last Sprint (Pre-orchestration)
10 manual commits building up from scratch to current state. Covered: snakes, AI, arena shrink, scoring, minimap, spectating, start/game over screens.

## Next Steps (Priority Order)
1. **Bigger map** — increase grid to 60x60 with smaller cells or camera zoom. Most impactful single change.
2. **Juice pass** — screen shake, score popups, food pulse, death particles. This is what separates "tech demo" from "game".
3. **Countdown + kill feed** — UX essentials for battle royale feel.
4. **Fix entity despawned warnings** — cleanup race condition.
5. **Snake head distinction** — eyes or size to make head visually clear.

## Blockers
None.

## Notes
- Bevy 0.18 API is well-understood now (see LEARNINGS.md). Should be faster going forward.
- Window is 800x700. With 60x60 grid at 12px cells = 720px, fits well. Or keep 16px cells but use camera zoom.
- Screen shake in Bevy: modify camera transform with decaying offset per frame.
- Floating text: spawn Text entity at world position, animate upward + fade out over ~1s.
- Food pulse: oscillate sprite size using sin(time).
- Death particles: spawn N small sprites that fly outward from death position.
