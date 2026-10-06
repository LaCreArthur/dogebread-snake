// Menu screens in the center card (home, lobby, results, dead links) and the modal cards (settings,
// how to play, skins). main.ts owns the connection; screens only call back into it.
import { NAME_ADJ, NAME_NOUN, nameOf, validName, type ClientMsg, type NameIdx } from "../game/protocol";
import type { Replica } from "../game/replica";
import { SEATS, TICK_MS } from "../game/rules";
import { $, clock, h } from "./dom";
import { load, mySkin, profile, save, saveProfile, saveSettings, settings, SKINS, touch } from "./profile";
import { snakeColor } from "./render";
import { swatch } from "./skins";
import { music } from "./sound";
import { stickerRow } from "./hud";

const panel = $("panel");
const modal = $("modal");

export interface Act {
  send(msg: ClientMsg): void;
  friends(): void; // make a room and wait in its lobby
  solo(): void; // make a room and start at once with bots
  join(code: string): void;
  home(): void;
}

export const CODE = /^[2-9A-HJKMNP-Z]{5}$/;

// ---- names: generated only; the home screen rolls them here, a room rolls them on the server

export function savedName(): NameIdx | null {
  const n = (load(() => localStorage, "dbs-name") ?? "").split(".").map(Number);
  return validName(n) ? n : null;
}
export const saveName = (n: NameIdx) => save(() => localStorage, "dbs-name", n.join("."));
function rollName(): NameIdx {
  const n: NameIdx = [Math.floor(Math.random() * NAME_ADJ.length), Math.floor(Math.random() * NAME_NOUN.length)];
  saveName(n);
  return n;
}

// ---- the menu background: doge-speak drifting behind the card

const WORDS = ["wow", "such snek", "very bread", "much bonk", "so crunch", "many doge", "plz no wall", "such zoom", "amaze", "very hungry"];
const bg = $("bg");
WORDS.forEach((w, i) => {
  const el = h("span", { textContent: w });
  el.style.left = `${(i * 37 + 7) % 86}%`;
  el.style.top = `${(i * 23 + 5) % 90}%`;
  el.style.color = `hsl(${(i * 67) % 360} 85% 68%)`;
  el.style.animationDelay = `${-i * 1.3}s`;
  el.style.fontSize = `${18 + ((i * 7) % 14)}px`;
  bg.append(el);
});

function show(screen: string, ...kids: Node[]) {
  panel.hidden = false;
  panel.replaceChildren(...kids);
  // a new screen starts at the top; results wait for the winner's moment (style.css). Re-rendering the
  // same screen keeps both, so a late room update does not hide the results again
  if (document.body.dataset.screen !== screen) {
    document.body.dataset.screen = screen;
    panel.classList.toggle("late", screen === "results");
    panel.scrollTop = 0;
  }
  music(settings.music); // menus only
}

/** A short card while something loads. */
export const showWait = (text: string) => show("wait", mascot("big bob"), h("h2", { textContent: text }));

export function hidePanel() {
  panel.hidden = true;
  document.body.dataset.screen = "play";
  music(false);
}

const mascot = (extra = "") => h("img", { className: `mascot ${extra}`, src: "/doge_head.png", alt: "" });

// ---- home: only hosts and solo players see it; a room link opens the room directly

export function showHome(act: Act) {
  let name = savedName() ?? rollName();
  const nameEl = h("b", { textContent: nameOf(name) });
  const skinBox = h("button", { className: "skin-btn", title: "Change skin", onclick: () => openSkins((k) => {
    profile.skin = k;
    saveProfile();
    skinBox.replaceChildren(swatch(mySkin()));
  }) }, swatch(mySkin()));
  const code = h("input", { maxLength: 5, placeholder: "CODE", autocomplete: "off", ariaLabel: "Room code" });
  const join = h("form", { className: "row", hidden: true }, code, h("button", { textContent: "Join" }));
  join.onsubmit = (e) => {
    e.preventDefault();
    const c = code.value.trim().toUpperCase();
    if (CODE.test(c)) act.join(c);
    else code.classList.add("bad");
  };
  show("home",
    h("div", { className: "hero" }, mascot("big"), h("h1", { className: "logo", textContent: "DogeBread Snake" }),
      h("p", { className: "tag", textContent: "such snek. very battle. wow." })),
    h("div", { className: "me" }, skinBox, nameEl, h("button", {
      className: "icon", textContent: "🎲", title: "New name",
      onclick: () => (nameEl.textContent = nameOf((name = rollName()))),
    })),
    h("button", { className: "main", textContent: "Play with friends", onclick: () => act.friends() }),
    h("button", { textContent: "Play now with bots", onclick: () => act.solo() }),
    h("p", { className: "links" },
      h("button", { className: "link", textContent: "got a code?", onclick: () => { join.hidden = false; code.focus(); } }),
      h("button", { className: "link", textContent: "how to play", onclick: openHowTo })),
    join,
  );
}

// ---- dead links: one message and one button that leads somewhere

