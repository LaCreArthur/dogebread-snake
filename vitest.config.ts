import { defineConfig } from "vitest/config";

// The rules tests are plain TypeScript, so no Cloudflare plugin here (it clashes with vitest's server).
// e2e/ holds the old Rust game's Playwright specs, hence the explicit include.
export default defineConfig({ test: { include: ["tests/**/*.test.ts"] } });
