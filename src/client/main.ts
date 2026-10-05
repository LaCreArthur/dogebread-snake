// Browser entry: home screen, room connection, lobby panel, HUD and input. All match state comes
// from the server through the replica; this file only shows it and sends turns.
import "./style.css";
import { cx, cy, opposite, SEATS, shrinkWarning, type Dir } from "../game/rules";
import { validName, type ClientMsg, type NameIdx, type ServerMsg, type StepMsg } from "../game/protocol";
import { apply, newReplica, type Replica } from "../game/replica";
import { createRenderer, snakeColor } from "./render";
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
let pending: Dir | -1 = -1; // my turn, sent but not applied yet
let pendingTick = 0;
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
  const warned = r.m !== null && shrinkWarning(r.m);
  if (msg.t === "snap") {
    countdownEnd = msg.countdownMs ? now + msg.countdownMs : -Infinity;
    pending = -1;
    feed.replaceChildren();
    bannerUntil = 0;
  }

  apply(r, msg);

  if (msg.t === "room") {
    save(() => localStorage, "dbs-name", msg.name.join("."));
    renderPanel();
  } else if (msg.t === "over") {
    play("gameOver");
    renderPanel();
  } else if (msg.t === "step" && r.m) {
    onStep(msg, warned);
  }
  updateHud();
}

const dist = (a: number, b: number) => Math.abs(cx(a) - cx(b)) + Math.abs(cy(a) - cy(b));

// feedback for what just happened; runs on the updated replica, so dead bodies include the fatal cell
function onStep(msg: StepMsg, warned: boolean) {
  const m = r.m!;
  const me = r.seat >= 0 ? m.snakes[r.seat] : undefined;
  for (const [id, killer, how] of msg.deaths) {
    const s = m.snakes[id];
    const text = how === "crushed" ? `${s.name} crushed! such squish!`
      : killer >= 0 ? `${s.name} bonked by ${m.snakes[killer].name}! wow`
      : `${s.name} ded! much rip!`;
    addFeed(text, snakeColor(id));
    view.burst(s.body, snakeColor(id));
    if (id === r.seat) {
      showBanner("wow. such ded.", 2500);
      view.shake(14);
    } else if (killer >= 0 && killer === r.seat) {
      showBanner("much bonk! wow", 1200);
      view.pop(s.body[0], "+1 bonk", "#fff");
      view.shake(7);
      play("bonk");
    } else if (me?.alive && dist(me.body[0], s.body[0]) < 12) {
      view.shake(4);
    }
  }
  if (msg.deaths.length) play("death");
  for (const [id, c] of msg.eaten) {
    view.sparkle(c);
    if (id !== r.seat) continue;
    view.pop(c, "+1", "#e8b04a");
    play("eat");
  }
  if (msg.shrunk) {
    play("shrinkImpact");
    view.shake(4);
  }
  if (msg.spedUp) {
    showBanner("much fast! wow!", 1500);
    play("speedUp");
  }
  if (!warned && shrinkWarning(m)) play("shrinkWarning");
  if (pending !== -1 && (!me?.alive || me.dir === pending || msg.tick >= pendingTick + 2)) pending = -1;
}

// ---- input

function sendTurn(d: Dir) {
  const me = r.m && r.seat >= 0 ? r.m.snakes[r.seat] : undefined;
  if (!me?.alive || (r.phase !== "playing" && r.phase !== "countdown")) return;
  if (d === opposite(me.dir) || d === (pending === -1 ? me.dir : pending)) return;
  send({ t: "turn", d });
  pending = d;
  pendingTick = r.m!.tick;
}

const KEYS: Record<string, Dir> = {
  ArrowUp: 0, KeyW: 0, ArrowRight: 1, KeyD: 1, ArrowDown: 2, KeyS: 2, ArrowLeft: 3, KeyA: 3,
};
addEventListener("keydown", (e) => {
  const d = KEYS[e.code];
  if (d === undefined || e.target instanceof HTMLInputElement) return;
  e.preventDefault();
  sendTurn(d);
});

// a swipe turns as soon as the finger has moved 30px; one touch can chain several turns
let touch: { x: number; y: number } | null = null;
canvas.addEventListener("touchstart", (e) => {
  touch = { x: e.touches[0].clientX, y: e.touches[0].clientY };
}, { passive: true });
canvas.addEventListener("touchmove", (e) => {
  const t = e.touches[0];
  if (!touch) return;
  const dx = t.clientX - touch.x;
  const dy = t.clientY - touch.y;
  if (Math.max(Math.abs(dx), Math.abs(dy)) < 30) return;
  sendTurn(Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 1 : 3) : dy > 0 ? 2 : 0);
  touch = { x: t.clientX, y: t.clientY };
}, { passive: true });

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
    h("p", { textContent: "10 snakes, one shrinking arena. Last snake alive wins. Bots fill the empty seats." }),
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
    (me ? `<span>coins <b>${me.score}</b></span><span>bonks <b>${me.kills}</b></span>` : "<span>watching</span>") +
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
  view.draw(r, pending, now);
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
if (import.meta.env.DEV) Object.assign(window, { __dbs: { replica: () => r, ais } });

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
