// Browser entry: home screen, room connection, lobby panel, HUD and input. All match state comes
// from the server through the replica; this file only shows it and sends my heading.
import "./style.css";
import { head, SEATS, WALL_HOLD_MS, wrap } from "../game/rules";
import { validName, type ClientMsg, type NameIdx, type ServerMsg, type StepMsg } from "../game/protocol";
import { apply, newReplica, type Replica } from "../game/replica";
import { createRenderer, snakeColor, STICK_PX, type Input } from "./render";
import { wsUrl } from "./net";
import { isMuted, play, toggleMute } from "./sound";

const $ = (id: string) => document.getElementById(id)!;
const hud = $("hud");
const feed = $("feed");
const banner = $("banner");
const panel = $("panel");
const canvas = $("game") as HTMLCanvasElement;
const mute = $("mute");
const view = createRenderer(canvas);
const CODE = /^[2-9A-HJKMNP-Z]{5}$/;
// dev: ?lag=MS adds MS of round trip (half each way), to feel a deployed room on localhost
const lag = import.meta.env.DEV ? Number(new URLSearchParams(location.search).get("lag")) || 0 : 0;
const delayed = (f: () => void) => (lag ? void setTimeout(f, lag / 2) : f());
const withLag = (url: string) => (lag ? `${url}${url.includes("?") ? "&" : "?"}lag=${lag}` : url);

// storage can throw (private mode, blocked site data): treat that as empty
const load = (s: () => Storage, k: string) => {
  try { return s().getItem(k); } catch { return null; }
};
const save = (s: () => Storage, k: string, v: string) => {
  try { s().setItem(k, v); } catch { /* not remembered */ }
};
// per tab: a reload takes the same snake back, and two tabs are two players
const token = load(() => sessionStorage, "dbs-token") ?? crypto.randomUUID();
save(() => sessionStorage, "dbs-token", token);
const savedName = (): NameIdx | null => {
  const n = (load(() => localStorage, "dbs-name") ?? "").split(".").map(Number);
  return validName(n) ? n : null;
};

let r: Replica = newReplica();
const input: Input = { steer: null, stick: null };
let sent: number | null = null; // the last heading sent
let sentAt = -Infinity;
let steerTimer = 0;
let lastStepAt = 0;
let countdownEnd = -Infinity;
let bannerText = "";
let bannerUntil = 0;
let ws: WebSocket | null = null;
let retries = 0;

// ---- connection

function enter(code: string) {
  history.replaceState(null, "", withLag(`?r=${code}`));
  r = newReplica();
  retries = 0;
  panel.hidden = false;
  panel.replaceChildren(h("p", { textContent: `Joining room ${code}...` }));
  connect(code);
}

function connect(code: string) {
  const sock = new WebSocket(wsUrl(code, token, savedName()));
  ws = sock;
  sock.onopen = () => (retries = 0);
  sock.onmessage = (e) => delayed(() => onMessage(JSON.parse(e.data)));
  sock.onclose = (e) => {
    if (ws !== sock) return;
    ws = null;
    if (e.code === 4000) return home("This room closed: nobody played for 10 minutes.");
    if (e.code === 4001) return home("You opened this room in another tab.");
    if (!r.code) return home(`Could not join room ${code}. Wrong code, or the room is full.`);
    if (retries++ < 5) setTimeout(() => connect(code), 500 * retries);
    else home("Connection lost.");
  };
}

function send(msg: ClientMsg) {
  const sock = ws;
  delayed(() => sock?.readyState === WebSocket.OPEN && sock.send(JSON.stringify(msg)));
}

function onMessage(msg: ServerMsg) {
  const now = performance.now();
  const before = r.m?.timeMs ?? 0;
  if (msg.t === "snap") {
    countdownEnd = msg.countdownMs ? now + msg.countdownMs : -Infinity;
    input.steer = sent = null;
    feed.replaceChildren();
    bannerUntil = 0;
  } else if (msg.t === "step" && r.m) {
    onEat(msg);
  }

  apply(r, msg);

  if (msg.t === "room") {
    save(() => localStorage, "dbs-name", msg.name.join("."));
    renderPanel();
  } else if (msg.t === "over") {
    play("gameOver");
    renderPanel();
  } else if (msg.t === "step" && r.m) {
    lastStepAt = now;
    onStep(msg, before);
  }
  updateHud();
}

