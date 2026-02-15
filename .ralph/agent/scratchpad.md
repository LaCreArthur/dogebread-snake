# Scratchpad

## Last Iteration (Sprint 1 — WASM Build)
- WASM compilation working via build-web.sh
- Refactored main.rs → lib.rs for wasm-bindgen
- web/index.html loads the game module
- 85MB wasm output — needs optimization later
- NOT tested in actual browser yet

## Next Up
- Goal 1: Doge theming — start with color palette and flavor text
- All rendering code is in client/src/rendering.rs
- Game logic in client/src/lib.rs
- Constants in shared/src/constants.rs

## Known Issues
- ui.rs is empty (1 line)
- server/ is placeholder stubs
