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
- Cycle 3 done: sounds ported (mute button); dev `?lag=MS` simulates a round trip (half each way,
  HUD shows it) so Arthur can feel grid controls at deployed latency before the grid-vs-steering pick.
  Measured: lag=200 -> own turn visible after 232 ms.
- Cycle 4 done (polish, design-neutral): dead snakes burst into crumb particles (replaces the 2 s grey
  blink), screen shake (my death 14 px, my bonk 7; nearby deaths and shrink cut after the attack), gold sparkle
  on every coin eaten, "+1" pop on my coins, "+1 bonk" pop + "much bonk! wow" banner + new procedural
  "bonk" sound on my kills, camera holds 1.2 s on my crumbs before following the leader, my own name
  label removed (it covered my pops; the ring marks me). Replay: .playwright-mcp/juice-replay.js
  (browser_run_code_unsafe with filename; eats a coin, dies on a wall, returns console errors).

## Measured (bot-only, 200 seeds, Rust brain)
- Match length median 50 s, p90 71 s, max 121 s; arena never below 38 wide, so the shrink rarely
  matters. 776 bot deaths in the first 5 s (766 head-on: symmetric spawns + no head-on check).
- Head-on avoidance (penalize cells another head can enter) + dead-end fallback: 120 early deaths,
  median 64 s. Reverted 2026-10-05: design change, waits for Arthur.

## Design proposal v1 (specialist pass 2026-10-05; PENDING Arthur, nothing built)
u = new-snake width. Bots obey all human rules.
- Hook: the cut-off (boost across a nose, victim bursts into crumbs). Any size kills any size.
- Move: constant 8 u/s, boost 16 u/s; turn 240 deg/s / width (circle ~4 widths), x2 while boosting.
  Boost burns 8 len/s dropped as tail crumbs; no boost below len 20. Phone: drag anywhere = heading
  (12 px dead zone, anchor follows past 60 px), double-tap-hold or 2nd finger = boost. Desktop: steer
  to mouse, hold click/space = boost, arrows turn. Net: target angle byte + boost bit on change, <=10/s;
  stream heads only, clients rebuild bodies from head paths (~2 KB/s).
- Death: head on other body or wall. Own body harmless. Head-head (or first slice): both bounce "BONK",
  nobody dies. Dead length: 25% to killer, 50% as golden crumbs (5 each) along body, 25% lost.
  Width +1 u per 400 len, cap 2.5 u; 1 slice per 4 len, 0.5 u apart.
- Rounds: 3:00 timed, respawn after 2 s at len 40, farthest from heads, 2 s ghost. Longest HUMAN at
  buzzer wins; exact tie splits pot. Start: 3-2-1 freeze on ring r=45 u facing clockwise. Wall r=75 u,
  closes to 30 u from 2:00 to 2:50, kills; crumbs held at 400 inside. Crown on longest human +
  edge arrow. Death card names killer; bonk them back = "REVENGE". Results 8 s (winner, bonks, room
  win tally) -> 5 s countdown -> next round auto; pot rounds need Ready (all ready or 20 s).
  Link opens into the live arena (join in progress).
- Retention (localStorage): skins by feats (toast default; garlic 10 bonks; baguette 400 len one life;
  taco first win; hot dog bonk the crown). Bests: longest, bonks/round, wins ("NEW BEST. wow.").
  Titles by lifetime wins 1/5/15/40. No daily streaks.
- Bots: fill to 12 snakes, grey names, human input limits, one screen vision, 300-500 ms reaction,
  +-10 deg aim error. Snacc 60% (graze, no boost, dodge late), Hungry 30% (eat death crumbs, cut off
  only bigger snakes), Bonker max 1 (hunt crown last 90 s). Never wall, never boost into a body.
  Never win: crown/leaderboard humans only; bot stops eating above 80% of longest human.
- Numbers: 12 snakes/room (16 public). Arena circle r=75 u. View ~3,500 u^2 on every device
  (portrait ~40x87 u), zoom out 30% per extra unit of width. Start len 40. 400 crumbs x1, magnet
  1.5 widths. ~1 len/s from crumbs; winners ~400-800.
- Onboarding: card (name, skins, PLAY) over live arena; ghost spawn facing crumb trail + "drag
  anywhere" thumb; 10 s "double-tap + hold = much zoom"; first-ever life: scripted Snacc suicide
  "WOW. FIRST BONK."
- Juice top 5: bonk burst (crumbs, shake, "+120 much bonk", feed); crunch pitch ladder; doge face looks
  where you steer instantly (hides delay), panics near bodies, x_x on death; near miss "wow. close.";
  "NEW TOP DOGE" + heartbeat last 10 s.
