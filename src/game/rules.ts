// Match rules, engine-free. The server runs them; clients share the geometry: each snake is a trail
// of head positions, one per tick, resampled into slices, so a body is drawn exactly where the
// server collides it. Units: u = a new snake's width; +y is up; angles in radians.

export const TICK_MS = 50;
export const SEATS = 10; // most humans in one match
const MIN_SNAKES = 8; // bots fill up to this
export const START_LEN = 40;
const MIN_LEN = 20;
export const SPEED = 8; // u/s
const TURN_RATE = (240 * Math.PI) / 180; // rad/s at width 1: a full circle about 4 widths across
const BOOST = 2; // speed and turn rate factor while boosting, so the turning circle stays the same
const BURN = 0.02; // length burned per second of boosting, as a share of length...
const BURN_MIN = 4; // ...and at least this; half of it drops behind as crumbs
const LEN_PER_SLICE = 4;
const SLICE_GAP = 0.5; // u between slices along the path
const MAGNET = 2; // u past the head's edge that crumbs are eaten from
const CRUMBS_PER_U2 = 400 / (Math.PI * 75 ** 2);
export const GOLD = 5; // length in one crumb dropped by a death
export const SKIN_COUNT = 5; // food skins, looks only; bots wear them in turn
const HIT = 0.8; // contact radius factor: grazes are forgiven
const BOOP_SLICES = 3; // a head touching another head or its first slices boops instead of dying
const BOOP_COST = 8; // length each snake loses in a boop
const CUT_TICKS = 1000 / TICK_MS; // a bonk is a cut-off when a boosting killer crossed the victim's path this recently
export const WALL_HOLD_MS = 20_000; // full arena; food keeps spawning
const WALL_CLOSE_MS = 90_000; // then the wall closes to WALL_MIN
const WALL_MIN = 8;
const WALL_END_MS = 30_000; // then to nothing, so every round ends
export const MAX_TICKS = (WALL_HOLD_MS + WALL_CLOSE_MS + WALL_END_MS) / TICK_MS + 1;
export const BOT_THINK_TICKS = 6; // a bot reacts every 300 ms
const LOOK = 10; // u a bot looks ahead
const STEP_U = (SPEED * TICK_MS) / 1000;
const CELL = 3; // u; bigger than any contact or bot look-around distance

export const width = (len: number) => Math.min(2.5, 1 + Math.max(0, len - START_LEN) / 400);
const sliceCount = (len: number) => Math.ceil(len / LEN_PER_SLICE);
export const wrap = (a: number) => a - 2 * Math.PI * Math.floor((a + Math.PI) / (2 * Math.PI));
const q = (v: number, k: number) => Math.round(v * k) / k; // wire precision, kept in the state too

/** The wall's radius at a time in the round. */
function wallRadius(r0: number, ms: number) {
  const t = ms - WALL_HOLD_MS;
  if (t <= 0) return r0;
  if (t <= WALL_CLOSE_MS) return r0 + ((WALL_MIN - r0) * t) / WALL_CLOSE_MS;
  return Math.max(0, WALL_MIN * (1 - (t - WALL_CLOSE_MS) / WALL_END_MS));
}

export type Death = "bonk" | "cut" | "wall";

export interface Snake {
  id: number;
  name: string;
  human: boolean;
  trail: number[]; // head positions oldest first: x0, y0, x1, y1, ...; the last pair is the head
  a: number; // heading
  ta: number; // target heading: the player's input
  boost: boolean; // the player's input; no effect at MIN_LEN or below
  len: number;
  alive: boolean;
  kills: number;
  diedTick: number; // -1 while alive
  skin: number; // looks only: a food skin index
}

export interface Crumb {
  id: number;
  x: number;
  y: number;
  v: number; // length it gives: 1, or GOLD when dropped by a death
  owner: number; // the snake whose boost dropped it, which cannot eat it; -1 = anyone
}

export interface Match {
  tick: number;
  timeMs: number;
  r0: number; // the arena's radius before the wall moves
  wall: number;
  snakes: Snake[];
  crumbs: Crumb[];
  nextCrumb: number;
  rng: number; // server-only: clients must not predict food
  over: boolean;
  winner: number; // snake id, -1 = nobody
}

