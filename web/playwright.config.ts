import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 360_000,
  workers: 1,
  fullyParallel: false,
  use: { baseURL: "http://localhost:5173" },
  webServer: {
    command: "npm run preview -- --port 5173 --strictPort",
    url: "http://localhost:5173",
    reuseExistingServer: true,
  },
});
