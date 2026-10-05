// Canvas drawing. Cells are at least MIN_CELL px (the Rust game used 12): where the arena fits the
// screen it is framed whole, otherwise the camera follows my snake (or the leader once I am out).
// Grid +y is up; screen y is down.
import { canShrink, cx, cy, DX, DY, H, SHRINK_EVERY_MS, SHRINK_WARN_MS, W, type Dir } from "../game/rules";
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

export interface Overlay {
  pending: Dir | -1; // my turn, sent but not applied yet
  deathAt: Map<number, number>; // snake id -> time of death (ms), for the death blink
}

export function createRenderer(canvas: HTMLCanvasElement) {
  const ctx = canvas.getContext("2d")!;
  let cam = { x: W / 2, y: H / 2, cs: MIN_CELL };
  let framed: object | null = null; // the match the camera last jumped to
  let last = performance.now();

  const resize = () => {
    const dpr = Math.min(devicePixelRatio || 1, 2);
    canvas.width = innerWidth * dpr;
    canvas.height = innerHeight * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.imageSmoothingEnabled = false;
  };
  addEventListener("resize", resize);
  resize();

  return function draw(r: Replica, o: Overlay, now: number) {
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
    const focus = (me?.alive ? me : leader)?.body[0];
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
    const sx = cw / 2;
    const sy = HUD_HEIGHT + PAD + ah / 2;
    const px = (x: number) => sx + (x - cam.x) * cs;
    const py = (y: number) => sy - (y + 1 - cam.y) * cs; // top edge of row y
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
    const untilShrink = SHRINK_EVERY_MS - (m.timeMs % SHRINK_EVERY_MS);
    if (r.phase === "playing" && !m.over && canShrink(m.bounds) && untilShrink <= SHRINK_WARN_MS) {
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
      if (!s.alive) {
        const age = now - (o.deathAt.get(s.id) ?? -Infinity);
        if (age > 2000 || Math.floor(age / 120) % 2) continue;
        ctx.globalAlpha = 0.5;
      }
      ctx.fillStyle = s.alive ? snakeColor(s.id, 0.7) : "#888";
      for (let i = s.body.length - 1; i >= 1; i--) {
        const c = s.body[i];
        ctx.fillRect(px(cx(c)) + gap, py(cy(c)) + gap, cs - gap * 2, cs - gap * 2);
      }
      sprite(dogeHead, s.body[0], cs * 1.3);
      ctx.globalAlpha = 1;
    }

    // names over human snakes; my head gets a ring and my queued turn an arrow
    ctx.font = `bold ${Math.round(Math.max(10, Math.min(14, cs)))}px "Comic Sans MS", "Chalkboard SE", cursive`;
    ctx.textAlign = "center";
    for (const s of m.snakes) {
      if (!s.alive || !s.human) continue;
      const x = px(cx(s.body[0])) + cs / 2;
      const y = py(cy(s.body[0])) - cs * 0.4;
      ctx.fillStyle = "rgba(0,0,0,0.6)";
      ctx.fillText(s.name, x + 1, y + 1);
      ctx.fillStyle = s.id === r.seat ? "#fff" : snakeColor(s.id);
      ctx.fillText(s.name, x, y);
    }
    if (me?.alive) {
      const hx = px(cx(me.body[0]));
      const hy = py(cy(me.body[0]));
      ctx.strokeStyle = "#fff";
      ctx.lineWidth = 2;
      ctx.strokeRect(hx - cs * 0.2, hy - cs * 0.2, cs * 1.4, cs * 1.4);
      if (o.pending !== -1) {
        const mx = hx + cs / 2 + DX[o.pending] * cs * 1.4;
        const my = hy + cs / 2 - DY[o.pending] * cs * 1.4;
        const a = Math.atan2(-DY[o.pending], DX[o.pending]);
        const s = Math.max(5, cs * 0.6);
        ctx.fillStyle = "#fff";
        ctx.beginPath();
        ctx.moveTo(mx + Math.cos(a) * s, my + Math.sin(a) * s);
        ctx.lineTo(mx + Math.cos(a + 2.4) * s, my + Math.sin(a + 2.4) * s);
        ctx.lineTo(mx + Math.cos(a - 2.4) * s, my + Math.sin(a - 2.4) * s);
        ctx.fill();
      }
    }
  };
}