- Risks: late/hidden phone controls (deaths with input turned away >=250 ms before impact >15%; no
  boost in round 1 >30%; first fix = own-head prediction); runaway/hiding leader (leader at 1:30 wins
  <60%, >=1 crown bonk/round); dead air/bot-run rooms (encounter every <=8 s, bot-caused human deaths
  20-35%, 0 bot wall deaths).

### Attack (fresh agent 2026-10-05) and answers; v2 = v1 + every ACCEPT
- Systemic: v1 feeds the leader 3 snowballs (flat boost cost, width-scaled magnet, unlimited crumbs in
  the closing zone) then adds 3 bot counterweights (Hungry hunts bigger, Bonker hunts crown, 80% cap).
  ACCEPT: delete both sets (fixes below).
- Hook: ACCEPT + tag each kill cut-off (killer boosting, crossed the victim's path within 1 s) or
  blunder; cut-off share is the POC pass/fail number.
- Boost cost: ACCEPT ~2% of len/s, min 4/s (flat 8/s = 20%/s at len 40, 1%/s at 800).
- Netcode: ACCEPT own-snake prediction from day one, others projected forward from angle + boost bit;
  first input change sent at once, then throttled; server collides on the same 20 Hz path clients
  draw. Delay budget: RTT + <=50 ms tick wait + <=100 ms throttle + 50-100 ms interp ~ 5 u at boost.
- Boost gesture: ACCEPT A/B in the POC: thumb past a drawn outer ring vs double-tap-hold (+250 ms,
  invisible).
- Turtle ring: ACCEPT the problem (own body harmless + 4-width circle: past ~130-150 len a coiled
  leader is untouchable, heads only bounce). Attack fix: burn (lost) length while the head overlaps
  its own body. PARTIAL: a ring of radius >= body/2pi never overlaps, so ship the burn and measure
  "leader at 2:00 wins <50%"; if rings persist, next candidates: own body lethal, or the final circle
  closes to ~0.
- Drops: ACCEPT only length above 40 drops on death (suicide farming); boost drops lose half; a
  snake cannot eat its own drops (friend feeding, crumb banking).
- Head-head: ACCEPT rename "boop" (BONK = kill only); both lose a few slices (no free shove).
- Crumb density: ACCEPT no spawning after 2:00, the wall sweeps crumbs inward, deaths feed the finale
  (v1 held 400 inside the closing wall: ~2,200 len in the last minute vs ~1,400 in the first two).
- Spawns: ACCEPT score points inside 70% radius against heads AND bodies, face center, no ghost
  (ghost-laying exploit).
- Endless no-zone arena (v1 section 5): ACCEPT cut; stage 2 public arena gets its own design.
- Retention: ACCEPT wins count only with 2+ humans; the room win tally lives in the Room DO.
- Bots: ACCEPT one type (graze, ignore golden crumbs, dodge late, never target by rank), no 80% cap,
  arena radius ~ sqrt(snake count). PARTIAL on "bots <= half the room": keep a floor of 6 snakes so
  solo and duo rooms (Arthur's main test today) have action. Max 10 humans (VISION.md).
- Magnet: ACCEPT constant ~2 u (width-scaled = 2.5x eat rate at full width). Winning range: re-measure.
- Onboarding: ACCEPT teach the cut-off (slow Snacc crossing ahead, "boost across its nose"); "much
  zoom" reads as camera zoom, rename. Deferred until after the POC.
- Juice: ACCEPT shake only on my death or my bonk (applied to the grid build 2026-10-05); the
  face-turn is replaced by prediction.
- Refresh to deny a kill: ACCEPT a disconnected snake keeps its last input until it dies or returns.
- Pot rounds: ACCEPT roster lock to Ready players (stage 3).
- Money/legal: DISMISS (Arthur 2026-10-05: no gambling-law analysis, fun non-commercial POC; the
  custodial L1 design stands).
- POC scope ACCEPT: 1 room of 6-8 snakes, phone + desktop steer/boost, prediction, cut-off kills with
  25/50 drops, boop, safe 2 s respawn at 40, self-overlap burn, crumbs, 3:00 round + closing wall (no
  late spawns), crown arrow, kill feed, auto rematch, 1 bot type, lag 0/150/300. Defer: pot, skins,
  titles, bests, onboarding, bot tiers, revenge, most juice, public arena. Pass/fail: cut-off share;
  deaths called fair at 300 ms; leader at 2:00 wins <50%; friends start a 3rd round.

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
- 2026-10-05 Playwright MCP refuses screenshot and code files outside the project ("outside allowed
  roots"): keep replay scripts and captures in .playwright-mcp/ (gitignored), not the scratchpad.
