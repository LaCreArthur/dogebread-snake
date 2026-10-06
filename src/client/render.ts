// Canvas drawing. The camera follows my snake (or, a moment after I die, the snake I watch) and zooms
// out as snakes widen. Bodies come from the shared trail geometry, so they are drawn where the
// server collides them; between ticks each head glides on along its heading.
import { boosting, head, slicesOf, speedOf, TICK_MS, WALL_HOLD_MS, width, type Snake } from "../game/rules";
import type { Replica } from "../game/replica";
import { settings, touch } from "./profile";
import { dogeHead, paintBody } from "./skins";

const OUTSIDE = "#1c140e";
const FLOOR = "#3a2e24"; // a dark wooden board
const SEAM = "rgba(0,0,0,0.22)"; // plank seams, every 4 u
const VIEW_U2 = 3500; // world area on screen for a new snake, on any device
const WARN_MS = 5_000; // the minimap's wall blinks this long before it closes
const MAP_R = 44; // minimap radius, px
const DEATH_HOLD_MS = 1200;
const WIN_ZOOM = 1.6; // the camera moves in on the winner at the end
export const STICK_PX = 60; // touch steering: the anchor follows a finger farther than this

/** Snake colors by id. The first 8 are the Rust game's palette. */
const PALETTE: [number, number, number][] = [
  [232, 176, 74], [102, 217, 128], [242, 89, 89], [89, 140, 250], [242, 115, 217],
  [51, 242, 242], [250, 153, 38], [191, 166, 250], [235, 235, 235], [176, 122, 76],
];
export const snakeColor = (id: number, k = 1) => {
  const [r, g, b] = PALETTE[id % PALETTE.length];
  return `rgb(${(r * k) | 0},${(g * k) | 0},${(b * k) | 0})`;
};

const coin = Object.assign(new Image(), { src: "/coin.png" });
export const font = (size: number) => `bold ${Math.round(size)}px "Comic Neue", "Comic Sans MS", cursive`;

// effects live in world units, like the snakes
interface Particle { x: number; y: number; vx: number; vy: number; age: number; life: number; size: number; color: string }
interface Pop { x: number; y: number; age: number; text: string; color: string }
const CRUMBS = ["#f0cf8a", "#c98a3f"]; // bread crumb, crust
const POP_S = 0.9;

export interface Input {
  steer: number | null; // my heading input
  boost: boolean;
  stick: { id: number; ax: number; ay: number; x: number; y: number } | null; // touch: pointer, anchor and finger, px
  watch: number | null; // the snake the camera follows once mine is dead (or I only watch)
}

