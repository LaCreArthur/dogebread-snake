import type { NameIdx } from "../game/protocol";

export const wsUrl = (code: string, token: string, name: NameIdx | null) =>
  `${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/api/rooms/${code}/ws?p=${token}` +
  (name ? `&n=${name.join(".")}` : "");