// eaten crumbs sparkle where they were, so this runs before the step removes them
function onEat(msg: StepMsg) {
  if (!msg.eaten.length) return;
  const eaten = new Map(msg.eaten);
  let mine = false;
  for (const c of r.m!.crumbs) {
    const id = eaten.get(c.id);
    if (id === undefined) continue;
    view.sparkle(c.x, c.y);
    if (id !== r.seat) continue;
    mine = true;
    if (c.v > 1) view.pop(c.x, c.y, `+${c.v}`, "#e8b04a");
  }
  if (mine) play("eat");
}

// feedback for what just happened; runs on the updated replica, so dead bodies include the fatal move
function onStep(msg: StepMsg, before: number) {
  const m = r.m!;
  for (const [id, killer, how] of msg.deaths) {
    const s = m.snakes[id];
    addFeed(how === "wall" ? `${s.name} squished! such wall!` : `${s.name} bonked by ${m.snakes[killer].name}! wow`, snakeColor(id));
    view.burst(s, snakeColor(id));
    if (id === r.seat) {
      showBanner("wow. such ded.", 2500);
      view.shake(14);
    } else if (killer >= 0 && killer === r.seat) {
      const [x, y] = head(s);
      showBanner("much bonk! wow", 1200);
      view.pop(x, y, "+1 bonk", "#fff");
      view.shake(7);
      play("bonk");
    }
  }
  if (msg.deaths.length) play("death");
  for (const [a, b] of msg.boops) {
    const [ax, ay] = head(m.snakes[a]);
    const [bx, by] = head(m.snakes[b]);
    view.pop((ax + bx) / 2, (ay + by) / 2, "boop!", "#fff");
    if (a !== r.seat && b !== r.seat) continue;
    view.shake(4);
    play("boop");
  }
  if (before < WALL_HOLD_MS && m.timeMs >= WALL_HOLD_MS) {
    showBanner("much wall! wow", 1500);
    play("shrinkWarning");
  }
}

// ---- input: my snake turns toward a heading at any angle; at most 10 headings a second go out

function steerTo(a: number) {
  input.steer = a;
  flushSteer();
}

function flushSteer() {
  const me = r.m && r.seat >= 0 ? r.m.snakes[r.seat] : undefined;
  const a = input.steer;
  if (a === null || !me?.alive || (r.phase !== "playing" && r.phase !== "countdown")) return;
  if (sent !== null && Math.abs(wrap(a - sent)) < 0.03) return;
  const wait = sentAt + 100 - performance.now();
  if (wait > 0) {
    steerTimer ||= window.setTimeout(() => {
      steerTimer = 0;
      flushSteer();
    }, wait);
    return;
  }
  send({ t: "steer", a: Math.round(a * 1000) / 1000 });
  sent = a;
  sentAt = performance.now();
}

// mouse: the snake heads toward the pointer. Touch: a stick appears under the thumb, the drag
// direction steers, and the stick follows a long drag so reversing stays one short move. A thumb
// on the stick outranks a mouse resting on the same screen.
canvas.addEventListener("pointerdown", (e) => {
  if (e.pointerType !== "mouse" && !input.stick) {
    input.stick = { id: e.pointerId, ax: e.clientX, ay: e.clientY, x: e.clientX, y: e.clientY };
  }
});
canvas.addEventListener("pointermove", (e) => {
  if (e.pointerType === "mouse") {
    if (input.stick) return;
    const dx = e.clientX - innerWidth / 2;
    const dy = e.clientY - innerHeight / 2;
    if (dx * dx + dy * dy > 100) steerTo(Math.atan2(-dy, dx));
    return;
  }
  const st = input.stick;
  if (st?.id !== e.pointerId) return;
  const dx = e.clientX - st.ax;
  const dy = e.clientY - st.ay;
  const d = Math.hypot(dx, dy);
  if (d > STICK_PX) {
    st.ax = e.clientX - (dx / d) * STICK_PX;
    st.ay = e.clientY - (dy / d) * STICK_PX;
  }
  st.x = e.clientX;
  st.y = e.clientY;
  if (d > 12) steerTo(Math.atan2(-dy, dx));
});
const lift = (e: PointerEvent) => {
  if (input.stick?.id === e.pointerId) input.stick = null;
};
canvas.addEventListener("pointerup", lift);
canvas.addEventListener("pointercancel", lift);