export function createRenderer(canvas: HTMLCanvasElement) {
  const ctx = canvas.getContext("2d")!;
  let cam = { x: 0, y: 0, k: 10 }; // k: px per u
  let framed: object | null = null; // the match the camera last jumped to
  let last = performance.now();
  let seenAlive = -Infinity; // last frame my snake was alive
  const parts: Particle[] = [];
  const pops: Pop[] = [];
  let shake = 0; // px, decays every frame
  const sl: number[] = [];
  const sp: number[] = []; // the same slices on screen
  // lite mode: a slow device (mean frame over 28 ms for 2 s) drops to 1x pixels and half the particles
  let lite = false;
  let frames = 0;
  let frameTime = 0;

  const spray = (x: number, y: number, n: number, speed: number, life: number, size: number, colors: string[]) => {
    for (let i = 0; i < (lite ? n / 2 : n); i++) {
      const a = Math.random() * 2 * Math.PI;
      const v = speed * (0.3 + 0.7 * Math.random());
      parts.push({
        x, y, vx: Math.cos(a) * v, vy: Math.sin(a) * v,
        age: 0, life: life * (0.6 + 0.4 * Math.random()), size, color: colors[i % colors.length],
      });
    }
  };

  const resize = () => {
    const dpr = lite ? 1 : Math.min(devicePixelRatio || 1, 2);
    canvas.width = innerWidth * dpr;
    canvas.height = innerHeight * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.imageSmoothingEnabled = false;
  };
  addEventListener("resize", resize);
  resize();

  function draw(r: Replica, input: Input, sinceTick: number, now: number) {
    const dt = Math.min(0.1, (now - last) / 1000);
    last = now;
    if (!lite && document.visibilityState === "visible") {
      frames++;
      frameTime += dt;
      if (frameTime >= 2) {
        lite = frameTime / frames > 0.028;
        if (lite) resize();
        frames = frameTime = 0;
      }
    }
    const cw = innerWidth;
    const ch = innerHeight;
    const m = r.m;
    if (!m) return void ctx.clearRect(0, 0, cw, ch); // menus show the page background
    ctx.fillStyle = OUTSIDE;
    ctx.fillRect(0, 0, cw, ch);

    // between ticks, live heads glide on along their heading
    const since = r.phase === "playing" && !m.over ? Math.min(sinceTick, TICK_MS) / 1000 : 0;
    const glide = (s: Snake): [number, number] => {
      const [x, y] = head(s);
      const d = s.alive ? since * speedOf(s) : 0;
      return [x + Math.cos(s.a) * d, y + Math.sin(s.a) * d];
    };

    // camera: my snake, held on my crumbs a moment after I die, then the snake I watch (else the
    // longest); close on the winner at the end, or the whole arena after a draw
    const me = r.seat >= 0 ? m.snakes[r.seat] : undefined;
    if (me?.alive) seenAlive = now;
    const leader = m.snakes.filter((s) => s.alive).sort((a, b) => b.len - a.len)[0];
    const champ = m.over && m.winner >= 0 ? m.snakes[m.winner] : undefined;
    const watched = input.watch !== null ? m.snakes[input.watch] : undefined;
    const focus = champ ??
      (me?.alive || (me && now - seenAlive < DEATH_HOLD_MS) ? me : watched?.alive ? watched : leader);
    let tx = 0;
    let ty = 0;
    let tk = Math.min(cw, ch) / (2 * m.r0 + 4);
    if (focus && !(m.over && !champ)) {
      [tx, ty] = glide(focus);
      tk = (Math.sqrt((cw * ch) / VIEW_U2) / (1 + 0.3 * (width(focus.len) - 1))) * (champ ? WIN_ZOOM : 1);
    }
    if (framed !== m) {
      framed = m;
      cam = { x: tx, y: ty, k: tk };
    }
    const kp = 1 - Math.exp(-dt * 10);
    const kz = 1 - Math.exp(-dt * 3);
    cam = { x: cam.x + (tx - cam.x) * kp, y: cam.y + (ty - cam.y) * kp, k: cam.k + (tk - cam.k) * kz };
    shake *= Math.exp(-dt * 10);
    const k = cam.k;
    const ox = cw / 2 + (Math.random() * 2 - 1) * shake - cam.x * k;
    const oy = ch / 2 + (Math.random() * 2 - 1) * shake + cam.y * k;
    const X = (x: number) => ox + x * k;
    const Y = (y: number) => oy - y * k;

    // the arena: a wooden board inside the wall (planks 4 u wide, joints staggered every 16 u); the
    // wall glows red once it closes in
    ctx.fillStyle = FLOOR;
    ctx.beginPath();
    ctx.arc(X(0), Y(0), m.wall * k, 0, 2 * Math.PI);
    ctx.fill();
    if (k >= 4) {
      ctx.save();
      ctx.clip();
      ctx.fillStyle = SEAM;
      for (let y = Math.floor((cam.y + ch / 2 / k) / 4) * 4; Y(y) < ch; y -= 4) {
        ctx.fillRect(0, Y(y), cw, 2);
        const shift = (y / 4) % 2 ? 8 : 0;
        for (let x = Math.floor((cam.x - cw / 2 / k - shift) / 16) * 16 + shift; X(x) < cw; x += 16) {
          ctx.fillRect(X(x), Y(y), 2, 4 * k);
        }
      }
      ctx.restore();
    }
    const closing = m.timeMs >= WALL_HOLD_MS && !m.over;
    ctx.strokeStyle = closing ? `rgba(255,70,70,${0.6 + 0.3 * Math.sin(now / 120)})` : "#6b4423";
    ctx.lineWidth = closing ? 4 : 3;
    ctx.beginPath();
    ctx.arc(X(0), Y(0), m.wall * k, 0, 2 * Math.PI);
    ctx.stroke();

    const pulse = 0.85 + 0.15 * Math.sin(now / 150);
    for (const c of m.crumbs) {
      const x = X(c.x);
      const y = Y(c.y);
      if (x < -20 || y < -20 || x > cw + 20 || y > ch + 20) continue;
      if (c.v > 1) {
        const s = 1.2 * k * pulse;
        if (coin.complete) ctx.drawImage(coin, x - s / 2, y - s / 2, s, s);
      } else {
        const s = Math.max(2, 0.5 * k);
        ctx.fillStyle = CRUMBS[c.id % 2];
        ctx.fillRect(x - s / 2, y - s / 2, s, s);
      }
    }

    // bodies in their food skin, outlined in the snake's color; mine is drawn last, on top
    for (const s of [...m.snakes.filter((s) => s !== me), ...(me ? [me] : [])]) {
      if (!s.alive) continue; // dead snakes burst into crumbs instead
      const [hx, hy] = glide(s);
      slicesOf(s.trail, s.len, sl, hx, hy);
      sp.length = sl.length;
      for (let i = 0; i < sl.length; i += 2) {
        sp[i] = X(sl[i]);
        sp[i + 1] = Y(sl[i + 1]);
      }
      const w = width(s.len) * k;
      paintBody(ctx, sp, w, s.skin, snakeColor(s.id), boosting(s) ? 0.25 + 0.15 * Math.sin(now / 50) : 0);
      const size = w * 1.6;
      if (dogeHead.complete) ctx.drawImage(dogeHead, X(hx) - size / 2, Y(hy) - size / 2, size, size);
    }

    const drag = Math.exp(-dt * 3);
    for (let i = parts.length - 1; i >= 0; i--) {
      const p = parts[i];
      p.age += dt;
      if (p.age >= p.life) {
        parts[i] = parts[parts.length - 1]; // already updated this frame
        parts.pop();
        continue;
      }
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.vx *= drag;
      p.vy *= drag;
      const size = p.size * k;
      ctx.globalAlpha = 1 - p.age / p.life;
      ctx.fillStyle = p.color;
      ctx.fillRect(X(p.x) - size / 2, Y(p.y) - size / 2, size, size);
    }
    ctx.globalAlpha = 1;

    // names over other human snakes (mine would hide my reward pops); my head gets a ring, and an
    // arrow shows where I steer at once, before the server turns the snake
    ctx.font = font(Math.max(11, Math.min(15, k * 1.4)));
    ctx.textAlign = "center";
    for (const s of m.snakes) {
      if (!s.alive || !s.human || s.id === r.seat) continue;
      const [hx, hy] = glide(s);
      const x = X(hx);
      const y = Y(hy) - width(s.len) * k - 4;
      ctx.fillStyle = "rgba(0,0,0,0.6)";
      ctx.fillText(s.name, x + 1, y + 1);
      ctx.fillStyle = snakeColor(s.id);
      ctx.fillText(s.name, x, y);
    }
    if (me?.alive) {
      const [hx, hy] = glide(me);
      const w = width(me.len) * k;
      ctx.strokeStyle = "#fff";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(X(hx), Y(hy), w * 0.95, 0, 2 * Math.PI);
      ctx.stroke();
      // "you" over my head while the round starts, so I find myself among the snakes
      if (r.phase === "countdown" || m.timeMs < 2500) {
        const y = Y(hy) - w * 1.4 - 10 - 4 * Math.abs(Math.sin(now / 200));
        ctx.font = font(18);
        ctx.lineWidth = 4;
        ctx.strokeStyle = "rgba(0,0,0,0.7)";
        ctx.strokeText("you ▼", X(hx), y);
        ctx.fillStyle = "#fff";
        ctx.fillText("you ▼", X(hx), y);
      }
      if (input.steer !== null) {
        const a = -input.steer; // screen y points down
        const mx = X(hx) + Math.cos(a) * (w + 12);
        const my = Y(hy) + Math.sin(a) * (w + 12);
        ctx.fillStyle = "#fff";
        ctx.beginPath();
        ctx.moveTo(mx + Math.cos(a) * 7, my + Math.sin(a) * 7);
        ctx.lineTo(mx + Math.cos(a + 2.4) * 7, my + Math.sin(a + 2.4) * 7);
        ctx.lineTo(mx + Math.cos(a - 2.4) * 7, my + Math.sin(a - 2.4) * 7);
        ctx.fill();
      }
    }
    if (champ) {
      const [hx, hy] = head(champ);
      const w = width(champ.len) * k;
      ctx.font = `${Math.round(w * 1.5)}px sans-serif`;
      ctx.fillText("👑", X(hx), Y(hy) - w * 0.8);
    }
    if (input.stick) {
      const { ax, ay, x, y } = input.stick;
      ctx.strokeStyle = input.boost ? "rgba(232,176,74,0.8)" : "rgba(255,255,255,0.25)";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(ax, ay, STICK_PX, 0, 2 * Math.PI);
      ctx.stroke();
      ctx.fillStyle = "rgba(255,255,255,0.35)";
      ctx.beginPath();
      ctx.arc(x, y, 14, 0, 2 * Math.PI);
      ctx.fill();
    }

    // other players off screen: an arrow at the edge, in their color, with their initials
    if (!m.over) {
      for (const s of m.snakes) {
        if (!s.alive || !s.human || s === me) continue;
        const [hx, hy] = glide(s);
        const dx = X(hx) - cw / 2;
        const dy = Y(hy) - ch / 2;
        const hw = cw / 2 - 22;
        const hh = ch / 2 - 22;
        const f = Math.max(Math.abs(dx) / hw, Math.abs(dy) / hh);
        if (f <= 1) continue;
        const x = cw / 2 + dx / f;
        const y = ch / 2 + dy / f;
        const a = Math.atan2(dy, dx);
        ctx.fillStyle = snakeColor(s.id);
        ctx.beginPath();
        ctx.moveTo(x + Math.cos(a) * 12, y + Math.sin(a) * 12);
        ctx.lineTo(x + Math.cos(a + 2.3) * 10, y + Math.sin(a + 2.3) * 10);
        ctx.lineTo(x + Math.cos(a - 2.3) * 10, y + Math.sin(a - 2.3) * 10);
        ctx.fill();
        ctx.font = font(12);
        ctx.fillText(s.name.split(" ").map((w) => w[0]).join(""), x - Math.cos(a) * 14, y - Math.sin(a) * 14 + 4);
      }
    }

    // reward text floats up and fades; a dark outline keeps it readable over any snake
    ctx.font = font(Math.max(14, k * 1.6));
    ctx.lineWidth = 3;
    ctx.lineJoin = "round";
    ctx.strokeStyle = "rgba(0,0,0,0.7)";
    for (let i = pops.length - 1; i >= 0; i--) {
      const p = pops[i];
      p.age += dt;
      if (p.age >= POP_S) {
        pops.splice(i, 1);
        continue;
      }
      const x = X(p.x);
      const y = Y(p.y + p.age * 3);
      ctx.globalAlpha = Math.min(1, (POP_S - p.age) * 5);
      ctx.strokeText(p.text, x, y);
      ctx.fillStyle = p.color;
      ctx.fillText(p.text, x, y);
    }
    ctx.globalAlpha = 1;
    if (!m.over) minimap(m, me, cw, ch, now);
  }

  /** The whole arena in a corner: the wall, and a dot per head (players bigger, mine white). */
  function minimap(m: NonNullable<Replica["m"]>, me: Snake | undefined, cw: number, ch: number, now: number) {
    const cx = touch ? 12 + MAP_R : cw - 12 - MAP_R; // phones: the boost button takes the right corner
    const cy = ch - 12 - MAP_R - (touch ? 8 : 0);
    const s = MAP_R / m.r0;
    ctx.fillStyle = "rgba(28,20,14,0.7)";
    ctx.strokeStyle = "rgba(255,243,214,0.5)";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(cx, cy, MAP_R + 3, 0, 2 * Math.PI);
    ctx.fill();
    ctx.stroke();
    const warn = m.timeMs >= WALL_HOLD_MS - WARN_MS;
    if (!warn || m.timeMs >= WALL_HOLD_MS || Math.sin(now / 100) > 0) {
      ctx.strokeStyle = warn ? "#ff4646" : "rgba(255,243,214,0.35)";
      ctx.beginPath();
      ctx.arc(cx, cy, m.wall * s, 0, 2 * Math.PI);
      ctx.stroke();
    }
    for (const sn of m.snakes) {
      if (!sn.alive) continue;
      const [x, y] = head(sn);
      ctx.fillStyle = sn === me ? "#fff" : snakeColor(sn.id);
      ctx.beginPath();
      ctx.arc(cx + x * s, cy - y * s, sn === me ? 3.5 : sn.human ? 3 : 2, 0, 2 * Math.PI);
      ctx.fill();
    }
  }

  return {
    draw,
    /** A dead snake bursts into crumbs: a few per slice, more from the head. */
    burst: (s: Snake, color: string) => {
      slicesOf(s.trail, s.len, sl);
      const size = 0.35 * width(s.len);
      for (let i = 0; i < sl.length; i += 2) spray(sl[i], sl[i + 1], i ? 2 : 12, i ? 4 : 8, 0.9, size, [color, ...CRUMBS]);
    },
    sparkle: (x: number, y: number) => spray(x, y, 6, 5, 0.4, 0.25, ["#ffd96a", "#fff6c8"]),
    /** Confetti for the winner, in gold, white and its own color. */
    cheer: (s: Snake) => spray(...head(s), 80, 22, 2.2, 0.5, ["#ffd96a", "#fff", snakeColor(s.id)]),
    pop: (x: number, y: number, text: string, color: string) => void pops.push({ x, y: y + 1, age: 0, text, color }),
    /** Strongest request wins; it decays over about a quarter second. Off in the settings: none. */
    shake: (px: number) => void (settings.shake && (shake = Math.max(shake, px))),
  };
}
