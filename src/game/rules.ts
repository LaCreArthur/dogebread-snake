// Match rules, engine-free. The server runs them; the client imports only constants and types.
// Numbers and tie-breaks are ported from the Rust game (shared/src/game.rs, client/src/game_systems.rs,
// client/src/input.rs) so bots and collisions behave the same.

export const W = 60;
export const H = 60;
export const SEATS = 10;
export const FOOD = 35;
export const START_LEN = 5;
export const STEP_MS = 125;
export const MIN_STEP_MS = 60;
export const SPEEDUP_EVERY_MS = 20_000;
export const SHRINK_EVERY_MS = 12_000;
export const SHRINK_WARN_MS = 2_000;
export const MIN_ARENA = 6;

/** 0 up (+y), 1 right, 2 down (-y), 3 left. +y is up, as in the Rust game; the client flips it. */
export type Dir = 0 | 1 | 2 | 3;
export const DX = [0, 1, 0, -1] as const;
export const DY = [1, 0, -1, 0] as const;
export const opposite = (d: Dir) => ((d + 2) % 4) as Dir;

export const cell = (x: number, y: number) => y * W + x;
export const cx = (c: number) => c % W;
export const cy = (c: number) => Math.floor(c / W);

/** [minX, minY, maxX, maxY], max exclusive. */
export type Bounds = [number, number, number, number];
export const inside = (b: Bounds, c: number) => cx(c) >= b[0] && cx(c) < b[2] && cy(c) >= b[1] && cy(c) < b[3];
export const canShrink = (b: Bounds) => b[2] - b[0] > MIN_ARENA && b[3] - b[1] > MIN_ARENA;
/** True during the warning before the next shrink. */
export const shrinkWarning = (m: { timeMs: number; bounds: Bounds }) =>
  canShrink(m.bounds) && SHRINK_EVERY_MS - (m.timeMs % SHRINK_EVERY_MS) <= SHRINK_WARN_MS;

export type Death = "bonk" | "ded" | "crushed";

export interface Snake {
  id: number;
  name: string;
  human: boolean;
  body: number[]; // cells, head first
  dir: Dir;
  next: Dir;
  alive: boolean;
  grow: number;
  score: number;
  kills: number;
  diedTick: number; // -1 while alive
}

export interface Match {
  tick: number;
  timeMs: number;
  stepMs: number;
  bounds: Bounds;
  snakes: Snake[];
  food: number[];
  rng: number; // server-only: clients must not predict food
  over: boolean;
  winner: number; // snake id, -1 = nobody
}

export interface StepResult {
  moved: [id: number, head: number, len: number][];
  deaths: [id: number, killer: number, how: Death][];
  eaten: [id: number, cell: number][];
  spawned: number[];
  shrunk: boolean;
  spedUp: boolean;
}

const SPAWNS: [number, number, Dir][] = [
  [10, 10, 1], [49, 10, 0], [10, 49, 1], [49, 49, 2], [30, 10, 0],
  [10, 30, 1], [49, 30, 3], [30, 49, 2], [20, 20, 1], [40, 40, 3],
];

export const BOT_NAMES = ["Cheems", "Bonk", "Shibe", "Floof", "Bork", "Snoot", "Woofer", "Pupper", "Doggo", "Smol"];

// mulberry32: 32-bit state, integer math only
function rand(m: Match): number {
  let t = (m.rng = (m.rng + 0x6d2b79f5) | 0);
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return (t ^ (t >>> 14)) >>> 0;
}
const range = (m: Match, min: number, max: number) => min + (rand(m) % (max - min));

/** Humans take the first seats in order; bots fill the rest. */
export function createMatch(seed: number, humans: string[]): Match {
  const snakes: Snake[] = SPAWNS.slice(0, SEATS).map(([x, y, dir], id) => {
    const body: number[] = [];
    for (let i = 0; i < START_LEN; i++) body.push(cell(x - DX[dir] * i, y - DY[dir] * i));
    const human = id < humans.length;
    return {
      id, name: human ? humans[id] : BOT_NAMES[id], human, body, dir, next: dir,
      alive: true, grow: 0, score: 0, kills: 0, diedTick: -1,
    };
  });
  const m: Match = {
    tick: 0, timeMs: 0, stepMs: STEP_MS, bounds: [1, 1, W - 1, H - 1], snakes, food: [],
    rng: seed | 0, over: false, winner: -1,
  };
  for (let i = 0; i < FOOD; i++) m.food.push(cell(range(m, 2, W - 2), range(m, 2, H - 2)));
  return m;
}

/** Queue a turn; a 180 against the current heading is ignored. */
export function turn(m: Match, id: number, d: Dir) {
  const s = m.snakes[id];
  if (s?.alive && d !== opposite(s.dir)) s.next = d;
}

const BOT_ORDER: Dir[] = [0, 2, 3, 1]; // the Rust Direction::ALL order, which decides ties

