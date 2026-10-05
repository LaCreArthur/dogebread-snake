// A client's copy of the room, rebuilt from server messages only. Used by the browser and by the
// test AI players, so both see exactly what a real player sees.
import { trimTrail } from "./rules";
import type { NameIdx, Phase, PlayerInfo, PublicMatch, ServerMsg, StepMsg } from "./protocol";

export interface Replica {
  code: string;
  phase: Phase;
  players: PlayerInfo[];
  you: number;
  name: NameIdx;
  seat: number; // my snake id, -1 = watching
  m: PublicMatch | null;
  rank: number[];
}

export const newReplica = (): Replica => ({
  code: "", phase: "lobby", players: [], you: 0, name: [0, 0], seat: -1, m: null, rank: [],
});

export function apply(r: Replica, msg: ServerMsg) {
  if (msg.t === "room") {
    r.code = msg.code;
    r.phase = msg.phase;
    r.players = msg.players;
    r.you = msg.you;
    r.name = msg.name;
    r.seat = msg.players.find((p) => p.id === msg.you)?.seat ?? -1;
  } else if (msg.t === "snap") {
    r.m = msg.m;
    r.rank = [];
  } else if (msg.t === "step" && r.m) {
    applyStep(r.m, msg);
  } else if (msg.t === "over" && r.m) {
    r.m.over = true;
    r.m.winner = msg.winner;
    r.rank = msg.rank;
  }
}

// the server's order: move, deaths, eat, spawn, then the wall takes what is outside
function applyStep(m: PublicMatch, s: StepMsg) {
  m.tick = s.tick;
  m.timeMs = s.timeMs;
  m.wall = s.wall;
  for (const [id, x, y, a, ta, len] of s.moved) {
    const snake = m.snakes[id];
    snake.trail.push(x, y);
    snake.a = a;
    snake.ta = ta;
    snake.len = len;
  }
  for (const [id, killer] of s.deaths) {
    m.snakes[id].alive = false;
    m.snakes[id].diedTick = s.tick;
    if (killer >= 0) m.snakes[killer].kills++;
  }
  const eaten = new Set(s.eaten.map(([c]) => c));
  m.crumbs = m.crumbs.filter((c) => !eaten.has(c.id));
  for (const [id, x, y, v] of s.spawned) m.crumbs.push({ id, x, y, v });
  m.crumbs = m.crumbs.filter((c) => c.x ** 2 + c.y ** 2 <= m.wall ** 2);
  for (const snake of m.snakes) if (snake.alive) trimTrail(snake.trail, snake.len);
}