export function showDeadEnd(title: string, line: string, action: string, onAction: () => void, act: Act) {
  show("dead",
    mascot("big sad"),
    h("h2", { textContent: title }),
    h("p", { className: "muted", textContent: line }),
    h("button", { className: "main", textContent: action, onclick: onAction }),
    h("button", { className: "link", textContent: "home", onclick: () => act.home() }),
  );
}

// ---- the room: the lobby before the first round, the results after each one

let invited = false; // the invite button pulses until it is used once per room
let roomCode = "";

export function showRoom(r: Replica, act: Act, news: string[], dev: Node[]) {
  if (r.code !== roomCode) {
    roomCode = r.code;
    invited = false;
  }
  const me = r.players.find((p) => p.id === r.you);
  const link = `${location.origin}/?r=${r.code}`;
  const invite = h("button", {
    className: invited ? "" : "pulse",
    textContent: r.phase === "over" ? "📨 Invite" : "📨 Invite friends", // short in the results row
    onclick: async () => {
      invited = true;
      invite.classList.remove("pulse");
      if (navigator.share) return navigator.share({ title: "DogeBread Snake", text: "join my snek room. wow", url: link }).catch(() => {});
      await navigator.clipboard?.writeText(link).catch(() => {});
      invite.textContent = r.phase === "over" ? "Copied! wow" : "Link copied! wow";
    },
  });
  const stickers = stickerRow((k) => act.send({ t: "sticker", k }));
  if (r.phase === "over" && r.m) {
    showResults(r, act, news, invite, stickers, dev);
    return;
  }
  const host = r.players.find((p) => p.host);
  const seated = r.players.slice(0, SEATS);
  const seats = h("ul", { className: "seats" }, ...Array.from({ length: SEATS }, (_, i) => {
    const p = seated[i];
    if (!p) return h("li", { className: "empty", textContent: "🤖 bot" });
    const mine = p.id === r.you;
    const li = h("li", { className: mine ? "mine" : "" },
      mine ? h("button", { className: "skin-btn", title: "Change skin", onclick: () => pickSkin(act) }, swatch(p.skin, undefined, 44, 22)) : swatch(p.skin, undefined, 44, 22),
      h("span", { className: "name", textContent: p.name }),
      ...(p.host ? [h("span", { textContent: "👑" })] : []),
      ...(mine ? [h("button", { className: "icon small", textContent: "🎲", title: "New name", onclick: () => act.send({ t: "name" }) })] : []));
    return li;
  }));
  const humans = seated.length;
  const watching = r.players.length - humans;
  const start = me?.host
    ? h("button", { className: "main", textContent: humans > 1 ? "Start! wow" : "Start with bots", onclick: () => act.send({ t: "start" }) })
    : h("p", { className: "wait", textContent: `waiting for ${host?.name ?? "the host"} to start` });
  show("lobby",
    h("div", { className: "room-head" }, mascot(), h("div", {},
      h("p", { className: "muted", textContent: "room code" }), h("div", { className: "code", textContent: r.code }))),
    invite,
    seats,
    h("p", { className: "muted", textContent: `${humans} doge${humans === 1 ? "" : "s"} here. bots fill the empty seats.${watching ? ` +${watching} watching` : ""}` }),
    start,
    stickers,
    ...(profile.played === 0 ? [howToLoop()] : []),
    ...dev,
  );
}

function showResults(r: Replica, act: Act, news: string[], invite: HTMLElement, stickers: HTMLElement, dev: Node[]) {
  const m = r.m!;
  const w = m.winner >= 0 ? m.snakes[m.winner] : undefined;
  const headline = !w ? "wow such draw! no survivors! very dead!"
    : w.id === r.seat ? "you win! very champion! such skill! wow!"
    : w.human ? `${w.name} wins! very champion! such skill! wow!` : `${w.name} wins! much skill! so impress!`;
  // the podium: 2nd, 1st, 3rd
  const podium = h("div", { className: "podium" }, ...[1, 0, 2].filter((i) => i < r.rank.length).map((i) => {
    const s = m.snakes[r.rank[i]];
    const name = h("span", { className: "name", textContent: s.name });
    name.style.color = snakeColor(s.id);
    return h("div", { className: `step p${i + 1}` },
      h("span", { className: "crown", textContent: i === 0 && s.id === m.winner ? "👑" : "" }),
      swatch(s.skin, snakeColor(s.id), 64, 26), name, h("div", { className: "block", textContent: `#${i + 1}` }));
  }));
  const table = h("ol", { className: "results" }, ...r.rank.map((id, i) => {
    const s = m.snakes[id];
    const name = h("span", { textContent: `${s.name}${id === r.seat ? " (you)" : ""}` });
    name.style.color = snakeColor(id);
    const lived = ((s.alive ? m.tick : s.diedTick) * TICK_MS) / 1000;
    return h("li", { className: id === r.seat ? "me" : "" },
      h("span", { textContent: `#${i + 1}` }), name,
      h("span", { textContent: `${s.kills} bonk${s.kills === 1 ? "" : "s"}` }), h("span", { textContent: clock(lived) }));
  }));
  const share = h("button", {
    textContent: "📤 Share",
    onclick: async () => {
      const place = r.rank.indexOf(r.seat) + 1;
      const me = r.seat >= 0 ? m.snakes[r.seat] : undefined;
      const text = !me ? "such snek battle. come play. wow"
        : me.id === m.winner ? `I won a round of DogeBread Snake with ${me.kills} bonks. very champion. wow`
        : `I placed #${place} of ${m.snakes.length} in DogeBread Snake. much fun. wow`;
      const url = `${location.origin}/?r=${r.code}`;
      if (navigator.share) return navigator.share({ title: "DogeBread Snake", text, url }).catch(() => {});
      await navigator.clipboard?.writeText(`${text} ${url}`).catch(() => {});
      share.textContent = "Copied! wow";
    },
  });
  show("results",
    h("h2", { textContent: headline }),
    podium,
    ...news.map((n) => h("p", { className: "badge", textContent: n })),
    table,
    h("div", { className: "row" }, share, h("button", { className: "main", textContent: "Play again", onclick: () => act.send({ t: "start" }) })),
    stickers,
    h("div", { className: "row room-mini" },
      h("span", { className: "code small", textContent: r.code }), invite,
      h("button", { className: "icon", textContent: "🎲", title: "New name", onclick: () => act.send({ t: "name" }) }),
      h("button", { className: "skin-btn", title: "Change skin", onclick: () => pickSkin(act) }, swatch(mySkin(), undefined, 44, 22))),
    ...dev,
  );
}

