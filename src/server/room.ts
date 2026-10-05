// One Durable Object per room. It owns the match: rules, randomness and bots run here, and every
// step it broadcasts what changed. Plain (non-hibernating) WebSockets keep it in memory while
// anyone is connected; idle rooms close so they stop using the free duration quota.
import { DurableObject } from "cloudflare:workers";
import { boost, createMatch, ranking, SEATS, steer, step, TICK_MS, type Match } from "../game/rules";
import {
  NAME_ADJ, NAME_NOUN, nameOf, validName,
  type ClientMsg, type NameIdx, type Phase, type ServerMsg,
} from "../game/protocol";

const MAX_CONNECTIONS = 16;
const COUNTDOWN_MS = 3_500;
const IDLE_MS = 10 * 60_000;
const EMPTY_GRACE_MS = 15_000; // lets a reload take its snake back before the room resets
const MAX_MSGS_PER_SEC = 20;

interface Player {
  id: number;
  token: string; // private: lets a reloaded tab take its snake back
  name: NameIdx;
  ws: WebSocket | null;
  seat: number;
  msgs: number;
  msgWindow: number;
}

export class Room extends DurableObject<Env> {
  private code = "";
  private players: Player[] = [];
  private nextId = 1;
  private host = 0;
  private phase: Phase = "lobby";
  private match: Match | null = null;
  private startsAt = 0; // end of the countdown (ms epoch)
  private timer: number | null = null;
  private idle: number | null = null;

  /** Only codes made by the Worker exist; returns false if the code is taken. */
  async init(code: string): Promise<boolean> {
    if (await this.ctx.storage.get("code")) return false;
    await this.ctx.storage.put("code", code);
    return true;
  }

  async fetch(req: Request): Promise<Response> {
    this.code ||= (await this.ctx.storage.get<string>("code")) ?? "";
    if (!this.code) return new Response("no such room", { status: 404 });
    if (this.players.filter((p) => p.ws).length >= MAX_CONNECTIONS) return new Response("full", { status: 503 });

    const url = new URL(req.url);
    const token = (url.searchParams.get("p") ?? "").slice(0, 64) || crypto.randomUUID();
    const asked = (url.searchParams.get("n") ?? "").split(".").map(Number);
    const [client, server] = Object.values(new WebSocketPair());
    server.accept();
    this.join(server, token, validName(asked) ? asked : null);
    return new Response(null, { status: 101, webSocket: client });
  }

  private join(ws: WebSocket, token: string, name: NameIdx | null) {
    let p = this.players.find((q) => q.token === token);
    if (p) {
      p.ws?.close(4001, "replaced");
      p.ws = ws;
    } else {
      const taken = name && this.players.some((q) => nameOf(q.name) === nameOf(name));
      p = { id: this.nextId++, token, name: name && !taken ? name : this.freeName(), ws, seat: -1, msgs: 0, msgWindow: 0 };
      this.players.push(p);
    }
    const me = p;
    ws.addEventListener("message", (e) => this.onMessage(me, ws, e.data));
    ws.addEventListener("close", () => this.onClose(me, ws));
    ws.addEventListener("error", () => this.onClose(me, ws));
    if (!this.players.some((q) => q.id === this.host && q.ws)) this.host = me.id;
    if (this.match) {
      const countdownMs = this.phase === "countdown" ? Math.max(0, this.startsAt - Date.now()) : 0;
      this.send(ws, { t: "snap", m: publicMatch(this.match), countdownMs });
    }
    this.sendRoom();
    this.armIdle();
  }

  private onMessage(p: Player, ws: WebSocket, data: unknown) {
    const now = Date.now();
    if (now - p.msgWindow >= 1000) {
      p.msgWindow = now;
      p.msgs = 0;
    }
    if (++p.msgs > MAX_MSGS_PER_SEC || typeof data !== "string" || data.length > 100) {
      ws.close(1008, "too many messages");
      return;
    }
    let msg: ClientMsg;
    try {
      msg = JSON.parse(data);
    } catch {
      return;
    }
    if (msg.t === "steer" && this.match && p.seat >= 0 && typeof msg.a === "number") {
      steer(this.match, p.seat, msg.a);
    } else if (msg.t === "boost" && this.match && p.seat >= 0 && typeof msg.on === "boolean") {
      boost(this.match, p.seat, msg.on);
    } else if (msg.t === "start" && p.id === this.host && (this.phase === "lobby" || this.phase === "over")) {
      this.start();
    } else if (msg.t === "name") {
      p.name = this.freeName();
      this.sendRoom();
    }
  }

