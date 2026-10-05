// Entry: static files are served by the platform; this Worker only handles /api.
export { Room } from "./room";

const ALPHABET = "23456789ABCDEFGHJKMNPQRSTUVWXYZ"; // no 0/O, 1/I/L
const CODE = /^\/api\/rooms\/([2-9A-HJKMNP-Z]{5})\/ws$/;

const newCode = () => [...crypto.getRandomValues(new Uint8Array(5))].map((b) => ALPHABET[b % ALPHABET.length]).join("");

export default {
  async fetch(req, env) {
    const url = new URL(req.url);
    if (req.method === "POST" && url.pathname === "/api/rooms") {
      for (;;) {
        const code = newCode();
        if (await env.ROOM.getByName(code).init(code)) return Response.json({ code });
      }
    }
    const ws = url.pathname.match(CODE);
    if (ws && req.headers.get("Upgrade") === "websocket") return env.ROOM.getByName(ws[1]).fetch(req);
    return new Response("not found", { status: 404 });
  },
} satisfies ExportedHandler<Env>;