function pickSkin(act: Act) {
  openSkins((k) => {
    profile.skin = k;
    saveProfile();
    act.send({ t: "skin", k });
  });
}

// ---- modal cards

function openModal(...kids: Node[]) {
  modal.hidden = false;
  modal.replaceChildren(h("div", { className: "card" }, ...kids));
}
export const closeModal = () => void (modal.hidden = true);
export const modalOpen = () => !modal.hidden;
modal.addEventListener("pointerdown", (e) => e.target === modal && closeModal()); // a tap outside the card closes it

export function openSettings(onChange: () => void) {
  const toggle = (label: string, key: keyof typeof settings) => {
    const b = h("button", {
      className: `toggle ${settings[key] ? "on" : ""}`,
      onclick: () => {
        settings[key] = !settings[key];
        saveSettings();
        b.classList.toggle("on", settings[key]);
        b.lastChild!.textContent = settings[key] ? "on" : "off";
        onChange();
      },
    }, h("span", { textContent: label }), h("b", { textContent: settings[key] ? "on" : "off" }));
    return b;
  };
  openModal(
    h("h2", { textContent: "settings. very config." }),
    toggle("🔊 Sound effects", "sound"),
    toggle("🎵 Music", "music"),
    ...(touch ? [toggle("🚀 Boost button", "boostButton")] : []),
    toggle("📳 Screen shake", "shake"),
    h("button", { textContent: "How to play", onclick: openHowTo }),
    h("p", { className: "muted", textContent: "Saved on this phone." }),
    h("button", { className: "main", textContent: "Done", onclick: closeModal }),
  );
}

/** The rules in four cards, worded for this device. */
function howToCards() {
  const boost = !touch ? "hold click or space to zoom" : settings.boostButton ? "hold 🚀 to zoom" : "double-tap and hold to zoom";
  return [
    ["👆", touch ? "drag anywhere to steer" : "your snek follows the mouse. arrows work too", "sway"],
    ["🚀", `${boost}. zooming burns size`, "zoom"],
    ["✂️", "cut in front of a snek. it bonks into you. much bread for you", "pop"],
    ["🧱", "the wall closes in. last doge alive wins. wow", "beat"],
  ].map(([icon, text, anim]) => h("div", { className: "how" }, h("span", { className: `icon-anim ${anim}`, textContent: icon }), h("p", { textContent: text })));
}

export function openHowTo() {
  openModal(h("h2", { textContent: "how to snek" }), ...howToCards(),
    h("button", { className: "main", textContent: "Got it! wow", onclick: closeModal }));
}

/** First-timers wait in the lobby with the rules cycling, one card at a time. */
function howToLoop() {
  const cards = howToCards();
  const box = h("div", { className: "how-loop" }, cards[0]);
  let i = 0;
  const timer = setInterval(() => {
    if (!box.isConnected) return clearInterval(timer);
    box.replaceChildren(cards[(i = (i + 1) % cards.length)]);
  }, 2800);
  return box;
}

export function openSkins(pick: (k: number) => void) {
  const grid = h("div", { className: "skins" }, ...SKINS.map((s, k) => {
    const has = s.has();
    return h("button", {
      className: `${has ? "" : "locked"} ${k === mySkin() ? "on" : ""}`,
      disabled: !has,
      onclick: () => {
        pick(k);
        closeModal();
      },
    }, swatch(k, undefined, 96, 36), h("b", { textContent: s.name }), h("span", { className: "muted", textContent: has ? (k === mySkin() ? "wearing" : "") : `🔒 ${s.feat}` }));
  }));
  openModal(h("h2", { textContent: "such skins" }), grid,
    h("p", { className: "muted", textContent: "Unlocked by feats. Saved on this phone." }),
    h("button", { className: "main", textContent: "Done", onclick: closeModal }));
}
