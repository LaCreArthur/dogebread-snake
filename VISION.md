# Vision: DogeBread Snake

## Goal
10 real humans in one match. Free browser snake battle royale for Dogecoin fans, desktop + mobile, no
install: share a link, friends are in the same match seconds later. 10 snakes, ~3 min, arena shrinks,
last Doge alive wins. Bots fill every empty seat, so a match is fun with 1 human or 10.

## Stages (agreed 2026-10-05)
1. **Rebuild + room links** (current). TypeScript rebuild; private link rooms, host starts, bots fill.
   Switch-over (delete the Rust/Bevy game) the day two humans can play one room; port effects after.
   Done when: deployed on Cloudflare free plan; 10 people play one match from one shared link.
2. **Public arena** (replaces a matchmaking queue): one public room loops matches back to back;
   newcomers spectate and join the next round; a second arena opens only when the first is full.
3. **Paid rooms** (optional, needs only stage 1): 1 DOGE entry, last paying snake alive takes the pot,
   Dogecoin L1, link rooms among friends only, testnet first. Design in STATE.md.

Growth (Arthur's side): one weekly fixed match time in a Telegram group he owns; 10 named RSVPs for the
first date; stage 1 is built for that date.

## Architecture (decided 2026-10-05)
- TypeScript everywhere. Engine-free rules module runs only on the server and in tests.
- Client: Canvas 2D arena, HTML overlay UI, Web Audio. Target < 300 KB, playable 1-2 s after a link
  tap, must work in Telegram's in-app browser.
- Server: Cloudflare Workers + one Durable Object per room (SQLite-backed, free plan); static assets on
  the same Worker.
- Network: server owns rules, randomness and bots; each step it broadcasts outcomes (new head
  positions, headings, lengths, deaths, boops, crumbs eaten and spawned); clients rebuild bodies from
  head trails with the server's own geometry code. Snapshot on join. Players send a target heading on
  change (at most 10 a second); an arrow on the head shows it at once. Own-snake prediction comes
  next. The stream is the replay.
- Names: generated Doge names (word lists, reroll), never free text.
- Abuse limits: server-issued room codes only, rooms run only while a human is connected, idle rooms
  close, per-socket message cap.

## Constraints
- Free hosting only (Cloudflare free plan: 100k req/day, 13,000 GB-s/day DO duration).
- Fun non-commercial POC. No ads, no accounts.
- Not a Sorolla/TinyForge game: the studio standard (iOS, paid UA, ads) does not apply.