export interface StepResult {
  moved: [id: number, x: number, y: number, a: number, ta: number, len: number, boost: 0 | 1][];
  deaths: [id: number, killer: number, how: Death][];
  boops: [a: number, b: number][];
  eaten: [crumb: number, id: number][];
  spawned: [crumb: number, x: number, y: number, v: number, owner: number][];
}

const BOT_NAMES = ["Cheems", "Bonk", "Shibe", "Floof", "Bork", "Snoot", "Woofer", "Pupper", "Doggo", "Smol"];

// mulberry32: 32-bit state, integer math only
function rand(m: Match): number {
  let t = (m.rng = (m.rng + 0x6d2b79f5) | 0);
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return ((t ^ (t >>> 14)) >>> 0) / 2 ** 32;
}

export const head = (s: Pick<Snake, "trail">): [number, number] => [s.trail[s.trail.length - 2], s.trail[s.trail.length - 1]];
export const boosting = (s: Pick<Snake, "boost" | "len">) => s.boost && s.len > MIN_LEN;
export const speedOf = (s: Pick<Snake, "boost" | "len">) => (boosting(s) ? SPEED * BOOST : SPEED);

/**
 * Slice centers, head first, every SLICE_GAP along the trail: [x0, y0, x1, y1, ...]. A head given
 * ahead of the last trail point is used instead of it (the client draws between ticks).
 */
export function slicesOf(t: number[], len: number, out: number[] = [], hx?: number, hy?: number): number[] {
  const n = sliceCount(len);
  let i = t.length - 2;
  let px = t[i];
  let py = t[i + 1];
  if (hx !== undefined && hy !== undefined) {
    i = t.length;
    px = hx;
    py = hy;
  }
  out.length = 0;
  out.push(px, py);
  let want = SLICE_GAP;
  while (out.length < n * 2 && i >= 2) {
    const qx = t[i - 2];
    const qy = t[i - 1];
    const seg = Math.hypot(qx - px, qy - py);
    if (seg >= want) {
      px += ((qx - px) * want) / seg;
      py += ((qy - py) * want) / seg;
      out.push(px, py);
      want = SLICE_GAP;
    } else {
      want -= seg;
      px = qx;
      py = qy;
      i -= 2;
    }
  }
  return out;
}

/** Drops the trail points the body no longer reaches. */
export function trimTrail(t: number[], len: number) {
  const need = sliceCount(len) * SLICE_GAP + 1;
  let d = 0;
  for (let i = t.length - 4; i > 0; i -= 2) {
    d += Math.hypot(t[i + 2] - t[i], t[i + 3] - t[i + 1]);
    if (d > need) return void t.splice(0, i);
  }
}

/** Every live snake's slices, bucketed by cell for nearby queries. */
export interface Bodies {
  slices: number[][]; // by snake id, head first; empty when dead
  widths: number[];
  cells: Map<number, number[]>; // cell -> snake id, slice index, ...
}

const cellKey = (i: number, j: number) => (i + 1024) * 2048 + j + 1024;

export function bodiesOf(snakes: readonly Snake[]): Bodies {
  const b: Bodies = { slices: [], widths: [], cells: new Map() };
  for (const s of snakes) {
    const sl = s.alive ? slicesOf(s.trail, s.len) : [];
    b.slices[s.id] = sl;
    b.widths[s.id] = width(s.len);
    for (let k = 0; k < sl.length; k += 2) {
      const key = cellKey(Math.floor(sl[k] / CELL), Math.floor(sl[k + 1] / CELL));
      const list = b.cells.get(key);
      if (list) list.push(s.id, k / 2);
      else b.cells.set(key, [s.id, k / 2]);
    }
  }
  return b;
}

/** Calls f for each slice in the cells around a point (everything within CELL of it, and more). */
function near(b: Bodies, x: number, y: number, f: (id: number, k: number, sx: number, sy: number) => void) {
  const ci = Math.floor(x / CELL);
  const cj = Math.floor(y / CELL);
  for (let i = ci - 1; i <= ci + 1; i++) {
    for (let j = cj - 1; j <= cj + 1; j++) {
      const list = b.cells.get(cellKey(i, j));
      if (!list) continue;
      for (let n = 0; n < list.length; n += 2) {
        const sl = b.slices[list[n]];
        const k = list[n + 1];
        f(list[n], k, sl[2 * k], sl[2 * k + 1]);
      }
    }
  }
}

