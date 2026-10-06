// Browser entry: room connection, input and the frame loop. Screens (screens.ts) and overlays
// (hud.ts) show the state; all match state comes from the server through the replica.
import "./style.css";
import { boosting, head, WALL_HOLD_MS, wrap, type Death, type Snake } from "../game/rules";
import type { ClientMsg, PlayerInfo, ServerMsg, StepMsg } from "../game/protocol";
import { apply, newReplica, type Replica } from "../game/replica";
import { createRenderer, snakeColor, STICK_PX, type Input } from "./render";
import { wsUrl } from "./net";
import { music, play } from "./sound";
import { $, clock, h } from "./dom";
import { load, mySkin, profile, save, saveProfile, settings, SKINS, touch, unlocked } from "./profile";
import { addFeed, clearFeed, hideDeath, hint, setWatch, showDeath, showSticker, toast, updateHud } from "./hud";
import {
  CODE, closeModal, hidePanel, modalOpen, openSettings, savedName, saveName, showDeadEnd, showHome, showRoom, showWait,
  type Act,
} from "./screens";

const banner = $("banner");
const sub = $("sub");
const canvas = $("game") as HTMLCanvasElement;
const boostBtn = $("boost");
const edge = $("edge");
const lagEl = $("lag");
const net = $("net");
const view = createRenderer(canvas);
// dev: ?lag=MS adds MS of round trip (half each way), to feel a deployed room on localhost
const lag = import.meta.env.DEV ? Number(new URLSearchParams(location.search).get("lag")) || 0 : 0;
const delayed = (f: () => void) => (lag ? void setTimeout(f, lag / 2) : f());
const withLag = (url: string) => (lag ? `${url}${url.includes("?") ? "&" : "?"}lag=${lag}` : url);

// per tab: a reload takes the same snake back, and two tabs are two players
const token = load(() => sessionStorage, "dbs-token") ?? crypto.randomUUID();
save(() => sessionStorage, "dbs-token", token);

let r: Replica = newReplica();
const input: Input = { steer: null, boost: false, stick: null, watch: null };
let sent: number | null = null; // the last heading sent
const boostBy = new Set<number | string>(); // what holds boost down: pointer ids, "Space"
let boostSent = false;
let sentAt = -Infinity;
let steerTimer = 0;
let lastStepAt = 0;
let countdownEnd = -Infinity;
let bannerText = "";
let bannerUntil = 0;
let deathUntil = 0; // the death card shows until then; the watch bar after
let cutHintAt = 0;
let ws: WebSocket | null = null;
let retries = 0;
let autostart = false; // "Play now": start the round as soon as the room says I am its host
let news: string[] = []; // the results card's badges: new bests, unlocks
let round = { bonks: 0, len: 0, had: unlocked() }; // my numbers this round, and the unlocks before it

const act: Act = {
  send,
  friends: () => void createRoom(false),
  solo: () => void createRoom(true),
  join: (code) => enter(code),
  home,
};

// ---- connection

function enter(code: string, auto = false) {
  history.replaceState(null, "", withLag(`?r=${code}`));
  r = newReplica();
  autostart = auto;
  retries = 0;
  showWait(auto ? "such bots. very ready..." : `joining room ${code}...`);
  connect(code);
}

function connect(code: string) {
  const sock = new WebSocket(wsUrl(code, token, savedName(), mySkin()));
  ws = sock;
  sock.onopen = () => (retries = 0);
  sock.onmessage = (e) => {
    net.hidden = true;
    delayed(() => onMessage(JSON.parse(e.data)));
  };
  sock.onclose = (e) => {
    if (ws !== sock) return;
    ws = null;
    if (e.code === 4000) return deadEnd("this room fell asleep.", "nobody played for 10 minutes. such quiet.", "Make a new room", () => createRoom(false));
    if (e.code === 4001) return deadEnd("this room is open in another tab.", "one doge, one tab. very rule.", "Play here", () => enter(code));
    if (!r.code) return deadEnd("much wrong room.", `room ${code} does not exist, or it is full.`, "Make your own room", () => createRoom(false));
    if (retries++ < 5) {
      net.hidden = false;
      return void setTimeout(() => connect(code), 500 * retries);
    }
    deadEnd("connection lost. very sad.", "the room could not be reached.", "Try again", () => enter(code));
  };
}

