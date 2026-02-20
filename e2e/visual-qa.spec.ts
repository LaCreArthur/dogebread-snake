import { test } from "@playwright/test";
import { waitForLoaded, waitForState, getDebugState } from "./helpers";

test("capture visual QA screenshots", async ({ page }) => {
  await page.goto("http://localhost:8080");
  await waitForLoaded(page);

  // 1. WaitingToStart screen
  await page.screenshot({ path: "qa-screenshots/01-waiting.png" });

  // 2. Start the game
  await page.keyboard.press("ArrowRight");

  // 3. Countdown
  await page.waitForTimeout(500);
  await page.screenshot({ path: "qa-screenshots/02-countdown.png" });

  // 4. Playing starts
  await waitForState(page, "Playing");
  await page.screenshot({ path: "qa-screenshots/03-playing-start.png" });

  // 5. A few seconds in
  await page.waitForTimeout(3000);
  await page.screenshot({ path: "qa-screenshots/04-playing-3s.png" });

  // 6. More gameplay
  await page.waitForTimeout(5000);
  await page.screenshot({ path: "qa-screenshots/05-playing-8s.png" });

  // 7. After arena shrink (~12s)
  await page.waitForTimeout(5000);
  await page.screenshot({ path: "qa-screenshots/06-after-shrink.png" });

  // 8. Late game
  await page.waitForTimeout(10000);
  const state = await getDebugState(page);
  await page.screenshot({ path: "qa-screenshots/07-late-game.png" });

  // 9. If game over, capture that too
  if (state?.gameState === "GameOver") {
    await page.screenshot({ path: "qa-screenshots/08-gameover.png" });
  } else {
    // Wait more for game over
    await page.waitForTimeout(15000);
    await page.screenshot({ path: "qa-screenshots/08-late-or-over.png" });
  }
});
