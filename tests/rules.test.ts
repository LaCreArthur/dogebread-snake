import { describe, expect, it } from "vitest";
import {
  cell, createMatch, cx, cy, FOOD, inside, ranking, SEATS, SHRINK_EVERY_MS, step, turn,
  type Dir, type Match,
} from "../src/game/rules";
import type { PublicMatch, ServerMsg } from "../src/game/protocol";
import { apply, newReplica } from "../src/game/replica";

const SOAK_SEEDS = 200;
const MAX_STEPS = 10_000; // a full match is ~5,200 steps at most: 26 shrinks x 12 s, mostly at 60 ms

// what a client can see, in a stable form
const view = (m: PublicMatch) => JSON.stringify({
  tick: m.tick, timeMs: m.timeMs, stepMs: m.stepMs, bounds: m.bounds, over: m.over, winner: m.winner,
  food: [...m.food].sort((a, b) => a - b),
  snakes: m.snakes.map((s) => [s.id, s.body, s.alive, s.score, s.kills, s.dir, s.diedTick]),
});

// the same path as the room: JSON over the wire, then the replica
const wire = (msg: ServerMsg) => JSON.parse(JSON.stringify(msg)) as ServerMsg;

describe("bot-only matches", () => {
  it(`end, keep their invariants, and a client rebuilt from step messages matches the server (${SOAK_SEEDS} seeds)`, () => {
    for (let seed = 1; seed <= SOAK_SEEDS; seed++) {
      const m = createMatch(seed, []);
      const client = newReplica();
      const { rng: _, ...snap } = m;
      apply(client, wire({ t: "snap", m: snap, countdownMs: 0 }));
      let steps = 0;
      while (!m.over) {
        if (++steps > MAX_STEPS) expect.fail(`seed ${seed}: no winner after ${MAX_STEPS} steps`);
        const r = step(m);
        apply(client, wire({ t: "step", tick: m.tick, timeMs: m.timeMs, stepMs: m.stepMs, bounds: m.bounds, ...r }));
        if (m.over) apply(client, wire({ t: "over", winner: m.winner, rank: ranking(m) }));

        if (view(client.m!) !== view(m)) expect(view(client.m!), `seed ${seed} tick ${m.tick}`).toBe(view(m));
        const bad = invariantBreak(m);
        if (bad) expect.fail(`seed ${seed} tick ${m.tick}: ${bad}`);
      }
      const alive = m.snakes.filter((s) => s.alive);
      expect(alive.length, `seed ${seed}`).toBeLessThanOrEqual(1);
      expect(m.winner, `seed ${seed}`).toBe(alive[0]?.id ?? -1);
    }
  });
});

function invariantBreak(m: Match): string {
  if (m.food.length > FOOD) return `${m.food.length} coins`;
  const out = m.food.find((f) => !inside(m.bounds, f));
  if (out !== undefined) return `coin at ${cx(out)},${cy(out)} outside ${m.bounds}`;
  const taken = new Set<number>();
  for (const s of m.snakes) {
    if (!s.alive) continue;
    if (!inside(m.bounds, s.body[0])) return `snake ${s.id} alive outside the arena`;
    for (const c of s.body) {
      if (taken.has(c)) return `two live segments on ${cx(c)},${cy(c)}`;
      taken.add(c);
    }
  }
  return "";
}

/** A match of human seats (no bot brain) where only the given snakes are alive. */
function arena(...snakes: { body: [number, number][]; dir: Dir }[]) {
  const m = createMatch(1, Array.from({ length: SEATS }, (_, i) => `P${i}`));
  m.food = [];
  m.snakes.forEach((s, i) => {
    const spec = snakes[i];
    s.alive = !!spec;
    if (!spec) return;
    s.body = spec.body.map(([x, y]) => cell(x, y));
    s.dir = s.next = spec.dir;
  });
  return m;
}

describe("collisions", () => {
  it("head-on: both die and nobody gets the kill", () => {
    const m = arena(
      { body: [[20, 30], [19, 30], [18, 30]], dir: 1 },
      { body: [[22, 30], [23, 30], [24, 30]], dir: 3 },
    );
    expect(step(m).deaths).toEqual([[0, -1, "ded"], [1, -1, "ded"]]);
    expect(m.snakes.map((s) => s.kills)).toEqual(Array(SEATS).fill(0));
    expect([m.over, m.winner]).toEqual([true, -1]);
  });

  it("running into a body: the runner dies and the body's owner gets the kill", () => {
    const m = arena(
      { body: [[20, 30], [20, 29], [20, 28]], dir: 0 },
      { body: [[22, 31], [21, 31], [20, 31], [19, 31]], dir: 1 },
    );
    expect(step(m).deaths).toEqual([[0, 1, "bonk"]]);
    expect(m.snakes[1].kills).toBe(1);
    expect([m.over, m.winner]).toEqual([true, 1]);
  });

  it("the wall kills: the last floor cell is safe, the wall cell is not", () => {
    const m = arena(
      { body: [[58, 30], [57, 30], [56, 30]], dir: 1 },
      { body: [[57, 20], [56, 20], [55, 20]], dir: 1 },
    );
    expect(step(m).deaths).toEqual([[0, -1, "ded"]]);
    expect(m.snakes[1].alive).toBe(true);
  });

  it("the shrinking wall crushes a snake left on the outer ring", () => {
    const m = arena(
      { body: [[1, 30], [1, 31], [1, 32]], dir: 2 },
      { body: [[30, 30], [29, 30], [28, 30]], dir: 1 },
    );
    m.timeMs = SHRINK_EVERY_MS - m.stepMs; // this step reaches the shrink
    const r = step(m);
    expect(r.shrunk).toBe(true);
    expect(m.bounds).toEqual([2, 2, 58, 58]);
    expect(r.deaths).toEqual([[0, -1, "crushed"]]);
    expect([m.over, m.winner]).toEqual([true, 1]);
  });
});

describe("turns", () => {
  it("a 180 is ignored and a 90 is taken", () => {
    const m = arena(
      { body: [[20, 30], [19, 30], [18, 30]], dir: 1 },
      { body: [[40, 40], [39, 40], [38, 40]], dir: 1 },
    );
    turn(m, 0, 3);
    step(m);
    expect(m.snakes[0].body[0]).toBe(cell(21, 30));
    turn(m, 0, 0);
    step(m);
    expect(m.snakes[0].body[0]).toBe(cell(21, 31));
  });
});