/** Humans take the first seats in order; bots fill up to MIN_SNAKES. The arena grows with the count. */
export function createMatch(seed: number, humans: string[], skins: number[] = []): Match {
  const n = Math.max(MIN_SNAKES, humans.length);
  const r0 = q(75 * Math.sqrt(n / 12), 100);
  const snakes = Array.from({ length: n }, (_, id): Snake => {
    // evenly on a ring, heading clockwise, so nobody starts nose to nose
    const t = (2 * Math.PI * id) / n;
    const a = q(wrap(t - Math.PI / 2), 1000);
    const x = Math.cos(t) * r0 * 0.6;
    const y = Math.sin(t) * r0 * 0.6;
    const trail: number[] = [];
    for (let i = Math.ceil((sliceCount(START_LEN) * SLICE_GAP) / STEP_U) + 1; i >= 0; i--) {
      trail.push(q(x - Math.cos(a) * STEP_U * i, 100), q(y - Math.sin(a) * STEP_U * i, 100));
    }
    const human = id < humans.length;
    return {
      id, name: human ? humans[id] : BOT_NAMES[id % BOT_NAMES.length], human, trail, a, ta: a, boost: false,
      len: START_LEN, alive: true, kills: 0, diedTick: -1, skin: skins[id] ?? ((seed >>> 0) + id) % SKIN_COUNT,
    };
  });
  const m: Match = {
    tick: 0, timeMs: 0, r0, wall: r0, snakes, crumbs: [], nextCrumb: 0, rng: seed | 0, over: false, winner: -1,
  };
  refill(m);
  return m;
}

/** A player's input: the heading their snake turns toward. */
export function steer(m: Match, id: number, a: number) {
  const s = m.snakes[id];
  if (s?.alive && Number.isFinite(a)) s.ta = q(wrap(a), 1000);
}

/** A player's input: boost on or off. */
export function boost(m: Match, id: number, on: boolean) {
  const s = m.snakes[id];
  if (s?.alive) s.boost = on;
}

/**
 * Bot brain, also used by the test AI players: of 16 headings, the one with the most clear space
 * ahead, then toward the nearest plain crumb, then the least turning; near the wall, back inward.
 */
export function botAngle(m: Pick<Match, "wall" | "crumbs">, b: Bodies, me: Snake, random: () => number): number {
  const [hx, hy] = head(me);
  const w = width(me.len);
  let food = me.a;
  let nearest = Infinity;
  for (const c of m.crumbs) {
    const d = (c.x - hx) ** 2 + (c.y - hy) ** 2;
    if (c.v === 1 && d < nearest) { // golden crumbs are left to the players
      nearest = d;
      food = Math.atan2(c.y - hy, c.x - hx);
    }
  }
  const center = Math.atan2(-hy, -hx);
  const edge = Math.hypot(hx, hy) / Math.max(1, m.wall);
  let pick = me.a;
  let top = -Infinity;
  for (let k = 0; k < 16; k++) {
    const a = me.a + ((k - 7.5) / 8) * 0.9 * Math.PI;
    let clear = LOOK;
    for (let d = 1; d <= LOOK && clear === LOOK; d++) {
      const px = hx + Math.cos(a) * d;
      const py = hy + Math.sin(a) * d;
      if (Math.hypot(px, py) + w > m.wall) clear = d - 1;
      else near(b, px, py, (id, _k, sx, sy) => {
        const reach = (w + b.widths[id]) / 2 + 0.5;
        if (id !== me.id && (sx - px) ** 2 + (sy - py) ** 2 < reach * reach) clear = d - 1;
      });
    }
    const score = clear * 10 + Math.cos(a - food) * 4 + Math.cos(a - me.a) * 2 + Math.cos(a - center) * edge * edge * 6;
    if (score > top) {
      top = score;
      pick = a;
    }
  }
  return wrap(pick + (random() - 0.5) * (Math.PI / 9)); // aim error, +-10 degrees
}

