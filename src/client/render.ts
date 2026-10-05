// Canvas drawing. Cells are at least MIN_CELL px (the Rust game used 12): where the arena fits the
// screen it is framed whole, otherwise the camera follows my snake (or the leader once I am out).
// Grid +y is up; screen y is down.
import { cx, cy, DX, DY, H, shrinkWarning, W, type Dir } from "../game/rules";
import type { Replica } from "../game/replica";

const BG = "#1a1a2e";
const WALL = "#2d2142";
const FLOOR = "#23233d";
const GRID = "rgba(255,255,255,0.035)";
export const HUD_HEIGHT = 44;
const MIN_CELL = 12;
const PAD = 8;

/** Snake colors by seat. The first 8 are the Rust game's palette. */
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

// effects live in grid units (+y up), like the snakes
interface Particle { x: number; y: number; vx: number; vy: number; age: number; life: number; size: number; color: string }
interface Pop { x: number; y: number; age: number; text: string; color: string }
const CRUMBS = ["#f0cf8a", "#c98a3f"]; // bread crumb, crust
const POP_S = 0.9;
const DEATH_HOLD_MS = 1200;

export function createRenderer(canvas: HTMLCanvasElement) {
  const ctx = canvas.getContext("2d")!;
  let cam = { x: W / 2, y: H / 2, cs: MIN_CELL };
  let framed: object | null = null; // the match the camera last jumped to
  let last = performance.now();
  const parts: Particle[] = [];
  const pops: Pop[] = [];
  let shake = 0; // px, decays every frame
  let seenAlive = -Infinity; // last frame my snake was alive

  const spray = (c: number, n: number, speed: number, life: number, size: number, colors: string[]) => {
    for (let i = 0; i < n; i++) {
      const a = Math.random() * 2 * Math.PI;
      const v = speed * (0.3 + 0.7 * Math.random());
      parts.push({
        x: cx(c) + 0.5, y: cy(c) + 0.5, vx: Math.cos(a) * v, vy: Math.sin(a) * v,
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

  function draw(r: Replica, pending: Dir | -1, now: number) {
    const dt = Math.min(0.1, (now - last) / 1000);
    last = now;
    const cw = innerWidth;
    const ch = innerHeight;
    ctx.fillStyle = BG;
    ctx.fillRect(0, 0, cw, ch);
    const m = r.m;
    if (!m) return;

    const [bx0, by0, bx1, by1] = m.bounds;
    // target: the arena with its walls, framed whole if it fits at MIN_CELL, else centered on the focus
    const ax0 = bx0 - 1, ay0 = by0 - 1, ax1 = bx1 + 1, ay1 = by1 + 1;
    const aw = cw - PAD * 2;
    const ah = ch - HUD_HEIGHT - PAD * 2;
    const over = r.phase === "over";
    const fit = Math.min(aw / (ax1 - ax0), ah / (ay1 - ay0));
    const tcs = over ? fit : Math.max(MIN_CELL, fit);
    const me = r.seat >= 0 ? m.snakes[r.seat] : undefined;
    const leader = m.snakes.filter((s) => s.alive).sort((a, b) => b.score - a.score)[0];
    // after my death the camera stays on my crumbs for a moment, then follows the leader
    if (me?.alive) seenAlive = now;
    const focus = (me?.alive || now - seenAlive < DEATH_HOLD_MS ? me : leader)?.body[0];
    const clamp = (v: number, lo: number, hi: number) => (lo > hi ? (lo + hi) / 2 : Math.min(hi, Math.max(lo, v)));
    const tx = clamp(focus === undefined ? W / 2 : cx(focus) + 0.5, ax0 + aw / 2 / tcs, ax1 - aw / 2 / tcs);
    const ty = clamp(focus === undefined ? H / 2 : cy(focus) + 0.5, ay0 + ah / 2 / tcs, ay1 - ah / 2 / tcs);
    if (framed !== m) {
      framed = m;
      cam = { x: tx, y: ty, cs: tcs };
    }
    const kp = 1 - Math.exp(-dt * 5);
    const kz = 1 - Math.exp(-dt * 3);
    cam = { x: cam.x + (tx - cam.x) * kp, y: cam.y + (ty - cam.y) * kp, cs: cam.cs + (tcs - cam.cs) * kz };
    const cs = cam.cs;
    shake *= Math.exp(-dt * 10);
    const sx = cw / 2 + (Math.random() * 2 - 1) * shake;
    const sy = HUD_HEIGHT + PAD + ah / 2 + (Math.random() * 2 - 1) * shake;
    const px = (x: number) => sx + (x - cam.x) * cs;
    const wy = (y: number) => sy - (y - cam.y) * cs;
    const py = (y: number) => wy(y + 1); // top edge of row y
    const sprite = (img: HTMLImageElement, c: number, size: number) => {
      if (img.complete) ctx.drawImage(img, px(cx(c)) + (cs - size) / 2, py(cy(c)) + (cs - size) / 2, size, size);
    };

    ctx.fillStyle = WALL;
    ctx.fillRect(px(0), py(H - 1), W * cs, H * cs);
    ctx.fillStyle = FLOOR;
    ctx.fillRect(px(bx0), py(by1 - 1), (bx1 - bx0) * cs, (by1 - by0) * cs);
    if (cs >= 6) {
      ctx.fillStyle = GRID;
      for (let x = bx0 + 1; x < bx1; x++) ctx.fillRect(px(x), py(by1 - 1), 1, (by1 - by0) * cs);
      for (let y = by0 + 1; y < by1; y++) ctx.fillRect(px(bx0), py(y - 1), (bx1 - bx0) * cs, 1);
    }

    // the ring that is about to become wall flashes red
    if (r.phase === "playing" && !m.over && shrinkWarning(m)) {
      ctx.fillStyle = `rgba(255,70,70,${0.3 + 0.2 * Math.sin(now / 70)})`;
      const w = (bx1 - bx0) * cs;
      const h = (by1 - by0) * cs;
      ctx.fillRect(px(bx0), py(by1 - 1), w, cs);
      ctx.fillRect(px(bx0), py(by0), w, cs);
      ctx.fillRect(px(bx0), py(by1 - 1), cs, h);
      ctx.fillRect(px(bx1 - 1), py(by1 - 1), cs, h);
    }

    const pulse = 0.85 + 0.15 * Math.sin(now / 150);
    for (const f of m.food) sprite(coin, f, cs * pulse);

    const gap = cs >= 6 ? 1 : 0;
    for (const s of m.snakes) {
      if (!s.alive) continue; // dead snakes burst into crumbs instead
      ctx.fillStyle = snakeColor(s.id, 0.7);
      for (let i = s.body.length - 1; i >= 1; i--) {
        const c = s.body[i];
        ctx.fillRect(px(cx(c)) + gap, py(cy(c)) + gap, cs - gap * 2, cs - gap * 2);
      }
      sprite(dogeHead, s.body[0], cs * 1.3);
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
      const size = p.size * cs;
      ctx.globalAlpha = 1 - p.age / p.life;
      ctx.fillStyle = p.color;
      ctx.fillRect(px(p.x) - size / 2, wy(p.y) - size / 2, size, size);
    }
    ctx.globalAlpha = 1;

    // names over other human snakes (mine would hide my reward pops); my head gets a ring and my
    // queued turn an arrow
    ctx.font = font(Math.max(10, Math.min(14, cs)));
    ctx.textAlign = "center";
    for (const s of m.snakes) {
      if (!s.alive || !s.human || s.id === r.seat) continue;
      const x = px(cx(s.body[0])) + cs / 2;
      const y = py(cy(s.body[0])) - cs * 0.4;
      ctx.fillStyle = "rgba(0,0,0,0.6)";
      ctx.fillText(s.name, x + 1, y + 1);
      ctx.fillStyle = snakeColor(s.id);
      ctx.fillText(s.name, x, y);
    }
    if (me?.alive) {
      const hx = px(cx(me.body[0]));
      const hy = py(cy(me.body[0]));
      ctx.strokeStyle = "#fff";
      ctx.lineWidth = 2;
      ctx.strokeRect(hx - cs * 0.2, hy - cs * 0.2, cs * 1.4, cs * 1.4);
      if (pending !== -1) {
        const mx = hx + cs / 2 + DX[pending] * cs * 1.4;
        const my = hy + cs / 2 - DY[pending] * cs * 1.4;
        const a = Math.atan2(-DY[pending], DX[pending]);
        const s = Math.max(5, cs * 0.6);
        ctx.fillStyle = "#fff";
        ctx.beginPath();
        ctx.moveTo(mx + Math.cos(a) * s, my + Math.sin(a) * s);
        ctx.lineTo(mx + Math.cos(a + 2.4) * s, my + Math.sin(a + 2.4) * s);
        ctx.lineTo(mx + Math.cos(a - 2.4) * s, my + Math.sin(a - 2.4) * s);
        ctx.fill();
      }
    }

    // reward text floats up and fades; a dark outline keeps it readable over any snake
    ctx.font = font(Math.max(14, cs * 1.2));
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
      const x = px(p.x);
      const y = wy(p.y + p.age * 2);
      ctx.globalAlpha = Math.min(1, (POP_S - p.age) * 5);
      ctx.strokeText(p.text, x, y);
      ctx.fillStyle = p.color;
      ctx.fillText(p.text, x, y);
    }
    ctx.globalAlpha = 1;
  }

  return {
    draw,
    /** A dead snake bursts into crumbs: a handful per slice, more from the head. */
    burst: (body: number[], color: string) =>
      body.forEach((c, i) => spray(c, i ? 3 : 12, i ? 4 : 8, 0.9, 0.35, [color, ...CRUMBS])),
    sparkle: (c: number) => spray(c, 6, 5, 0.4, 0.22, ["#ffd96a", "#fff6c8"]),
    pop: (c: number, text: string, color: string) => void pops.push({ x: cx(c) + 0.5, y: cy(c) + 1, age: 0, text, color }),
    /** Strongest request wins; it decays over about a quarter second. */
    shake: (px: number) => void (shake = Math.max(shake, px)),
  };
}
