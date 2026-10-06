import type { NameIdx } from "../game/protocol";

export const wsUrl = (code: string, token: string, name: NameIdx | null, skin = 0) =>
  `${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/api/rooms/${code}/ws?p=${token}&s=${skin}` +
  (name ? `&n=${name.join(".")}` : "");