  private onClose(p: Player, ws: WebSocket) {
    if (p.ws !== ws) return; // already replaced by a newer tab
    p.ws = null;
    // between matches a leaver is simply gone; during a match the seat waits for a reload
    if (this.phase === "lobby" || this.phase === "over") this.players = this.players.filter((q) => q !== p);
    if (!this.players.some((q) => q.ws)) return this.armIdle();
    if (p.id === this.host) this.host = this.players.find((q) => q.ws)!.id;
    this.sendRoom();
  }

  private start() {
    this.players = this.players.filter((p) => p.ws);
    this.players.forEach((p, i) => (p.seat = i < SEATS ? i : -1));
    const seated = this.players.filter((p) => p.seat >= 0);
    this.match = createMatch(crypto.getRandomValues(new Uint32Array(1))[0], seated.map((p) => nameOf(p.name)));
    this.phase = "countdown";
    this.startsAt = Date.now() + COUNTDOWN_MS;
    clearTimeout(this.idle);
    this.broadcast({ t: "snap", m: publicMatch(this.match), countdownMs: COUNTDOWN_MS });
    this.sendRoom();
    this.timer = setTimeout(() => {
      this.phase = "playing";
      this.sendRoom();
      this.loop();
    }, COUNTDOWN_MS);
  }

  // ticks run on a fixed schedule from the start, so timer delays do not add up
  private loop() {
    const m = this.match!;
    const due = this.startsAt + (m.tick + 1) * TICK_MS;
    this.timer = setTimeout(() => {
      const r = step(m);
      this.broadcast({ t: "step", tick: m.tick, timeMs: m.timeMs, wall: m.wall, ...r });
      // one line per death and per end, the playtest numbers (cut-off share, death times)
      for (const [id, killer, how] of r.deaths) console.log(`death ${who(m, id)} ${how} ${who(m, killer)} ${m.timeMs / 1000}s`);
      if (!m.over) return this.loop();
      console.log(`over ${who(m, m.winner)} ${m.timeMs / 1000}s`);
      this.phase = "over";
      this.broadcast({ t: "over", winner: m.winner, rank: ranking(m) });
      this.players = this.players.filter((p) => p.ws);
      this.sendRoom();
      this.armIdle();
    }, Math.max(0, due - Date.now()));
  }

  /** Nobody connected: stop everything so the room costs nothing. */
  private reset() {
    clearTimeout(this.timer);
    clearTimeout(this.idle);
    this.players = [];
    this.match = null;
    this.phase = "lobby";
  }

  /** Empty room: reset soon. Lobby or results with people: close after IDLE_MS. Match with people: never. */
  private armIdle() {
    clearTimeout(this.idle);
    const empty = !this.players.some((p) => p.ws);
    if (!empty && this.phase !== "lobby" && this.phase !== "over") return;
    this.idle = setTimeout(() => {
      for (const p of this.players) p.ws?.close(4000, "idle");
      this.reset();
    }, empty ? EMPTY_GRACE_MS : IDLE_MS);
  }

  private freeName(): NameIdx {
    const taken = new Set(this.players.map((p) => nameOf(p.name)));
    let name: NameIdx = [0, 0];
    for (let i = 0; i < 20; i++) {
      const r = crypto.getRandomValues(new Uint8Array(2));
      name = [r[0] % NAME_ADJ.length, r[1] % NAME_NOUN.length];
      if (!taken.has(nameOf(name))) break;
    }
    return name;
  }

  private sendRoom() {
    const players = this.players
      .filter((p) => p.ws || p.seat >= 0)
      .map((p) => ({ id: p.id, name: nameOf(p.name), host: p.id === this.host, seat: p.seat }));
    for (const p of this.players) {
      if (p.ws) this.send(p.ws, { t: "room", code: this.code, phase: this.phase, players, you: p.id, name: p.name });
    }
  }

  private broadcast(msg: ServerMsg) {
    const data = JSON.stringify(msg);
    for (const p of this.players) if (p.ws) trySend(p.ws, data);
  }

  private send(ws: WebSocket, msg: ServerMsg) {
    trySend(ws, JSON.stringify(msg));
  }
}

function trySend(ws: WebSocket, data: string) {
  try {
    ws.send(data);
  } catch {
    // socket already closing; its close handler cleans up
  }
}

function publicMatch({ rng: _, nextCrumb: __, ...m }: Match) {
  return m;
}

/** A snake's name for the logs, starred when a human plays it; "-" for no snake (the wall, a draw). */
function who(m: Match, id: number) {
  const s = m.snakes[id];
  return s ? `${s.name}${s.human ? "*" : ""}` : "-";
}