/** Advance one tick. Mutates the match and returns what changed, which is what clients receive. */
export function step(m: Match): StepResult {
  const r: StepResult = { moved: [], deaths: [], boops: [], eaten: [], spawned: [] };
  if (m.over) return r;
  m.tick++;
  m.timeMs += TICK_MS;
  m.wall = q(wallRadius(m.r0, m.timeMs), 100);
  const movers = m.snakes.filter((s) => s.alive);

  // bots decide on the pre-move state, each one every BOT_THINK_TICKS
  const thinkers = movers.filter((s) => !s.human && (m.tick + s.id) % BOT_THINK_TICKS === 0);
  if (thinkers.length) {
    const before = bodiesOf(m.snakes);
    for (const s of thinkers) s.ta = q(botAngle(m, before, s, () => rand(m)), 1000);
  }

  const dt = TICK_MS / 1000;
  const droppers: number[] = []; // boosting snakes that drop a crumb this tick
  for (const s of movers) {
    const k = boosting(s) ? BOOST : 1;
    const turn = ((TURN_RATE * k) / width(s.len)) * dt;
    s.a = q(wrap(s.a + Math.max(-turn, Math.min(turn, wrap(s.ta - s.a)))), 1000);
    const [x, y] = head(s);
    s.trail.push(q(x + Math.cos(s.a) * SPEED * k * dt, 100), q(y + Math.sin(s.a) * SPEED * k * dt, 100));
    if (k === 1) continue;
    // crumbs of 1 at a chance of half the burn: on average, half of the cost lies behind for the others
    const before = s.len;
    s.len = q(Math.max(MIN_LEN, s.len - Math.max(BURN_MIN, BURN * s.len) * dt), 10);
    if (rand(m) < (before - s.len) / 2) droppers.push(s.id);
  }

  // contacts on the post-move bodies, all at once: a head on another body dies, heads that meet boop
  const b = bodiesOf(m.snakes);
  const drops: [x: number, y: number, v: number, owner: number][] = [];
  for (const id of droppers) {
    const sl = b.slices[id];
    drops.push([sl[sl.length - 2] + (rand(m) - 0.5) * 0.5, sl[sl.length - 1] + (rand(m) - 0.5) * 0.5, 1, id]);
  }
  const dead = new Map<number, [killer: number, how: Death]>();
  const boops = new Map<number, [number, number]>();
  for (const s of movers) {
    const [x, y] = head(s);
    const w = b.widths[s.id];
    if (Math.hypot(x, y) + (w / 2) * HIT > m.wall) {
      dead.set(s.id, [-1, "wall"]);
      continue;
    }
    let killer = -1;
    let hit = Infinity; // the killer's slice that was touched, nearest to its head
    let booped = -1;
    near(b, x, y, (id, k, sx, sy) => {
      if (id === s.id) return; // your own body is harmless
      const reach = ((w + b.widths[id]) / 2) * HIT;
      if ((sx - x) ** 2 + (sy - y) ** 2 >= reach * reach) return;
      if (k < BOOP_SLICES) booped = id;
      else if (k < hit) [killer, hit] = [id, k];
    });
    if (killer >= 0) dead.set(s.id, [killer, cutOff(m.snakes[killer], hit) ? "cut" : "bonk"]);
    else if (booped >= 0) {
      const pair: [number, number] = [Math.min(s.id, booped), Math.max(s.id, booped)];
      boops.set(pair[0] * 64 + pair[1], pair);
    }
  }

  for (const [id, [killer, how]] of dead) {
    const s = m.snakes[id];
    s.alive = false;
    s.diedTick = m.tick;
    r.deaths.push([id, killer, how]);
    if (killer >= 0) {
      m.snakes[killer].kills++;
      m.snakes[killer].len = q(m.snakes[killer].len + s.len / 4, 10);
    }
    // half the length bursts into golden crumbs along the body; the last quarter is lost
    const sl = b.slices[id];
    const n = Math.floor(s.len / 2 / GOLD);
    for (let i = 0; i < n; i++) {
      const j = 2 * Math.floor((i * sl.length) / (2 * n));
      drops.push([sl[j] + rand(m) - 0.5, sl[j + 1] + rand(m) - 0.5, GOLD, -1]);
    }
  }

  for (const [i, j] of boops.values()) {
    const s = m.snakes[i];
    const o = m.snakes[j];
    if (!s.alive || !o.alive) continue;
    const [sx, sy] = head(s);
    const [ox, oy] = head(o);
    const d = Math.hypot(sx - ox, sy - oy);
    const a = d > 1e-6 ? Math.atan2(sy - oy, sx - ox) : wrap(s.a + Math.PI);
    // push the heads apart, turn them away from each other, and charge both
    const push = Math.max(0, ((b.widths[i] + b.widths[j]) / 2) * HIT - d) / 2 + 0.05;
    setHead(s, sx + Math.cos(a) * push, sy + Math.sin(a) * push);
    setHead(o, ox - Math.cos(a) * push, oy - Math.sin(a) * push);
    s.a = q(a, 1000);
    o.a = q(wrap(a + Math.PI), 1000);
    s.len = q(Math.max(MIN_LEN, s.len - BOOP_COST), 10);
    o.len = q(Math.max(MIN_LEN, o.len - BOOP_COST), 10);
    r.boops.push([i, j]);
  }
  endIfDecided(m);

  for (const s of m.snakes) {
    if (!s.alive) continue;
    const [x, y] = head(s);
    const reach = width(s.len) / 2 + MAGNET;
    m.crumbs = m.crumbs.filter((c) => {
      if (c.owner === s.id || (c.x - x) ** 2 + (c.y - y) ** 2 > reach * reach) return true;
      s.len = q(s.len + c.v, 10);
      r.eaten.push([c.id, s.id]);
      return false;
    });
  }
  for (const [x, y, v, owner] of drops) addCrumb(m, r, x, y, v, owner);
  refill(m, r);
  m.crumbs = m.crumbs.filter((c) => c.x ** 2 + c.y ** 2 <= m.wall ** 2);

  for (const s of movers) {
    const [x, y] = head(s);
    r.moved.push([s.id, x, y, s.a, s.ta, s.len, s.boost ? 1 : 0]);
    if (s.alive) trimTrail(s.trail, s.len);
  }
  return r;
}