// keys: arrows or WASD point the snake; two pressed together point it diagonally
const KEYS: Record<string, [number, number]> = {
  ArrowUp: [0, 1], KeyW: [0, 1], ArrowDown: [0, -1], KeyS: [0, -1],
  ArrowLeft: [-1, 0], KeyA: [-1, 0], ArrowRight: [1, 0], KeyD: [1, 0],
};
const held = new Set<string>();
addEventListener("keydown", (e) => {
  if (!KEYS[e.code] || e.target instanceof HTMLInputElement) return;
  e.preventDefault();
  held.add(e.code);
  if (e.repeat) return;
  let x = 0;
  let y = 0;
  for (const k of held) {
    x += KEYS[k][0];
    y += KEYS[k][1];
  }
  if (x || y) steerTo(Math.atan2(y, x));
});
addEventListener("keyup", (e) => held.delete(e.code));
addEventListener("blur", () => held.clear());

// ---- screens

function h<K extends keyof HTMLElementTagNameMap>(tag: K, props: Record<string, unknown> = {}, ...kids: (Node | string)[]) {
  const el = document.createElement(tag);
  Object.assign(el, props);
  el.append(...kids);
  return el;
}

function home(error = "") {
  ws?.close();
  ws = null;
  r = newReplica();
  history.replaceState(null, "", withLag("/"));
  const input = h("input", { maxLength: 5, placeholder: "CODE", autocomplete: "off" });
  const join = h("form", { className: "row" }, input, h("button", { textContent: "Join" }));
  join.onsubmit = (e) => {
    e.preventDefault();
    const code = input.value.trim().toUpperCase();
    if (CODE.test(code)) enter(code);
  };
  panel.hidden = false;
  panel.replaceChildren(
    h("h1", { textContent: "DogeBread Snake" }),
    h("p", { textContent: "Up to 10 players in one shrinking arena. Last snake alive wins. Bots fill the empty seats." }),
    h("p", { textContent: "Steer with the mouse, a thumb drag, or the arrow keys." }),
    h("button", { className: "main", textContent: "Create room", onclick: createRoom }),
    join,
    ...(error ? [h("p", { textContent: error })] : []),
  );
}

async function createRoom() {
  const res = await fetch("/api/rooms", { method: "POST" }).catch(() => null);
  if (!res?.ok) return home("Could not create a room. Try again.");
  enter(((await res.json()) as { code: string }).code);
}

