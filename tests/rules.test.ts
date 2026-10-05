import { describe, expect, it } from "vitest";
import {
  boost, createMatch, GOLD, head, MAX_TICKS, ranking, SEATS, slicesOf, START_LEN, steer, step, WALL_HOLD_MS,
  type Match,
} from "../src/game/rules";
import type { PublicMatch, ServerMsg } from "../src/game/protocol";
import { apply, newReplica } from "../src/game/replica";

const SOAK_SEEDS = 30;
const CHECK_EVERY = 25; // ticks between full client/server comparisons

// what a client can see, in a stable form
const view = (m: PublicMatch) => JSON.stringify({
  tick: m.tick, timeMs: m.timeMs, wall: m.wall, over: m.over, winner: m.winner,
  crumbs: [...m.crumbs].sort((a, b) => a.id - b.id),
  snakes: m.snakes.map((s) => [s.id, s.trail, s.a, s.ta, s.boost, s.len, s.alive, s.kills, s.diedTick]),
});

// the same path as the room: JSON over the wire, then the replica
const wire = (msg: ServerMsg) => JSON.parse(JSON.stringify(msg)) as ServerMsg;

describe("bot-only matches", () => {
  it(`end in time with one winner at most, and a client rebuilt from step messages matches the server (${SOAK_SEEDS} seeds)`, () => {
    for (let seed = 1; seed <= SOAK_SEEDS; seed++) {
      const m = createMatch(seed, []);
      const client = newReplica();
      const { rng: _, nextCrumb: __, ...snap } = m;
      apply(client, wire({ t: "snap", m: snap, countdownMs: 0 }));
      while (!m.over) {
        if (m.tick >= MAX_TICKS) expect.fail(`seed ${seed}: no winner after ${MAX_TICKS} ticks`);
        // bots never boost, so three of them press and release it the way players might
        for (let id = 0; id < 3; id++) if ((m.tick + 7 * id) % 30 === 0) boost(m, id, (m.tick * 31 + seed + id) % 3 === 0);
        const r = step(m);
        apply(client, wire({ t: "step", tick: m.tick, timeMs: m.timeMs, wall: m.wall, ...r }));
        if (m.over) apply(client, wire({ t: "over", winner: m.winner, rank: ranking(m) }));
        if ((m.tick % CHECK_EVERY === 0 || m.over) && view(client.m!) !== view(m)) {
          expect(view(client.m!), `seed ${seed} tick ${m.tick}`).toBe(view(m));
        }
      }
      const alive = m.snakes.filter((s) => s.alive);
      expect(alive.length, `seed ${seed}`).toBeLessThanOrEqual(1);
      expect(m.winner, `seed ${seed}`).toBe(alive[0]?.id ?? -1);
    }
  });
});

/** Human seats (no bot brain) where only the given snakes are alive, straight, with no food around. */
function arena(...specs: [x: number, y: number, a: number, len?: number][]): Match {
  const m = createMatch(1, Array.from({ length: SEATS }, (_, i) => `P${i}`));
  m.timeMs = WALL_HOLD_MS; // the wall is about to close, so no food spawns
  m.crumbs = [];
  m.snakes.forEach((s, i) => {
    s.alive = i < specs.length;
    if (!s.alive) return;
    const [x, y, a, len = START_LEN] = specs[i];
    Object.assign(s, { a, ta: a, len, trail: [] });
    for (let k = 80; k >= 0; k--) s.trail.push(x - Math.cos(a) * 0.4 * k, y - Math.sin(a) * 0.4 * k);
  });
  return m;
}

