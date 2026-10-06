// Food skins, drawn from code (no art files): a crust stroke, a crumb stroke and a pattern along the
// body. The snake's own color stays as an outline, so players still tell snakes apart by color.

export const dogeHead = Object.assign(new Image(), { src: "/doge_head.png" });

type Detail = (ctx: CanvasRenderingContext2D, p: number[], w: number) => void;
interface Look { crust: string; crumb: string; inner: number; detail: Detail }

/** Calls fn every `every` points with the point and the unit normal of the path there. */
function along(p: number[], every: number, fn: (x: number, y: number, nx: number, ny: number, i: number) => void) {
  for (let i = 2 * every; i + 2 < p.length; i += 2 * every) {
    const dx = p[i + 2] - p[i - 2];
    const dy = p[i + 3] - p[i - 1];
    const d = Math.hypot(dx, dy) || 1;
    fn(p[i], p[i + 1], -dy / d, dx / d, i / 2);
  }
}

const sliceLines = (color: string): Detail => (ctx, p, w) => {
  ctx.beginPath();
  along(p, 2, (x, y, nx, ny) => {
    ctx.moveTo(x - nx * w * 0.4, y - ny * w * 0.4);
    ctx.lineTo(x + nx * w * 0.4, y + ny * w * 0.4);
  });
  ctx.strokeStyle = color;
  ctx.lineWidth = Math.max(1, w * 0.12);
  ctx.stroke();
};

function dots(ctx: CanvasRenderingContext2D, p: number[], w: number, every: number, colors: string[], spread: number) {
  const r = Math.max(1.2, w * 0.1);
  along(p, every, (x, y, nx, ny, i) => {
    const side = i % 2 ? spread : -spread;
    ctx.fillStyle = colors[i % colors.length];
    ctx.beginPath();
    ctx.arc(x + nx * w * side, y + ny * w * side, r, 0, 2 * Math.PI);
    ctx.fill();
  });
}

const LOOKS: Look[] = [
  // toast
  { crust: "#b9762f", crumb: "#f0cf8a", inner: 0.62, detail: sliceLines("#a0652a") },
  // garlic bread: toast with herbs and garlic
  {
    crust: "#c9852f", crumb: "#f3d48c", inner: 0.62,
    detail: (ctx, p, w) => {
      sliceLines("#b07434")(ctx, p, w);
      dots(ctx, p, w, 1, ["#4f8a2b", "#fffaf0", "#3e7a22"], 0.2);
    },
  },
  // baguette: one long crust with diagonal cuts
  {
    crust: "#c27a2c", crumb: "#e3a857", inner: 0.5,
    detail: (ctx, p, w) => {
      ctx.beginPath();
      along(p, 3, (x, y, nx, ny) => {
        const tx = ny; // tangent
        const ty = -nx;
        ctx.moveTo(x - (nx * 0.3 + tx * 0.35) * w, y - (ny * 0.3 + ty * 0.35) * w);
        ctx.lineTo(x + (nx * 0.3 + tx * 0.35) * w, y + (ny * 0.3 + ty * 0.35) * w);
      });
      ctx.strokeStyle = "#f6dc9c";
      ctx.lineWidth = Math.max(1.5, w * 0.16);
      ctx.stroke();
    },
  },
  // taco: a shell with lettuce and tomato at the edges
  {
    crust: "#e0a526", crumb: "#f5cf5e", inner: 0.55,
    detail: (ctx, p, w) => dots(ctx, p, w, 1, ["#6abf3f", "#e0533d", "#6abf3f", "#f2f2e0"], 0.38),
  },
  // hot dog: a bun, the sausage, a mustard zigzag
  {
    crust: "#d08d43", crumb: "#eebb72", inner: 0.8,
    detail: (ctx, p, w) => {
      ctx.beginPath();
      ctx.moveTo(p[0], p[1]);
      for (let i = 2; i < p.length; i += 2) ctx.lineTo(p[i], p[i + 1]);
      ctx.strokeStyle = "#b8432a";
      ctx.lineWidth = w * 0.45;
      ctx.stroke();
      ctx.beginPath();
      along(p, 1, (x, y, nx, ny, i) => {
        const side = i % 2 ? 0.14 : -0.14;
        ctx.lineTo(x + nx * w * side, y + ny * w * side);
      });
      ctx.strokeStyle = "#f7d23e";
      ctx.lineWidth = Math.max(1, w * 0.08);
      ctx.stroke();
    },
  },
];

/** A body along screen points p (x, y pairs, head first), w px wide. glow > 0 adds the boost halo. */
export function paintBody(ctx: CanvasRenderingContext2D, p: number[], w: number, skin: number, outline: string, glow = 0) {
  const look = LOOKS[skin] ?? LOOKS[0];
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.beginPath();
  ctx.moveTo(p[0], p[1]);
  for (let i = 2; i < p.length; i += 2) ctx.lineTo(p[i], p[i + 1]);
  if (glow > 0) {
    ctx.strokeStyle = `rgba(255,255,255,${glow})`;
    ctx.lineWidth = w + 8;
    ctx.stroke();
  }
  ctx.strokeStyle = outline;
  ctx.lineWidth = w + Math.max(3, w * 0.3);
  ctx.stroke();
  ctx.strokeStyle = look.crust;
  ctx.lineWidth = w;
  ctx.stroke();
  ctx.strokeStyle = look.crumb;
  ctx.lineWidth = w * look.inner;
  ctx.stroke();
  look.detail(ctx, p, w);
}

/** A small wiggling snake in a skin, for pickers and seat cards. */
export function swatch(skin: number, outline = "#6b4423", width = 84, height = 32) {
  const c = document.createElement("canvas");
  const dpr = Math.min(devicePixelRatio || 1, 2);
  c.width = width * dpr;
  c.height = height * dpr;
  c.style.width = `${width}px`;
  c.style.height = `${height}px`;
  const ctx = c.getContext("2d")!;
  ctx.scale(dpr, dpr);
  const w = height * 0.42;
  const p: number[] = [];
  const x0 = width - height * 0.5;
  for (let x = x0; x >= w * 0.7; x -= 3) p.push(x, height / 2 + Math.sin((x0 - x) / 9) * height * 0.16);
  const draw = () => {
    ctx.clearRect(0, 0, width, height);
    paintBody(ctx, p, w, skin, outline);
    const s = w * 1.7;
    if (dogeHead.complete) ctx.drawImage(dogeHead, p[0] - s / 2, p[1] - s / 2, s, s);
  };
  draw();
  if (!dogeHead.complete) dogeHead.addEventListener("load", draw, { once: true });
  return c;
}