function send(msg: ClientMsg) {
  const sock = ws;
  delayed(() => sock?.readyState === WebSocket.OPEN && sock.send(JSON.stringify(msg)));
}

function leave() {
  ws?.close();
  ws = null;
  r = newReplica();
  net.hidden = true;
  resetRoundUi();
  updateHud(r);
}

function home() {
  leave();
  history.replaceState(null, "", withLag("/"));
  showHome(act);
}

function deadEnd(title: string, line: string, action: string, onAction: () => void) {
  leave();
  history.replaceState(null, "", withLag("/"));
  showDeadEnd(title, line, action, onAction, act);
}

async function createRoom(auto: boolean) {
  showWait("such room. very wait...");
  const res = await fetch("/api/rooms", { method: "POST" }).catch(() => null);
  if (!res?.ok) return deadEnd("much error.", "could not make a room.", "Try again", () => createRoom(auto));
  enter(((await res.json()) as { code: string }).code, auto);
}

function onMessage(msg: ServerMsg) {
  const now = performance.now();
  if (msg.t === "sticker") {
    const p = r.players.find((q) => q.id === msg.id);
    const color = p && p.seat >= 0 && r.m && r.phase !== "lobby" ? snakeColor(p.seat) : "#fff3d6";
    return showSticker(p?.name ?? "?", msg.k, color);
  }
  const before = r.m?.timeMs ?? 0;
  const prev = r.code ? r.players : null;
  if (msg.t === "snap") {
    countdownEnd = msg.countdownMs ? now + msg.countdownMs : -Infinity;
    lastStepAt = now;
    input.steer = sent = null;
    boostBy.clear();
    input.boost = boostSent = false;
    clearFeed();
    bannerUntil = 0;
    resetRoundUi();
    if (msg.m.tick === 0) round = { bonks: 0, len: 0, had: unlocked() }; // not on a reconnect mid-round
  } else if (msg.t === "step" && r.m) {
    onEat(msg);
  }

  apply(r, msg);

  if (msg.t === "room") {
    saveName(msg.name);
    if (prev) roomNews(prev);
    const me = r.players.find((p) => p.id === r.you);
    if (autostart && r.phase === "lobby" && me?.host) {
      autostart = false;
      send({ t: "start" });
    } else if (r.phase === "playing") {
      lastStepAt = now;
    }
    renderPanel();
  } else if (msg.t === "over") {
    // the victory moment: the camera moves in on the winner; the results panel waits (style.css)
    const m = r.m!;
    const w = m.winner >= 0 ? m.snakes[m.winner] : undefined;
    play(w && w.id === r.seat ? "win" : "gameOver");
    showBanner(!w ? "such draw! wow" : w.id === r.seat ? "you win! wow" : `${w.name} wins! wow`, 2500);
    if (w) view.cheer(w);
    endRound();
    resetRoundUi();
    renderPanel();
  } else if (msg.t === "step" && r.m) {
    lastStepAt = now;
    onStep(msg, before);
  }
  updateHud(r);
  updateWatch();
}

/** Who joined, and who took over from a host who left. */
function roomNews(prev: PlayerInfo[]) {
  for (const p of r.players) if (p.id !== r.you && !prev.some((q) => q.id === p.id)) toast(`${p.name} joined. wow`);
  const was = prev.find((p) => p.host);
  const now = r.players.find((p) => p.host);
  if (was && now && was.id !== now.id && !r.players.some((p) => p.id === was.id)) {
    toast(now.id === r.you ? `${was.name} left. you are host now. wow` : `${was.name} left. ${now.name} is host now`);
  }
}