function renderPanel() {
  if (r.phase === "countdown" || r.phase === "playing") {
    panel.hidden = true;
    return;
  }
  const me = r.players.find((p) => p.id === r.you);
  const host = r.players.find((p) => p.host);
  const link = `${location.origin}/?r=${r.code}`;
  const invite = h("button", {
    textContent: "Invite friends",
    onclick: async () => {
      if (navigator.share) return navigator.share({ title: "DogeBread Snake", url: link }).catch(() => {});
      await navigator.clipboard.writeText(link).catch(() => {});
      invite.textContent = "Link copied!";
    },
  });
  const start = me?.host
    ? h("button", { className: "main", textContent: r.phase === "over" ? "Play again" : "Start", onclick: () => send({ t: "start" }) })
    : h("p", { textContent: `Waiting for ${host?.name ?? "the host"} to start.` });
  const top = r.phase === "over" && r.m ? [h("h2", { textContent: resultText() })] : [h("h1", { textContent: "DogeBread Snake" })];
  const place = r.rank.indexOf(r.seat) + 1;
  panel.hidden = false;
  panel.replaceChildren(
    ...top,
    ...(r.phase === "over" && place ? [h("p", { textContent: `You placed #${place} of ${r.rank.length}.` })] : []),
    h("div", { className: "code", textContent: r.code }),
    invite,
    h("ul", {}, ...r.players.map((p) => h("li", { textContent: `${p.name}${p.id === r.you ? " (you)" : ""}${p.host ? " 👑" : ""}` }))),
    h("p", { textContent: `${Math.min(r.players.length, SEATS)}/${SEATS} players. Bots fill the empty seats.` }),
    h("div", { className: "row" }, h("button", { textContent: "New name", onclick: () => send({ t: "name" }) }), start),
    ...devButtons(),
  );
}

function resultText() {
  const m = r.m!;
  const w = m.winner >= 0 ? m.snakes[m.winner] : undefined;
  if (!w) return "wow such draw! no survivors! very dead!";
  if (w.id === r.seat) return "you win! very champion! such skill! wow!";
  return w.human ? `${w.name} wins! very champion! such skill! wow!` : `${w.name} wins! much skill! so impress!`;
}

// ---- HUD, feed, banners

function updateHud() {
  const m = r.m;
  if (!m || r.phase === "lobby") return void (hud.textContent = "");
  const alive = m.snakes.filter((s) => s.alive).length;
  const t = Math.floor(m.timeMs / 1000);
  const me = r.seat >= 0 ? m.snakes[r.seat] : undefined;
  hud.innerHTML = `<span>alive <b>${alive}/${m.snakes.length}</b></span>` +
    `<span><b>${Math.floor(t / 60)}:${String(t % 60).padStart(2, "0")}</b></span>` +
    (me ? `<span>size <b>${Math.floor(me.len)}</b></span><span>bonks <b>${me.kills}</b></span>` : "<span>watching</span>") +
    (lag ? `<span>lag <b>${lag}</b></span>` : "");
}

function addFeed(text: string, color: string) {
  const el = h("div", { textContent: text });
  el.style.color = color;
  feed.prepend(el);
  while (feed.children.length > 5) feed.lastChild!.remove();
  setTimeout(() => (el.style.opacity = "0"), 3500);
  setTimeout(() => el.remove(), 4000);
}

function showBanner(text: string, ms: number) {
  bannerText = text;
  bannerUntil = performance.now() + ms;
}

function frame(now: number) {
  view.draw(r, input, now - lastStepAt, now);
  const left = countdownEnd - now;
  const text = left > 2500 ? "such 3" : left > 1500 ? "very 2" : left > 500 ? "much 1" : left > -500 ? "WOW GO!"
    : now < bannerUntil ? bannerText : "";
  if (banner.textContent !== text) {
    if (["such 3", "very 2", "much 1"].includes(text)) play("beep");
    else if (text === "WOW GO!") play("go");
    banner.textContent = text;
  }
  requestAnimationFrame(frame);
}

// ---- dev: AI players join over the network like real people (npm run dev only)

const ais: Replica[] = [];
if (import.meta.env.DEV) Object.assign(window, { __dbs: { replica: () => r, ais, input } });

function devButtons() {
  if (!import.meta.env.DEV) return [];
  return [h("button", {
    textContent: "+ AI player",
    onclick: async () => ais.push((await import("./aiPlayer")).addAiPlayer(r.code)),
  })];
}

const showMute = () => (mute.textContent = isMuted() ? "🔇" : "🔊");
mute.onclick = () => {
  toggleMute();
  showMute();
};
showMute();

const asked = new URLSearchParams(location.search).get("r")?.toUpperCase() ?? "";
if (CODE.test(asked)) enter(asked);
else home();
requestAnimationFrame(frame);