function setHead(s: Snake, x: number, y: number) {
  s.trail[s.trail.length - 2] = q(x, 100);
  s.trail[s.trail.length - 1] = q(y, 100);
}

function addCrumb(m: Match, r: StepResult | undefined, x: number, y: number, v: number, owner = -1) {
  const c = { id: m.nextCrumb++, x: q(x, 100), y: q(y, 100), v, owner };
  m.crumbs.push(c);
  r?.spawned.push([c.id, c.x, c.y, v, owner]);
}

/** Food spawns until the wall starts closing; after that only deaths feed the arena. */
function refill(m: Match, r?: StepResult) {
  if (m.timeMs >= WALL_HOLD_MS) return;
  const target = Math.round(CRUMBS_PER_U2 * Math.PI * m.r0 ** 2);
  while (m.crumbs.length < target) {
    const d = m.wall * 0.95 * Math.sqrt(rand(m));
    const t = 2 * Math.PI * rand(m);
    addCrumb(m, r, Math.cos(t) * d, Math.sin(t) * d, 1);
  }
}

function endIfDecided(m: Match) {
  if (m.over) return;
  const alive = m.snakes.filter((s) => s.alive);
  if (alive.length > 1) return;
  m.over = true;
  m.winner = alive[0]?.id ?? -1;
}

/** Is the killer boosting, with its head where its slice k is now less than CUT_TICKS ago? */
function cutOff(s: Snake, k: number) {
  const t = s.trail; // one head position per tick, newest last
  let d = k * SLICE_GAP;
  for (let i = t.length - 2; d > 0 && i >= 2 && i > t.length - 2 - 2 * CUT_TICKS; i -= 2) {
    d -= Math.hypot(t[i] - t[i - 2], t[i + 1] - t[i - 1]);
  }
  return boosting(s) && d <= 0;
}

/** Best first: alive, then died later, then bonks, then length. */
export function ranking(m: Pick<Match, "snakes">): number[] {
  const last = (s: Snake) => (s.alive ? Number.MAX_SAFE_INTEGER : s.diedTick);
  return [...m.snakes]
    .sort((a, b) => last(b) - last(a) || b.kills - a.kills || b.len - a.len || a.id - b.id)
    .map((s) => s.id);
}
