// Dev-only test player: joins over the network like a person and steers with the bot brain,
// using only what the server sends to any client.
import { botDir } from "../game/rules";
import type { ServerMsg } from "../game/protocol";
import { apply, newReplica } from "../game/replica";
import { wsUrl } from "./net";

export function addAiPlayer(code: string) {
  const r = newReplica();
  const ws = new WebSocket(wsUrl(code, crypto.randomUUID(), null));
  ws.onmessage = (e) => {
    const msg: ServerMsg = JSON.parse(e.data);
    apply(r, msg);
    const me = r.seat >= 0 ? r.m?.snakes[r.seat] : undefined;
    if (msg.t === "step" && me?.alive && r.m) {
      const d = botDir(r.m, me);
      if (d !== me.dir) ws.send(JSON.stringify({ t: "turn", d }));
    }
  };
  return r;
}
