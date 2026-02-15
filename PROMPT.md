# DogeBread Snake — Continuous Improvement Loop

You are in a Ralph loop. Each iteration starts with FRESH CONTEXT.
State lives in files. Learnings persist. Failures evaporate.

---

## Phase 1: RE-ANCHOR (do this FIRST)

1. **Scratchpad** (handoff from last iteration): `.ralph/agent/scratchpad.md`
2. **Learnings**: `LEARNINGS.md`
3. **Git state**: `git log --oneline -5 && git status`
4. **Epic goals below**

---

## Epic: Polish & Web Release

### What exists
- 6-snake battle royale in Rust/Bevy 0.18 (~1700 lines)
- WASM build working (build-web.sh → web/index.html)
- Full game loop, AI personalities, minimap, arena shrinking

### Remaining goals (prioritized)
1. **Doge theming** — gold/meme colors, "wow such snake" flavor text, themed UI
2. **Sound effects** — eat, death, arena shrink warning, game over  
3. **Visual polish** — particle effects, screen shake, smooth camera transitions
4. **Menu flow** — title screen with Doge branding, name entry
5. **WASM optimization** — 85MB is too big, add wasm-opt, tune Cargo.toml
6. **Game feel** — tune speeds, timing, AI difficulty

### Constraints
- Stay on Bevy 0.18
- Don't break existing gameplay
- Keep shared/ clean for future multiplayer

---

## Phase 2: WORK

Pick the NEXT unfinished goal from the list above. Check scratchpad for what was done last.

**Before complex action:**
- Hypothesis: What do I expect to happen?
- Success criteria: How will I verify?

**During work:**
- Stuck >5 min? STOP. Research online. Challenge the blocking assumption.
- Found simpler way? Note it in scratchpad.

---

## Phase 3: SELF-IMPROVING GATES

### Gate 1: Validation
- [ ] `cargo build -p client` succeeds
- [ ] No regressions in gameplay

### Gate 2: Scratchpad Update
Update `.ralph/agent/scratchpad.md`:
- What goal you worked on
- What you accomplished
- What to do next
- Any blockers

### Gate 3: Learning Capture
If genuinely new insight: add ONE entry to `LEARNINGS.md`

### Gate 4: Git Commit
Commit your work with a descriptive message.

### Gate 5: Progress Assessment  
- All 6 goals complete? → Output `<promise>LOOP_COMPLETE</promise>`
- More work to do? → Let the loop continue to next iteration

---

**DO NOT output LOOP_COMPLETE until ALL goals are genuinely addressed.**
**Each iteration should make meaningful progress on exactly ONE goal.**
