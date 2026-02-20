import { type Page, expect } from "@playwright/test";

/** Shape of the debug bridge state exposed on window.__gameDebug. */
export interface DebugState {
  loaded: boolean;
  gameState: "WaitingToStart" | "Countdown" | "Playing" | "GameOver";
  aliveCount: number;
  totalSnakes: number;
  playerScore: number;
  playerKills: number;
  tick: number;
  arenaBounds: {
    min_x: number;
    min_y: number;
    max_x: number;
    max_y: number;
  };
}

/** Read the debug bridge state from the page. Returns null if not yet set. */
export async function getDebugState(page: Page): Promise<DebugState | null> {
  return page.evaluate(() => (window as any).__gameDebug ?? null);
}

/** Wait until the debug bridge reports a specific game state. */
export async function waitForState(
  page: Page,
  state: DebugState["gameState"],
  timeoutMs = 30_000
): Promise<DebugState> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const debug = await getDebugState(page);
    if (debug?.gameState === state) return debug;
    await page.waitForTimeout(250);
  }
  throw new Error(`Timed out waiting for game state "${state}" after ${timeoutMs}ms`);
}

/** Wait for the debug bridge to be loaded (any state). */
export async function waitForLoaded(page: Page, timeoutMs = 30_000): Promise<DebugState> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const debug = await getDebugState(page);
    if (debug?.loaded) return debug;
    await page.waitForTimeout(250);
  }
  throw new Error(`Timed out waiting for game to load after ${timeoutMs}ms`);
}

/** Assert that the debug state looks sane (basic invariant checks). */
export function assertSaneState(debug: DebugState) {
  expect(debug.loaded).toBe(true);
  expect(debug.aliveCount).toBeGreaterThanOrEqual(0);
  expect(debug.aliveCount).toBeLessThanOrEqual(debug.totalSnakes);
  expect(debug.playerScore).toBeGreaterThanOrEqual(0);
  expect(debug.playerKills).toBeGreaterThanOrEqual(0);
  expect(debug.arenaBounds.min_x).toBeLessThan(debug.arenaBounds.max_x);
  expect(debug.arenaBounds.min_y).toBeLessThan(debug.arenaBounds.max_y);
}

/** Collect all JS console errors during a test. */
export function collectErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(msg.text());
  });
  page.on("pageerror", (err) => errors.push(err.message));
  return errors;
}
