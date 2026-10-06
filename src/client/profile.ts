// What this device remembers: settings, the player's stats and the skins they unlock. Saved on this
// phone only (no accounts); storage can throw (private mode, blocked site data), which reads as empty.

export const load = (s: () => Storage, k: string) => {
  try { return s().getItem(k); } catch { return null; }
};
export const save = (s: () => Storage, k: string, v: string) => {
  try { s().setItem(k, v); } catch { /* not remembered */ }
};
const loadJson = <T extends object>(k: string, base: T): T => {
  try { return { ...base, ...JSON.parse(load(() => localStorage, k) ?? "{}") }; } catch { return base; }
};

export const touch = matchMedia("(pointer: coarse)").matches;

export const settings = loadJson("dbs-settings", { sound: true, music: false, boostButton: true, shake: true });
export const saveSettings = () => save(() => localStorage, "dbs-settings", JSON.stringify(settings));

export const profile = loadJson("dbs-profile", {
  played: 0,
  bonks: 0, // lifetime
  wins: 0,
  cut: false, // ever made a cut-off kill
  bestLen: 0, // in one round
  bestBonks: 0,
  skin: 0,
  seen: { steer: false, boost: false, cut: false }, // first-match hints already learned
});
export const saveProfile = () => save(() => localStorage, "dbs-profile", JSON.stringify(profile));

/** Food skins; each but toast is unlocked by a feat. */
export const SKINS = [
  { name: "toast", feat: "free", has: () => true },
  { name: "garlic bread", feat: "10 bonks", has: () => profile.bonks >= 10 },
  { name: "baguette", feat: "size 200", has: () => profile.bestLen >= 200 },
  { name: "taco", feat: "win a round", has: () => profile.wins >= 1 },
  { name: "hot dog", feat: "a cut-off", has: () => profile.cut },
];
export const unlocked = () => SKINS.map((s) => s.has());

/** The device's skin, falling back to toast if its feat is gone (wiped stats). */
export const mySkin = () => (SKINS[profile.skin]?.has() ? profile.skin : 0);
