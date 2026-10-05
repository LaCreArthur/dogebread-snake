# State

Goal: VISION.md stage 1 (TypeScript rebuild + room links, bots fill, deployed on Cloudflare free plan,
Rust game deleted at switch-over).
Mode: pair (Arthur at the keyboard; his playtest is the judge). Started 2026-10-05.
Loop job: 62a0a19e (cron */20, "/iterate tick", session-only, expires 2026-10-12)
Branch: ts-rebuild (master keeps the Rust game until switch-over).

## Current
- Cycle 1 done: rules port + Room Durable Object + canvas client + room links + host start + dev
  "+ AI player" (joins over WebSocket, steers with botDir on its replica). Smoke-tested at 390x844:
  create, 3 AI players, start, countdown, turns reach the server, replicas in sync, reload mid-match
  keeps the seat (15 s empty-room grace), no console errors. typecheck, 6 vitest tests, build pass.
- Cycle 2: design proposal for Arthur ("current version is weak, 10x more fun/addictive, question every
  assumption: grid, rules, balance"; his concept art: dogebread.io, free-angle doge-head + bread-slice
  body, food skins: garlic bread, baguette, taco, hot dog). Specialist design pass -> roadmap attack ->
  AskUserQuestion. Nothing built until he picks.

## Measured (bot-only, 200 seeds, Rust brain)
- Match length median 50 s, p90 71 s, max 121 s; arena never below 38 wide, so the shrink rarely
  matters. 776 bot deaths in the first 5 s (766 head-on: symmetric spawns + no head-on check).
- Head-on avoidance (penalize cells another head can enter) + dead-end fallback: 120 early deaths,
  median 64 s. Reverted 2026-10-05: design change, waits for Arthur.

## Rules reference (ported from the Rust game, client/src + shared/src)
- Grid 60x60, walls at 0 and 59, playable 1..58. 10 snakes, 35 food, initial length 5.
- Step 125 ms; every 20 s step interval x0.85 (min 60 ms). Arena shrinks 1 cell per side every 12 s
  until inner size <= 6; warning 2 s before.
- Step: all alive snakes move (queued dir, no 180), then collisions vs post-move state: out of arena,
  self body, other alive body (killer +1 kill), head-to-head (both die, no killer). Food on head:
  +2 growth, +1 score. Refill food to 35 inside bounds (min+1..max-2). After shrink, heads outside die
  ("crushed"), food outside removed. Over when alive <= 1.
- Bots: Hungry/Cautious/Aggressive by id%3; 1-step safety + 2-step exit check; score = food pull,
  wall distance, head attraction/repulsion, +3 keep direction.
- Spawns: 10 fixed positions + directions (see src/game/rules.ts).
- Text: "such 3 / very 2 / much 1 / WOW GO!", "X bonked by Y! wow", "X ded! much rip!",
  "X crushed! such squish!".

## Decisions
- 2026-10-05 Stage plan, stack, public arena instead of queue, no prediction, generated names,
  L1 for paid rooms: see VISION.md.
- 2026-10-05 Not started with `tinyforge create`: TinyForge's standard is for Sorolla iOS paid-UA
  games; this is a solo multiplayer web POC.
- 2026-10-05 Non-hibernating WebSockets (`server.accept()`): simplest; idle rooms close to protect the
  free duration quota.
- 2026-10-05 Room link = `/?r=CODE` (no SPA routing config).
- 2026-10-05 Arthur: no game design change (movement, rules, rounds, balance, bots) without his OK
  first. Polish and UI/UX may go 10x without asking. Graphics can wait: working POC, clean systems.

## Paid rooms design (stage 3, not started)
Dogecoin L1, custodial per room: one HD seed as Worker secret (separate per network, offline backup);
one deposit address per seat; seat counts at 1 confirmation; payout address from MyDoge connect or
typed; host starts at >= 2 paid; bots fill but cannot win (last paying snake alive takes pot, tie on
same step splits); unpaid humans spectate; one closing tx pays winner minus fee (~0.015 DOGE) and
refunds extras; void close (expiry/cancel/crash) refunds all; cap open paid rooms (5 -> max 50 DOGE).
Testnet: faucet.doge.toys, QED Esplora testnet API (doge-electrs-testnet-demo.qed.me, no key, CORS),
unifralabs DogecoinTestnetWallet. MyDoge extension API: connect, requestTransaction (repo archived
2026-10-05; QR/pay link is the fallback that always works).

## Avoid
- 2026-10-05 vitest reading vite.config.ts with @cloudflare/vite-plugin: "There is already a server
  associated with the config". Tests use their own vitest.config.ts.
