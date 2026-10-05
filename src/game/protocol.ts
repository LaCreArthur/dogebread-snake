import type { Match, StepResult } from "./rules";

export type Phase = "lobby" | "countdown" | "playing" | "over";

/** Player names come only from these lists, so no free text ever reaches other players. */
export const NAME_ADJ = ["Such", "Much", "Very", "So", "Many", "Wow", "Smol", "Big", "Heckin", "Plz"];
export const NAME_NOUN = ["Doge", "Shibe", "Floof", "Bork", "Snoot", "Pupper", "Doggo", "Woofer", "Bean", "Bread"];
export type NameIdx = [number, number];
export const nameOf = ([a, n]: NameIdx) => `${NAME_ADJ[a]} ${NAME_NOUN[n]}`;
export const validName = (v: unknown): v is NameIdx =>
  Array.isArray(v) && v.length === 2 && Number.isInteger(v[0]) && Number.isInteger(v[1]) &&
  v[0] >= 0 && v[0] < NAME_ADJ.length && v[1] >= 0 && v[1] < NAME_NOUN.length;

export interface PlayerInfo {
  id: number;
  name: string;
  host: boolean;
  seat: number; // snake id in the current match, -1 = watching
}

/** The match as clients see it: no RNG state. */
export type PublicMatch = Omit<Match, "rng" | "nextCrumb">;

export type StepMsg = { t: "step"; tick: number; timeMs: number; wall: number } & StepResult;

export type ServerMsg =
  | { t: "room"; code: string; phase: Phase; players: PlayerInfo[]; you: number; name: NameIdx }
  | { t: "snap"; m: PublicMatch; countdownMs: number }
  | StepMsg
  | { t: "over"; winner: number; rank: number[] };

export type ClientMsg = { t: "steer"; a: number } | { t: "start" } | { t: "name" };
