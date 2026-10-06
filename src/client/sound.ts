// Procedural sound effects, ported sample for sample from the Rust game (client/src/audio.rs), and a
// short chiptune loop for the menus.
import { settings } from "./profile";

const RATE = 22050;
const TAU = 2 * Math.PI;

type Wave = (t: number, d: number) => number;

// the Rust game's noise: 32-bit LCG, top 16 bits mapped to [-1, 1)
function lcg(seed: number) {
  let s = seed >>> 0;
  return () => {
    s = (Math.imul(s, 1103515245) + 12345) >>> 0;
    return (s >>> 16) / 32768 - 1;
  };
}

const DEFS = {
  eat: [0.08, () => (t, d) => Math.sin(TAU * (440 + (440 * t) / d) * t) * (1 - t / d) * 0.5],
  death: [0.2, () => {
    const noise = lcg(0xdeadbeef);
    return (t, d) => (Math.sin(TAU * 60 * t) * 0.4 + noise() * 0.6) * (1 - t / d) ** 2 * 0.5;
  }],
  shrinkWarning: [0.5, () => (t, d) => Math.sin(TAU * (Math.floor(t * 10) % 2 ? 800 : 600) * t) * (0.8 - (0.3 * t) / d) * 0.35],
  beep: [0.1, () => (t, d) => Math.sin(TAU * 880 * t) * (1 - t / d) * 0.4],
  go: [0.2, () => (t, d) =>
    ((Math.sin(TAU * 523.25 * t) + Math.sin(TAU * 659.25 * t) * 0.8 + Math.sin(TAU * 783.99 * t) * 0.6) / 2.4) *
    Math.sqrt(1 - t / d) * 0.5],
  gameOver: [0.5, () => (t, d) =>
    Math.sin(TAU * (800 - (600 * t) / d) * (1 + 0.02 * Math.sin(TAU * 6 * t)) * t) * Math.sqrt(1 - t / d) * 0.45],
  // not in the Rust game: my bonk, a "boing" falling from 320 to 120 Hz
  bonk: [0.18, () => (t, d) => Math.sin(TAU * (320 * t - (100 * t * t) / d)) * (1 - t / d) ** 2 * 0.6],
  // my boost: a quick zoom rising from 250 to 900 Hz
  boost: [0.15, () => (t, d) => Math.sin(TAU * (250 * t + (325 * t * t) / d)) * (1 - t / d) * 0.3],
  // my win: C E G C rising
  win: [0.6, () => (t, d) => Math.sin(TAU * [523.25, 659.25, 783.99, 1046.5][Math.min(3, Math.floor(t / 0.12))] * t) * (1 - t / d) * 0.4],
  // my boop: a short squeak rising from 500 to 900 Hz
  boop: [0.09, () => (t, d) => Math.sin(TAU * (500 * t + (200 * t * t) / d)) * (1 - t / d) * 0.5],
} satisfies Record<string, [seconds: number, make: () => Wave]>;

export type Sound = keyof typeof DEFS;

let ctx: AudioContext | null = null;
const buffers = new Map<Sound, AudioBuffer>();

// browsers start audio suspended until the page gets a gesture
for (const type of ["pointerdown", "keydown"]) addEventListener(type, () => ctx?.resume(), { capture: true });

export function play(sound: Sound) {
  if (!settings.sound) return;
  ctx ??= new AudioContext();
  let buf = buffers.get(sound);
  if (!buf) {
    const [seconds, make] = DEFS[sound];
    const wave = make();
    buf = ctx.createBuffer(1, Math.floor(RATE * seconds), RATE);
    const data = buf.getChannelData(0);
    for (let i = 0; i < data.length; i++) data[i] = wave(i / RATE, seconds);
    buffers.set(sound, buf);
  }
  const src = ctx.createBufferSource();
  src.buffer = buf;
  src.connect(ctx.destination);
  src.start();
}

// ---- menu music: 4 bars of eighths in C major pentatonic over a bass line, looped (MIDI notes, 0 = rest)
const EIGHTH = 60 / 112 / 2;
const MELODY = [
  72, 0, 76, 0, 79, 76, 81, 79, 76, 0, 74, 0, 72, 74, 76, 0,
  79, 0, 81, 0, 84, 81, 79, 76, 74, 76, 74, 72, 0, 0, 67, 0,
];
const BASS = [48, 55, 45, 52, 41, 48, 43, 50]; // one per half bar
const hz = (n: number) => 440 * 2 ** ((n - 69) / 12);
let musicTimer = 0;
let nextNote = 0;
let beat = 0;

function note(type: OscillatorType, n: number, at: number, length: number, volume: number) {
  const osc = ctx!.createOscillator();
  const gain = ctx!.createGain();
  osc.type = type;
  osc.frequency.value = hz(n);
  gain.gain.setValueAtTime(volume, at);
  gain.gain.exponentialRampToValueAtTime(0.001, at + length);
  osc.connect(gain).connect(ctx!.destination);
  osc.start(at);
  osc.stop(at + length);
}

/** Menus only; idempotent. Notes are scheduled 0.2 s ahead, so timer jitter does not matter. */
export function music(on: boolean) {
  if (on === !!musicTimer) return;
  if (!on) {
    clearInterval(musicTimer);
    musicTimer = 0;
    return;
  }
  ctx ??= new AudioContext();
  nextNote = ctx.currentTime + 0.05;
  musicTimer = window.setInterval(() => {
    nextNote = Math.max(nextNote, ctx!.currentTime); // a throttled background tab skips notes, not bursts them
    while (nextNote < ctx!.currentTime + 0.2) {
      const m = MELODY[beat % MELODY.length];
      if (m) note("square", m, nextNote, EIGHTH * 0.9, 0.035);
      if (beat % 4 === 0) note("triangle", BASS[(beat / 4) % BASS.length], nextNote, EIGHTH * 3.6, 0.09);
      nextNote += EIGHTH;
      beat++;
    }
  }, 50);
}