describe("contacts", () => {
  it("a head running into a body dies there: the owner gets the bonk and a quarter of the length, the rest bursts into gold", () => {
    const m = arena([7.5, 1.1, -Math.PI / 2], [10, 0, 0]);
    const r = step(m);
    expect(r.deaths).toEqual([[0, 1, "bonk"]]);
    expect([m.snakes[1].kills, m.snakes[1].len]).toEqual([1, START_LEN * 1.25]);
    expect(r.spawned.map(([, , , v]) => v)).toEqual(Array(START_LEN / 2 / GOLD).fill(GOLD));
    const body = slicesOf(m.snakes[0].trail, m.snakes[0].len);
    for (const [, x, y] of r.spawned) {
      let d = Infinity;
      for (let i = 0; i < body.length; i += 2) d = Math.min(d, Math.hypot(x - body[i], y - body[i + 1]));
      expect(d).toBeLessThan(0.75);
    }
    expect([m.over, m.winner]).toEqual([true, 1]);
  });

  it("heads that meet boop: both live, both pay length, they face apart and do not boop again", () => {
    const m = arena([0, 0, 0], [1.5, 0, Math.PI]);
    const r = step(m);
    expect([r.deaths, r.boops]).toEqual([[], [[0, 1]]]);
    expect(m.snakes.slice(0, 2).map((s) => s.len)).toEqual([START_LEN - 8, START_LEN - 8]);
    expect(Math.cos(m.snakes[0].a)).toBeLessThan(-0.99);
    expect(Math.cos(m.snakes[1].a)).toBeGreaterThan(0.99);
    expect(step(m).boops).toEqual([]);
    expect(m.over).toBe(false);
  });

  it("the wall kills a head that reaches it", () => {
    const m = arena([68, 0, 0], [0, 0, Math.PI / 2]);
    expect(step(m).deaths).toEqual([[0, -1, "wall"]]);
    expect([m.over, m.winner]).toEqual([true, 1]);
  });

  it("a snake's own body is harmless, even circling over it", () => {
    const m = arena([0, 0, 0, 160], [-30, -30, 0]);
    for (let i = 0; i < 60; i++) {
      steer(m, 0, m.snakes[0].a + Math.PI / 2);
      const r = step(m);
      expect([r.deaths, r.boops], `tick ${m.tick}`).toEqual([[], []]);
    }
    expect(m.snakes[0].len).toBe(160);
  });

  it("a crumb within reach of the head is eaten and grows the snake", () => {
    const m = arena([0, 0, 0], [-30, -30, 0]);
    m.crumbs = [{ id: 7, x: 2.8, y: 0, v: 1, owner: -1 }];
    expect(step(m).eaten).toEqual([[7, 0]]);
    expect(m.snakes[0].len).toBe(START_LEN + 1);
  });
});

describe("boost", () => {
  it("doubles the speed and burns length to the minimum; about half of it drops behind, for the others only", () => {
    const m = arena([0, 0, 0], [-30, -30, 0]);
    boost(m, 0, true);
    for (let i = 0; i < 20; i++) step(m);
    expect(head(m.snakes[0])[0]).toBeCloseTo(16, 1);
    expect(m.snakes[0].len).toBe(START_LEN - 4); // the 4 a second minimum, more than 2% of 40
    // circling over its own drops until the length runs out: it eats none of them
    for (let i = 0; i < 100; i++) {
      steer(m, 0, m.snakes[0].a + Math.PI / 2);
      step(m);
    }
    expect(m.snakes[0].len).toBe(20);
    const drops = m.crumbs.filter((c) => c.owner === 0 && c.v === 1).length;
    expect(drops).toBeGreaterThanOrEqual(5); // 20 burned: 10 on average
    expect(drops).toBeLessThanOrEqual(15);
    const [x, y] = head(m.snakes[0]);
    m.crumbs = [{ id: 7, x: x + 1, y, v: 1, owner: 0 }, { id: 8, x: x + 1.1, y, v: 1, owner: 1 }];
    expect(step(m).eaten).toEqual([[8, 0]]);
  });

  it("does nothing at the minimum length", () => {
    const m = arena([0, 0, 0, 20], [-30, -30, 0]);
    boost(m, 0, true);
    const r = step(m);
    expect([head(m.snakes[0])[0], m.snakes[0].len, r.spawned]).toEqual([0.4, 20, []]);
  });
});

describe("steering", () => {
  it("a snake turns toward its heading input gradually, then holds it", () => {
    const m = arena([0, 0, 0], [-30, -30, 0]);
    steer(m, 0, Math.PI / 2);
    step(m);
    expect(m.snakes[0].a).toBeGreaterThan(0);
    expect(m.snakes[0].a).toBeLessThan(Math.PI / 4);
    for (let i = 0; i < 20; i++) step(m);
    expect(m.snakes[0].a).toBe(m.snakes[0].ta);
  });
});