/** Bot brain. Also drives the test AI players, which see the same state as any client. */
export function botDir(m: Pick<Match, "bounds" | "snakes" | "food">, me: Snake): Dir {
  const occupied = new Uint8Array(W * H);
  for (const s of m.snakes) if (s.alive) for (const c of s.body) occupied[c] = 1;
  const [x0, y0, x1, y1] = m.bounds;
  const free = (x: number, y: number) => x >= x0 && x < x1 && y >= y0 && y < y1 && !occupied[cell(x, y)];
  const personality = me.id % 3; // 0 hungry, 1 cautious, 2 aggressive
  const hx = cx(me.body[0]);
  const hy = cy(me.body[0]);
  let best = me.dir;
  let bestScore = -Infinity;
  for (const d of BOT_ORDER) {
    if (d === opposite(me.dir)) continue;
    const nx = hx + DX[d];
    const ny = hy + DY[d];
    if (!free(nx, ny)) continue;
    // 2-step look-ahead: the next cell needs at least one free exit
    let exit = false;
    for (const e of BOT_ORDER) if (e !== opposite(d) && free(nx + DX[e], ny + DY[e])) { exit = true; break; }
    if (!exit) continue;

    let score = 0;
    let nearest = Infinity;
    for (const f of m.food) nearest = Math.min(nearest, Math.abs(cx(f) - nx) + Math.abs(cy(f) - ny));
    if (nearest < Infinity) {
      const food = 100 - nearest;
      score += personality === 0 ? food * 2 : personality === 1 ? food : Math.trunc(food / 2);
    }
    const wall = Math.min(nx - x0, x1 - 1 - nx, ny - y0, y1 - 1 - ny);
    score += wall * (personality === 1 ? 5 : 2);
    if (personality !== 0) {
      for (const o of m.snakes) {
        if (o === me || !o.alive) continue;
        const dist = Math.abs(cx(o.body[0]) - nx) + Math.abs(cy(o.body[0]) - ny);
        if (personality === 2 && dist < 8) score += (8 - dist) * 4; // aggressive: cut heads off
        if (personality === 1 && dist < 6) score -= (6 - dist) * 3; // cautious: keep away
      }
    }
    if (d === me.dir) score += 3;
    if (score > bestScore) { bestScore = score; best = d; }
  }
  return best;
}

/** Advance one step. Mutates the match and returns what changed, which is what clients receive. */
export function step(m: Match): StepResult {
  const r: StepResult = { moved: [], deaths: [], eaten: [], spawned: [], shrunk: false, spedUp: false };
  if (m.over) return r;
  m.tick++;

  // bots all decide on the pre-move state
  const turns = m.snakes.filter((s) => s.alive && !s.human).map((s) => [s.id, botDir(m, s)] as const);
  for (const [id, d] of turns) turn(m, id, d);

  const movers = m.snakes.filter((s) => s.alive);
  for (const s of movers) {
    s.dir = s.next;
    const h = s.body[0];
    s.body.unshift(cell(cx(h) + DX[s.dir], cy(h) + DY[s.dir]));
    if (s.grow > 0) s.grow--;
    else s.body.pop();
    r.moved.push([s.id, s.body[0], s.body.length]);
  }

  // collisions against the post-move state; the first cause found for a snake wins
  const bodies = new Map<number, number>(); // cell -> owner, heads excluded
  for (const s of movers) for (let i = 1; i < s.body.length; i++) bodies.set(s.body[i], s.id);
  const dead = new Map<number, [number, Death]>();
  const kill = (id: number, killer: number, how: Death) => dead.has(id) || dead.set(id, [killer, how]);
  for (const s of movers) {
    const h = s.body[0];
    if (!inside(m.bounds, h)) { kill(s.id, -1, "ded"); continue; }
    const owner = bodies.get(h);
    if (owner === s.id) { kill(s.id, -1, "ded"); continue; }
    if (owner !== undefined) kill(s.id, owner, "bonk");
    for (const o of movers) if (o !== s && o.body[0] === h) { kill(s.id, -1, "ded"); kill(o.id, -1, "ded"); }
  }
  for (const [id, [killer, how]] of dead) {
    if (killer >= 0) m.snakes[killer].kills++;
    m.snakes[id].alive = false;
    m.snakes[id].diedTick = m.tick;
    r.deaths.push([id, killer, how]);
  }
  endIfDecided(m);

  const heads = new Map<number, Snake>();
  for (const s of m.snakes) if (s.alive) heads.set(s.body[0], s);
  m.food = m.food.filter((f) => {
    const s = heads.get(f);
    if (!s) return true;
    s.grow += 2;
    s.score++;
    r.eaten.push([s.id, f]);
    return false;
  });
  const [x0, y0, x1, y1] = m.bounds;
  while (m.food.length < FOOD) {
    const f = cell(range(m, x0 + 1, x1 - 1), range(m, y0 + 1, y1 - 1));
    m.food.push(f);
    r.spawned.push(f);
  }

  const before = m.timeMs;
  m.timeMs += m.stepMs;
  const crossed = (every: number) => Math.floor(m.timeMs / every) > Math.floor(before / every);
  if (crossed(SPEEDUP_EVERY_MS)) {
    m.stepMs = Math.max(MIN_STEP_MS, Math.round(m.stepMs * 0.85));
    r.spedUp = true;
  }
  if (crossed(SHRINK_EVERY_MS) && canShrink(m.bounds)) {
    m.bounds = [x0 + 1, y0 + 1, x1 - 1, y1 - 1];
    r.shrunk = true;
    for (const s of m.snakes) {
      if (!s.alive || inside(m.bounds, s.body[0])) continue;
      s.alive = false;
      s.diedTick = m.tick;
      r.deaths.push([s.id, -1, "crushed"]);
    }
    m.food = m.food.filter((f) => inside(m.bounds, f));
    endIfDecided(m);
  }
  return r;
}

function endIfDecided(m: Match) {
  if (m.over) return;
  const alive = m.snakes.filter((s) => s.alive);
  if (alive.length > 1) return;
  m.over = true;
  m.winner = alive[0]?.id ?? -1;
}

/** Best first: alive, then died later, then kills, then score. */
export function ranking(m: Pick<Match, "snakes">): number[] {
  const last = (s: Snake) => (s.alive ? Number.MAX_SAFE_INTEGER : s.diedTick);
  return [...m.snakes]
    .sort((a, b) => last(b) - last(a) || b.kills - a.kills || b.score - a.score || a.id - b.id)
    .map((s) => s.id);
}