function renderPanel() {
  if (r.phase === "countdown" || r.phase === "playing") return hidePanel();
  if (autostart) return; // the round starts in a moment; keep the wait card
  showRoom(r, act, news, devButtons());
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
    if (c.v > 1) view.pop(c.x, c.y, `+${c.v}`, "#f2b632");
  }
  if (mine) play("eat");
}

// feedback for what just happened; runs on the updated replica, so dead bodies include the fatal move
function onStep(msg: StepMsg, before: number) {
  const m = r.m!;
  for (const [id, killer, how] of msg.deaths) {
    const s = m.snakes[id];
    const by = how === "cut" ? "cut off" : "bonked";
    addFeed(how === "wall" ? `${s.name} squished! such wall!` : `${s.name} ${by} by ${m.snakes[killer].name}! wow`, snakeColor(id));
    view.burst(s, snakeColor(id));
    if (id === r.seat) {
      myDeath(killer, how);
    } else if (killer >= 0 && killer === r.seat) {
      const [x, y] = head(s);
      showBanner(how === "cut" ? "such cut! wow" : "much bonk! wow", 1200);
      view.pop(x, y, "+1 bonk", "#fff");
      view.shake(7);
      play("bonk");
      round.bonks++;
      profile.bonks++;
      if (how === "cut") profile.cut = profile.seen.cut = true;
      saveProfile();
    }
    if (input.watch === id) pickWatch();
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
  const me = r.seat >= 0 ? m.snakes[r.seat] : undefined;
  if (me?.alive) round.len = Math.max(round.len, me.len);
}

// ---- my death, and watching the others

const named = (s: Snake) => {
  const el = h("b", { textContent: s.name });
  el.style.color = snakeColor(s.id);
  return el;
};

function myDeath(killer: number, how: Death) {
  const m = r.m!;
  const k = killer >= 0 ? m.snakes[killer] : undefined;
  view.shake(14);
  const cause = !k ? ["squished by the wall. such wall."]
    : how === "cut" ? [named(k), " cut you off. very sneaky."] : ["you bonked into ", named(k), "'s bread."];
  const place = m.snakes.filter((s) => s.alive).length + 1;
  deathUntil = performance.now() + 2500;
  showDeath(h("span", {}, ...cause), `#${place} of ${m.snakes.length} · lived ${clock(m.timeMs / 1000)}`, updateWatch);
  // the camera holds on my crumbs, follows my killer for 3 s, then a friend
  input.watch = k?.alive ? k.id : null;
  if (input.watch === null) pickWatch();
  else setTimeout(() => r.m === m && input.watch === killer && pickWatch(), 4200);
}

/** The alive player with the most length, else the longest bot. */
function pickWatch() {
  const alive = (r.m?.snakes ?? []).filter((s) => s.alive && s.id !== r.seat).sort((a, b) => b.len - a.len);
  input.watch = (alive.find((s) => s.human) ?? alive[0])?.id ?? null;
}

function cycleWatch() {
  const alive = (r.m?.snakes ?? []).filter((s) => s.alive && s.id !== r.seat);
  if (!alive.length) return;
  input.watch = (alive.find((s) => s.id > (input.watch ?? -1)) ?? alive[0]).id;
  updateWatch();
}

let watchKey = "";
function updateWatch() {
  const m = r.m;
  const on = !!m && !m.over && (r.phase === "countdown" || r.phase === "playing") && !steerable() &&
    performance.now() >= deathUntil;
  const w = on ? m!.snakes[input.watch ?? -1] : undefined;
  const target = w?.alive ? w : m?.snakes.filter((s) => s.alive).sort((a, b) => b.len - a.len)[0];
  const key = on ? `${r.seat}:${target?.id}` : "";
  if (key === watchKey) return;
  watchKey = key;
  if (!on || !target) return setWatch(null, sendSticker);
  setWatch([...(r.seat < 0 ? ["match running. you play next round. "] : []), "👀 watching ", named(target), " · tap to switch"], sendSticker);
}
const sendSticker = (k: number) => send({ t: "sticker", k });

function resetRoundUi() {
  input.watch = null;
  deathUntil = 0;
  cutHintAt = 0;
  hideDeath();
  setWatch(null, sendSticker);
  watchKey = "";
  hint("");
}

/** After a round I played: stats, new bests and unlocks for the results card. */
function endRound() {
  news = [];
  const m = r.m!;
  if (r.seat < 0) return;
  profile.played++;
  if (m.winner === r.seat) profile.wins++;
  const len = Math.floor(round.len);
  if (len > profile.bestLen) {
    if (profile.bestLen > 0) news.push(`NEW BEST size: ${len}! wow`);
    profile.bestLen = len;
  }
  if (round.bonks > profile.bestBonks) {
    news.push(`NEW BEST: ${round.bonks} bonk${round.bonks === 1 ? "" : "s"} in a round! wow`);
    profile.bestBonks = round.bonks;
  }
  unlocked().forEach((has, i) => has && !round.had[i] && news.push(`such unlock: ${SKINS[i].name}! wow`));
  saveProfile();
}

// ---- input: my snake turns toward a heading at any angle; at most 10 headings a second go out,
// while boost on/off goes at once

const steerable = () => {
  const me = r.m && r.seat >= 0 ? r.m.snakes[r.seat] : undefined;
  return me?.alive && (r.phase === "playing" || r.phase === "countdown") ? me : undefined;
};
const boostButtonOn = () => touch && settings.boostButton;

function steerTo(a: number) {
  input.steer = a;
  flushSteer();
  if (!profile.seen.steer && steerable()) {
    profile.seen.steer = true;
    saveProfile();
  }
}

function flushSteer() {
  const a = input.steer;
  if (a === null || !steerable()) return;
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

function holdBoost(by: number | string, on: boolean) {
  if (on) boostBy.add(by);
  else boostBy.delete(by);
  flushBoost();
}

function flushBoost() {
  input.boost = boostBy.size > 0;
  const me = steerable();
  if (!me || input.boost === boostSent) return;
  send({ t: "boost", on: input.boost });
  boostSent = input.boost;
  if (input.boost && boosting({ boost: true, len: me.len })) play("boost");
  if (input.boost && r.phase === "playing" && !profile.seen.boost) {
    profile.seen.boost = true;
    saveProfile();
  }
}

// mouse: the snake heads toward the pointer, a held button boosts. Touch: a stick appears under the
// thumb, the drag direction steers, and the stick follows a long drag so reversing stays one short
// move; a second finger, the rocket button, or (with the button off) a double tap held down boosts.
// A thumb on the stick outranks a mouse resting on the same screen. Watching: a tap switches snake.
let lastLift = -Infinity; // when the stick's thumb last lifted
canvas.addEventListener("pointerdown", (e) => {
  if (!steerable()) return void ((r.phase === "playing" || r.phase === "countdown") && cycleWatch());
  if (e.pointerType === "mouse") return void (e.button === 0 && holdBoost(e.pointerId, true));
  if (input.stick) return holdBoost(e.pointerId, true);
  input.stick = { id: e.pointerId, ax: e.clientX, ay: e.clientY, x: e.clientX, y: e.clientY };
  if (!boostButtonOn() && e.timeStamp - lastLift < 300) holdBoost(e.pointerId, true);
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
// the rocket: a touch that starts here only boosts, never steers
boostBtn.addEventListener("pointerdown", (e) => {
  e.preventDefault();
  holdBoost(e.pointerId, true);
});
// on the window: a button or finger can be released anywhere
const lift = (e: PointerEvent) => {
  holdBoost(e.pointerId, false);
  if (input.stick?.id !== e.pointerId) return;
  input.stick = null;
  lastLift = e.timeStamp;
};
addEventListener("pointerup", lift);
addEventListener("pointercancel", lift);
// the page never scrolls or bounces under a steering thumb (Telegram's browser included); cards may
document.addEventListener("touchmove", (e) => {
  if (!(e.target as Element).closest?.("#panel, #modal")) e.preventDefault();
}, { passive: false });

// keys: arrows or WASD point the snake; two pressed together point it diagonally
const KEYS: Record<string, [number, number]> = {
  ArrowUp: [0, 1], KeyW: [0, 1], ArrowDown: [0, -1], KeyS: [0, -1],
  ArrowLeft: [-1, 0], KeyA: [-1, 0], ArrowRight: [1, 0], KeyD: [1, 0],
};
const held = new Set<string>();
addEventListener("keydown", (e) => {
  if (e.code === "Escape") return closeModal();
  if (e.target instanceof HTMLInputElement || modalOpen() || !steerable()) return;
  if (e.code === "Space") {
    e.preventDefault();
    return holdBoost("Space", true);
  }
  if (!KEYS[e.code]) return;
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
addEventListener("keyup", (e) => {
  held.delete(e.code);
  if (e.code === "Space") holdBoost("Space", false);
});
addEventListener("blur", () => {
  held.clear();
  boostBy.clear();
  flushBoost();
});

// ---- banners, first-match hints, and the loop

function showBanner(text: string, ms: number) {
  bannerText = text;
  bannerUntil = performance.now() + ms;
}

const boostHow = () => (!touch ? "hold click or space" : settings.boostButton ? "hold 🚀" : "double-tap and hold");
const controls = () => (touch ? "👆 drag anywhere to steer" : "🖱️ the mouse steers") + ` · ${boostHow()} to zoom`;

/** One hint at a time at the top, each until its action is done: steer, boost, then the cut-off. */
function hints(now: number) {
  const me = steerable();
  const m = r.m;
  const seen = profile.seen;
  let text = "";
  if (me && m && r.phase === "playing") {
    if (!seen.steer) text = touch ? "👆 drag anywhere to steer" : "🖱️ move the mouse to steer";
    else if (!seen.boost && m.timeMs > 8000 && me.len > 20) text = `🚀 ${boostHow()} to zoom`;
    else if (seen.boost && !seen.cut) {
      const [x, y] = head(me);
      const near = m.snakes.some((s) => s.alive && s !== me && Math.hypot(head(s)[0] - x, head(s)[1] - y) < 12);
      if (!cutHintAt && near) cutHintAt = now;
      if (cutHintAt) text = "✂️ zoom in front of a snek: it bonks into you. wow";
      if (cutHintAt && now - cutHintAt > 5000) {
        seen.cut = true;
        saveProfile();
      }
    }
  }
  hint(text);
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
  const controlsText = left > 0 && profile.played < 3 && steerable() ? controls() : "";
  if (sub.textContent !== controlsText) sub.textContent = controlsText;
  hints(now);
  const m = r.m;
  const live = !!m && r.phase === "playing" && !m.over;
  edge.classList.toggle("on", live && m!.timeMs >= WALL_HOLD_MS - 5000 && m!.timeMs < WALL_HOLD_MS);
  lagEl.hidden = !(live && now - lastStepAt > 500);
  boostBtn.hidden = !(boostButtonOn() && steerable());
  boostBtn.classList.toggle("on", input.boost);
  if (deathUntil && now >= deathUntil) updateWatch();
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

$("gear").onclick = () => openSettings(() => music(settings.music && document.body.dataset.screen !== "play"));

const asked = new URLSearchParams(location.search).get("r")?.toUpperCase() ?? "";
if (CODE.test(asked)) enter(asked);
else home();
requestAnimationFrame(frame);
