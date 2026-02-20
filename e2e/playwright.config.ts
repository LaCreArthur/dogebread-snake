import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  testMatch: "*.spec.ts",
  timeout: 60_000,
  retries: 1,
  use: {
    headless: true,
    viewport: { width: 1280, height: 720 },
    launchOptions: {
      args: ["--use-gl=angle", "--use-angle=swiftshader"],
    },
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
  webServer: {
    command: "python3 -m http.server 8080 --directory ../web",
    port: 8080,
    reuseExistingServer: !process.env.CI,
    timeout: 10_000,
  },
});
