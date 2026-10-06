// Overlays on the arena: the top bar, first-match hints, the death card, the watch bar, stickers,
// toasts and the kill feed. Plain DOM; main.ts decides when each shows.
import { STICKERS } from "../game/protocol";
import type { Replica } from "../game/replica";
import { $, clock, h } from "./dom";
import { snakeColor } from "./render";

const hud = $("hud");
const feed = $("feed");
const hintEl = $("hint");
const death = $("death");
const watchBar = $("watch");
const stickers = $("stickers");
const toastEl = $("toast");

// the top bar is built once per match, then only its numbers and dots change
let hudFor: object | null = null;
const chips = { time: h("b"), size: h("b"), bonks: h("b") };
const sizeChips = [h("span", {}, "🍞 ", chips.size), h("span", {}, "💥 ", chips.bonks)];
const watchChip = h("span", {}, "👀 watching");
let dots: HTMLElement[] = [];

export function updateHud(r: Replica) {
  const m = r.m;
  hud.hidden = !m || (r.phase !== "countdown" && r.phase !== "playing");
  if (hud.hidden || !m) return;
  if (hudFor !== m) {
    hudFor = m;
    dots = m.snakes.map((s) => {
      const d = h("i", { title: s.name });
      d.style.background = snakeColor(s.id);
      return d;
    });
    hud.replaceChildren(
      h("div", { className: "chips" }, h("span", {}, "⏱ ", chips.time), ...sizeChips, watchChip),
      h("div", { className: "roster" }, ...dots),
    );
  }
  const me = r.seat >= 0 ? m.snakes[r.seat] : undefined;
  chips.time.textContent = clock(m.timeMs / 1000);
  chips.size.textContent = String(Math.floor(me?.len ?? 0));
  chips.bonks.textContent = String(me?.kills ?? 0);
  for (const c of sizeChips) c.hidden = !me?.alive;
  watchChip.hidden = !!me?.alive;
  m.snakes.forEach((s, i) => (dots[i].className = (s.alive ? "" : "dead ") + (s === me ? "me" : "")));
}

export function addFeed(text: string, color: string) {
  const el = h("div", { textContent: text });
  el.style.color = color;
  feed.prepend(el);
  while (feed.children.length > 5) feed.lastChild!.remove();
  setTimeout(() => (el.style.opacity = "0"), 3500);
  setTimeout(() => el.remove(), 4000);
}
export const clearFeed = () => feed.replaceChildren();

let toastTimer = 0;
export function toast(text: string) {
  toastEl.textContent = text;
  toastEl.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => toastEl.classList.remove("show"), 2500);
}

/** A sticker floats up the left edge for a few seconds; at most 6 on screen. */
export function showSticker(from: string, k: number, color: string) {
  const name = h("span", { textContent: `${from}: ` });
  name.style.color = color;
  const el = h("div", {}, name, STICKERS[k] ?? "");
  stickers.prepend(el);
  while (stickers.children.length > 6) stickers.lastChild!.remove();
  setTimeout(() => el.remove(), 3000);
}

/** The six sticker buttons; after a send they rest 2 s (the server drops faster ones). */
export function stickerRow(send: (k: number) => void) {
  const row = h("div", { className: "stickers-row" });
  const buttons = STICKERS.map((s, k) => h("button", {
    textContent: s.split(" ")[0],
    title: s,
    onclick: () => {
      send(k);
      for (const b of buttons) b.disabled = true;
      setTimeout(() => buttons.forEach((b) => (b.disabled = false)), 2000);
    },
  }));
  row.append(...buttons);
  return row;
}

let shownHint = "";
export function hint(text: string) {
  if (text === shownHint) return;
  shownHint = text;
  hintEl.textContent = text;
  hintEl.classList.toggle("show", !!text);
}

let deathTimer = 0;
/** The death card: big title, then the cause and the numbers; it gives way to the watch bar. */
export function showDeath(cause: Node, numbers: string, then: () => void) {
  death.replaceChildren(h("h2", { textContent: "wow. such ded." }), h("p", {}, cause), h("p", { className: "muted", textContent: numbers }));
  death.classList.add("show");
  clearTimeout(deathTimer);
  deathTimer = window.setTimeout(() => {
    death.classList.remove("show");
    then();
  }, 2500);
}
export function hideDeath() {
  clearTimeout(deathTimer);
  death.classList.remove("show");
}

const watchText = h("p");
let watchRow: HTMLElement | null = null;
/** The bottom bar while I watch: who the camera follows, and the stickers. null hides it. */
export function setWatch(text: (Node | string)[] | null, send: (k: number) => void) {
  watchBar.hidden = !text;
  if (!text) return;
  watchRow ??= stickerRow(send);
  watchText.replaceChildren(...text);
  if (!watchBar.children.length) watchBar.append(watchText, watchRow);
}
