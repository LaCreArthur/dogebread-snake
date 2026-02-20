import { test, expect } from "@playwright/test";
import {
  waitForLoaded,
  waitForState,
  getDebugState,
  assertSaneState,
  collectErrors,
} from "./helpers";

test("game loads and transitions through states", async ({ page }) => {
  const errors = collectErrors(page);

  await page.goto("http://localhost:8080");
  const loaded = await waitForLoaded(page);
  assertSaneState(loaded);
  expect(loaded.gameState).toBe("WaitingToStart");
  expect(loaded.totalSnakes).toBe(10);
  expect(loaded.aliveCount).toBe(10);

  // Press arrow to start the game
  await page.keyboard.press("ArrowRight");

  // Should transition through Countdown to Playing
  const playing = await waitForState(page, "Playing");
  expect(playing.totalSnakes).toBe(10);
  expect(playing.aliveCount).toBeGreaterThan(0);
  assertSaneState(playing);

  // Wait for deaths to occur (proves game logic is running)
  // Software rendering is slow so we give it more time
  let deathOccurred = false;
  for (let i = 0; i < 60; i++) {
    await page.waitForTimeout(500);
    const state = await getDebugState(page);
    if (state && state.aliveCount < state.totalSnakes) {
      deathOccurred = true;
      assertSaneState(state);
      expect(state.tick).toBeGreaterThan(0);
      break;
    }
    // If game over already, that counts too
    if (state?.gameState === "GameOver") {
      deathOccurred = true;
      break;
    }
  }

  expect(deathOccurred).toBe(true);

  // No JS errors during the test
  const realErrors = errors.filter(
    (e) => !e.includes("Fetch API cannot load") && !e.includes("favicon")
  );
  expect(realErrors).toEqual([]);
});

test("arena shrinks during gameplay", async ({ page }) => {
  const errors = collectErrors(page);

  await page.goto("http://localhost:8080");
  await waitForLoaded(page);

  // Start the game
  await page.keyboard.press("ArrowRight");
  const playing = await waitForState(page, "Playing");
  const initialBounds = playing.arenaBounds;
  const initialWidth = initialBounds.max_x - initialBounds.min_x;

  // Wait for arena to shrink (shrink interval is 12s, give extra time for slow rendering)
  await page.waitForTimeout(20_000);

  const after = await getDebugState(page);
  // Game might be over by now, but bounds should still be readable
  if (after && after.gameState !== "GameOver") {
    const currentWidth = after.arenaBounds.max_x - after.arenaBounds.min_x;
    expect(currentWidth).toBeLessThan(initialWidth);
    assertSaneState(after);
  }

  const realErrors = errors.filter(
    (e) => !e.includes("Fetch API cannot load") && !e.includes("favicon")
  );
  expect(realErrors).toEqual([]);
});
