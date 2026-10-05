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
  AskUserQuestion. Arthur picked 2026-10-05 (see "Arthur's picks" below).
- Cycle 3 done: sounds ported (mute button); dev `?lag=MS` simulates a round trip (half each way,
  HUD shows it) so Arthur can feel grid controls at deployed latency before the grid-vs-steering pick.
  Measured: lag=200 -> own turn visible after 232 ms.
- Cycle 4 done (polish, design-neutral): dead snakes burst into crumb particles (replaces the 2 s grey
  blink), screen shake (my death 14 px, my bonk 7; nearby deaths and shrink cut after the attack), gold sparkle
  on every coin eaten, "+1" pop on my coins, "+1 bonk" pop + "much bonk! wow" banner + new procedural
  "bonk" sound on my kills, camera holds 1.2 s on my crumbs before following the leader, my own name
  label removed (it covered my pops; the ring marks me).
- Cycle 5 done (Arthur's picks: free steering, last snake alive, boop): grid replaced by the
  free-steering core (rules reference below), bots, mouse/touch/keys, renderer (round strokes + slice
  lines, doge head, heads glide between ticks, red closing wall, coin sprite = golden crumb). Juice
  ported: death burst along the body, my-death/my-bonk shake + banners, sparkle on eaten crumbs, "+5"
  pop on my golden crumbs only, "boop!" pop (+ new boop sound and 4 px shake when mine), "much wall!
  wow" + warning sound when the wall starts closing. Deleted: grid rules, speed-up, shrink steps,
  queued-turn arrow, speedUp + shrinkImpact sounds, "+1" pops on plain crumbs. typecheck, 7 vitest
  tests, build pass (16.1 kB JS, 7.4 kB gzip). Replay .playwright-mcp/steer-replay.js
  (browser_run_code_unsafe with filename) at 390x844 with the touch stick: ate, died on the wall;
  then watched to the end (bot won at 1:31, results panel), no console errors.
  Open (polish): snakes read small on a phone (~10 px wide) - Arthur's playtest judges.
- Cycle 6 done (boost, from the v3 build order): hold click / Space / a second finger / double-tap and
  hold. Speed and turn x2 while len > 20; burns max(4, 2% len)/s; each boosting tick drops a 1-crumb
  at its tail with chance burn/2 (first version dropped one per even length crossed, which over-drops
  when the snake eats while boosting); the owner cannot eat its own drops. Bots never boost.
  Boosting glow (pulsing white halo), gold stick ring, zoom sound. Wire: moved + boost 0/1, spawned +
  owner, client {t:"boost", on}. typecheck (both tsconfigs), 9 vitest tests (soak toggles boost on 3
  snakes), build pass (17.0 kB JS, 7.7 kB gzip). Headless replay at 390x844: double-tap-hold boost
  for 20 ticks moved 15.9 u (2x), lifting stops it, no console errors.
- Cycle 7 done (cut-off tagging, the rest of build item 6): a bonk is "cut" when the killer is
  boosting and its head was at the touched slice <= 1 s ago (trail walk; any boosting snake under
  ~len 64 is younger than 1 s end to end). Kill credit tie-break is now the touched slice nearest a
  head (was the last one found). Feed "X cut off by Y! wow", my banner "such cut! wow". The room logs
  `death <victim> <how> <killer> <s>` and `over <winner> <s>` (* = human; dev AI players count as
  human) to the dev server output: read it after Arthur's rounds for the cut-off share. 10 tests,
  typecheck, replay pass; log lines seen in the dev log.
- Cycle 8 done (victory moment, polish): at the end the camera moves in on the winner (x1.6 zoom;
  whole arena only after a draw), crown emoji on its head, confetti (gold, white, its color), banner
  "X wins! wow" / "you win! wow" / "such draw! wow", a C-E-G-C "win" sound when I win (else gameOver),
  results panel fades in after 2.5 s (CSS class "late", visibility hidden so it takes no taps).
  Replay .playwright-mcp/end-replay.js (die at once, watch the bots, ~100 s): banner, panel hidden at
  0.6 s and shown at 3.2 s, Play again works, no console errors. Confetti retuned after the capture
  (bigger, longer, winner's color): re-captured in cycle 9, reads as confetti.
- Cycle 9 done (results panel, UI): after a round the panel shows every snake best first (place,
  name in its color, crown, "(you)", bonks, time alive; my row gold-tinted), then New name / Play
  again, then the room code, invite and players. Deleted "You placed #N of M." (the table shows it).
  end-replay.js capture at 390x844 with 1 human: fits without scrolling, no console errors.

## Measured (bot-only, 100 seeds, free steering, 2026-10-05; grid numbers in git at 400a8da)
- Round length p10 75 s, median 95 s, p90 108 s, max 111 s (target 60-120 s, max 2:30). Deaths: 96%
  bonks, 4% wall; 10% of deaths by 18 s, median death 52 s. ~10 boops/round, 0 draws, longest 291 len.

## Design proposal v1 (specialist pass 2026-10-05; Arthur picked, see "Arthur's picks")
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

### Arthur's picks 2026-10-05 and v3 = v2 adapted to them
- Picks (AskUserQuestion): movement = free steering (grid replaced; test version first: one room of
  6-8 snakes, one bot type, a lag setting); round = last snake alive (no respawn, the arena shrinks,
  dead players watch; he rejected 3:00 rounds with respawn); head-on = boop (both bounce, lose a few
  slices, nobody dies).
- v3 deltas: no respawn, ghost, crown, auto rematch or win tally; the host starts the next round;
  bots can win. Spawn ring at 0.6 r0 heading clockwise. Wall holds 20 s, closes to 8 u by 1:50, to 0
  by 2:20; a simultaneous last death is a draw. All length drops on death (the "above 40" clause
  existed only for respawn). Self-overlap burn dropped: the closing wall is the anti-turtle. Crumbs
  outside the wall vanish; no spawning once it closes. Min 8 snakes, max 10 humans, r0 =
  75*sqrt(n/12). Metrics: cut-off share; deaths fair at 300 ms; round median 60-120 s, max 2:30;
  friends start a 3rd round.
- Build order: 5 core (done) -> 6 boost (2% len/s, min 4/s, half dropped, owner can't eat own drops,
  none below 20) + cut-off tagging -> 7 own-snake prediction, others projected, lag 0/150/300 ->
  8 tuning from Arthur's playtest.

## Rules reference (free steering, src/game/rules.ts; the grid rules are in git at 400a8da)
- u = new-snake width, +y up, 20 Hz ticks. Speed 8 u/s; turn 240 deg/s / width toward the target
  heading. Body = trail of head positions (1 per tick, 0.01 u), resampled every 0.5 u into
  ceil(len/4) slices; width 1 + (len-40)/400, cap 2.5. Start len 40, min 20.
- Contacts on post-move bodies, contact = (wa+wb)/2*0.8: head on another snake's slice >= 3 -> death,
  owner gets the kill + len/4 (tagged "cut" when it is boosting and its head was there <= 1 s ago);
  slices 0-2 -> boop: heads pushed apart, faced apart, both -8 len.
  Own body harmless. |head| + w/2*0.8 > wall -> death. Death drops floor(len/2/5) golden crumbs
  (5 len) along the body; the rest is lost.
- Crumbs: 400 per pi*75^2 kept full until 20 s; eaten within w/2 + 2 u of the head (lower id wins a
  tie); removed outside the wall.
- Arena: n = max(8, humans), r0 = 75*sqrt(n/12). Over when <= 1 alive (0 = draw). Ranking: alive,
  later death, kills, len, id.
- Bots (and dev AI players): every 6 ticks, best of 16 headings within +-162 deg: clear distance
  (1 u probes to 10 u) x10 + toward the nearest plain crumb x4 + keep x2 + center x6*edge^2; +-10 deg
  error.
- Boost: while held and len > 20, speed and turn x2, burn max(4, 2% len)/s (floor 20); each tick a
  1-crumb drops at the tail with chance burn/2, owned by the booster, who cannot eat it. Bots never
  boost.
- Wire: step {tick, timeMs, wall, moved [id,x,y,a,ta,len,boost 0|1], deaths [id,killer,how], boops
  [a,b], eaten [crumb,id], spawned [crumb,x,y,v,owner]}; client {t:"steer", a} <= 10/s, first change
  at once, skip < 0.03 rad; {t:"boost", on} at once on change.
- Client: camera on my head (1.2 s on my crumbs after death, then the longest; the winner at x1.6 at
  the end, the whole arena after a draw); zoom
  sqrt(screen px^2/3500)/(1 + 0.3(w-1)); heads glide along their heading between ticks. Touch stick
  (12 px dead zone, anchor follows past 60 px, outranks the mouse); mouse = angle from screen center;
  arrows/WASD = 8 directions on keydown. Boost: mouse button, Space, a second finger, or a double tap
  (< 300 ms after a lift) held; window blur releases everything.
- Text: "such 3 / very 2 / much 1 / WOW GO!", "X bonked by Y! wow", "X cut off by Y! wow",
  "X squished! such wall!", "wow. such ded.", "much bonk! wow", "such cut! wow", "much wall! wow",
  "boop!", "X wins! wow", "you win! wow", "such draw! wow".

## Decisions
- 2026-10-05 Stage plan, stack, public arena instead of queue, generated names, L1 for paid rooms:
  see VISION.md. Own-snake prediction is planned for cycle 7 (was "no prediction" for the grid).
- 2026-10-05 Arthur's picks: free steering, last snake alive, boop (see "Arthur's picks").
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
- 2026-10-05 The Playwright MCP browser gets real OS mouse moves (trusted "mouse" pointermoves) that
  steered against the replay's touch input (and probably boosted once). The replay now launches its own
  headless Chrome (launch({headless: true, channel: "chrome"}); the default headless shell is not
  installed). Before blaming input code, log pointerType + isTrusted.
