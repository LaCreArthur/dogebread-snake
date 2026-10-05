// A client's copy of the room, rebuilt from server messages only. Used by the browser and by the
// test AI players, so both see exactly what a real player sees.
import { cx, cy, inside, type Dir } from "./rules";
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

function applyStep(m: PublicMatch, s: StepMsg) {
  m.tick = s.tick;
  m.timeMs = s.timeMs;
  m.stepMs = s.stepMs;
  for (const [id, head, len] of s.moved) {
    const snake = m.snakes[id];
    snake.dir = dirBetween(snake.body[0], head);
    snake.body.unshift(head);
    snake.body.length = Math.min(snake.body.length, len);
  }
  for (const [id, killer] of s.deaths) {
    m.snakes[id].alive = false;
    m.snakes[id].diedTick = s.tick;
    if (killer >= 0) m.snakes[killer].kills++;
  }
  // same as the server: a head on a cell eats every coin there
  const eaten = new Set(s.eaten.map(([, c]) => c));
  for (const [id] of s.eaten) m.snakes[id].score++;
  m.bounds = s.bounds;
  m.food = [...m.food.filter((f) => !eaten.has(f)), ...s.spawned].filter((f) => inside(m.bounds, f));
}

function dirBetween(from: number, to: number): Dir {
  const dx = cx(to) - cx(from);
  const dy = cy(to) - cy(from);
  return dx > 0 ? 1 : dx < 0 ? 3 : dy > 0 ? 0 : 2;
}
