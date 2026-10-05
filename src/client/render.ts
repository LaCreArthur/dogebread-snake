// Canvas drawing. The camera follows my snake (or, a moment after I die, the longest one) and zooms
// out as snakes widen. Bodies come from the shared trail geometry, so they are drawn where the
// server collides them; between ticks each head glides on along its heading.
import { boosting, head, slicesOf, speedOf, TICK_MS, WALL_HOLD_MS, width, type Snake } from "../game/rules";
import type { Replica } from "../game/replica";

const OUTSIDE = "#120f1f";
const FLOOR = "#23233d";
const GRID = "rgba(255,255,255,0.04)";
const VIEW_U2 = 3500; // world area on screen for a new snake, on any device
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

const image = (src: string) => Object.assign(new Image(), { src });
const dogeHead = image("/doge_head.png");
const coin = image("/coin.png");
const font = (size: number) => `bold ${Math.round(size)}px "Comic Sans MS", "Chalkboard SE", cursive`;

// effects live in world units, like the snakes
interface Particle { x: number; y: number; vx: number; vy: number; age: number; life: number; size: number; color: string }
interface Pop { x: number; y: number; age: number; text: string; color: string }
const CRUMBS = ["#f0cf8a", "#c98a3f"]; // bread crumb, crust
const POP_S = 0.9;

export interface Input {
  steer: number | null; // my heading input
  boost: boolean;
  stick: { id: number; ax: number; ay: number; x: number; y: number } | null; // touch: pointer, anchor and finger, px
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

  const spray = (x: number, y: number, n: number, speed: number, life: number, size: number, colors: string[]) => {
    for (let i = 0; i < n; i++) {
      const a = Math.random() * 2 * Math.PI;
      const v = speed * (0.3 + 0.7 * Math.random());
      parts.push({
        x, y, vx: Math.cos(a) * v, vy: Math.sin(a) * v,
        age: 0, life: life * (0.6 + 0.4 * Math.random()), size, color: colors[i % colors.length],
      });
    }
  };

  const resize = () => {
    const dpr = Math.min(devicePixelRatio || 1, 2);
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
    const cw = innerWidth;
    const ch = innerHeight;
    ctx.fillStyle = OUTSIDE;
    ctx.fillRect(0, 0, cw, ch);
    const m = r.m;
    if (!m) return;

    // between ticks, live heads glide on along their heading
    const since = r.phase === "playing" && !m.over ? Math.min(sinceTick, TICK_MS) / 1000 : 0;
    const glide = (s: Snake): [number, number] => {
      const [x, y] = head(s);
      const d = s.alive ? since * speedOf(s) : 0;
      return [x + Math.cos(s.a) * d, y + Math.sin(s.a) * d];
    };

    // camera: my snake, held on my crumbs a moment after I die, then the longest; close on the winner
    // at the end, or the whole arena after a draw
    const me = r.seat >= 0 ? m.snakes[r.seat] : undefined;
    if (me?.alive) seenAlive = now;
    const leader = m.snakes.filter((s) => s.alive).sort((a, b) => b.len - a.len)[0];
    const champ = m.over && m.winner >= 0 ? m.snakes[m.winner] : undefined;
    const focus = champ ?? (me?.alive || (me && now - seenAlive < DEATH_HOLD_MS) ? me : leader);
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

    // the arena: floor inside the wall with a faint grid; the wall glows red once it closes in
    ctx.fillStyle = FLOOR;
    ctx.beginPath();
    ctx.arc(X(0), Y(0), m.wall * k, 0, 2 * Math.PI);
    ctx.fill();
    if (k >= 4) {
      ctx.save();
      ctx.clip();
      ctx.fillStyle = GRID;
      for (let x = Math.ceil((cam.x - cw / 2 / k) / 4) * 4; X(x) < cw; x += 4) ctx.fillRect(X(x), 0, 1, ch);
      for (let y = Math.floor((cam.y + ch / 2 / k) / 4) * 4; Y(y) < ch; y -= 4) ctx.fillRect(0, Y(y), cw, 1);
      ctx.restore();
    }
    const closing = m.timeMs >= WALL_HOLD_MS && !m.over;
    ctx.strokeStyle = closing ? `rgba(255,70,70,${0.6 + 0.3 * Math.sin(now / 120)})` : "#3a3a66";
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

    // bodies as one round stroke, crossed by bread-slice lines; mine is drawn last, on top
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    for (const s of [...m.snakes.filter((s) => s !== me), ...(me ? [me] : [])]) {
      if (!s.alive) continue; // dead snakes burst into crumbs instead
      const [hx, hy] = glide(s);
      slicesOf(s.trail, s.len, sl, hx, hy);
      const w = width(s.len) * k;
      ctx.beginPath();
      ctx.moveTo(X(sl[0]), Y(sl[1]));
      for (let i = 2; i < sl.length; i += 2) ctx.lineTo(X(sl[i]), Y(sl[i + 1]));
      if (boosting(s)) {
        ctx.strokeStyle = `rgba(255,255,255,${0.25 + 0.15 * Math.sin(now / 50)})`;
        ctx.lineWidth = w + 8;
        ctx.stroke();
      }
      ctx.strokeStyle = snakeColor(s.id, 0.45);
      ctx.lineWidth = w + 2;
      ctx.stroke();
      ctx.strokeStyle = snakeColor(s.id, 0.8);
      ctx.lineWidth = w;
      ctx.stroke();
      ctx.beginPath();
      for (let i = 4; i + 3 < sl.length; i += 4) {
        const dx = X(sl[i + 2]) - X(sl[i - 2]);
        const dy = Y(sl[i + 3]) - Y(sl[i - 1]);
        const d = Math.hypot(dx, dy) || 1;
        const px = (-dy / d) * w * 0.38;
        const py = (dx / d) * w * 0.38;
        ctx.moveTo(X(sl[i]) - px, Y(sl[i + 1]) - py);
        ctx.lineTo(X(sl[i]) + px, Y(sl[i + 1]) + py);
      }
      ctx.strokeStyle = snakeColor(s.id, 0.55);
      ctx.lineWidth = Math.max(1, w * 0.12);
      ctx.stroke();
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
    /** Strongest request wins; it decays over about a quarter second. */
    shake: (px: number) => void (shake = Math.max(shake, px)),
  };
}
