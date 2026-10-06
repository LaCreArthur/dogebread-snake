export const $ = (id: string) => document.getElementById(id)!;

export function h<K extends keyof HTMLElementTagNameMap>(tag: K, props: Record<string, unknown> = {}, ...kids: (Node | string)[]) {
  const el = document.createElement(tag);
  Object.assign(el, props);
  el.append(...kids);
  return el;
}

export const clock = (seconds: number) => `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
