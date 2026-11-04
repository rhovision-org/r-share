import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test } from "@playwright/test";
import { nodeHeight } from "./helpers";

const PASSWORD = "e2e-admin-password";

async function importKeystore(page: import("@playwright/test").Page, path: string) {
  await page.goto("/");
  await page.getByRole("button", { name: "Import", exact: true }).first().click();
  await page.getByLabel("Keystore JSON").fill(readFileSync(resolve("..", path), "utf8"));
  await page.getByLabel("Password", { exact: true }).fill(PASSWORD);
  await page.getByRole("button", { name: "Import", exact: true }).last().click();
  await expect(page.getByRole("button", { name: "Lock" })).toBeVisible({ timeout: 30_000 });
}

test("two admin profiles sign a global pause and resume", async ({ page }) => {
  await importKeystore(page, ".devnet/keystores/admin1.json");
  const second = await page.context().newPage();
  await importKeystore(second, ".devnet/keystores/admin2.json");
  await second.close();
  await page.getByRole("link", { name: "Admin" }).click();
  await page.getByRole("link", { name: "Explore" }).click();
  await page.getByRole("link", { name: "Admin" }).click();
  await expect(page.getByRole("heading", { name: "Admin and arbitration" })).toBeVisible();

  const base = await nodeHeight(page);
  const pauseNonce = base + 1000;
  await page.getByLabel("Nonce").fill(String(pauseNonce));
  await page.getByRole("button", { name: "Build payload" }).click();
  await expect(page.getByRole("button", { name: "Sign with current profile" })).toBeEnabled();
  await page.getByRole("button", { name: "Sign with current profile" }).click();
  const sigStat = page.locator(".stat", { hasText: "Signatures" });
  await expect(sigStat).toContainText("1", { timeout: 30_000 });

  const profileSelect = page.getByLabel("Profile");
  await expect(profileSelect.locator("option")).toHaveCount(3, { timeout: 30_000 });
  await profileSelect.selectOption({ label: "1111CLScsUWtwpkr9BPJ" });
  await page.getByLabel("Password", { exact: true }).fill(PASSWORD);
  await page.getByRole("button", { name: "Switch profile" }).click();
  await page.waitForTimeout(3_000);
  await expect(page.getByRole("button", { name: "Sign with current profile" })).toBeEnabled({
    timeout: 30_000,
  });
  await page.getByRole("button", { name: "Sign with current profile" }).click();
  await expect(sigStat).toContainText("2", { timeout: 30_000 });
  await page.getByRole("button", { name: "Submit" }).click();

  const pauseStat = page.locator(".stat", { hasText: "Global pause" });
  await expect(pauseStat).toContainText("true", { timeout: 120_000 });

  await page.getByLabel("Action").selectOption("resumeAll");
  await page.getByLabel("Nonce").fill(String(pauseNonce + 1));
  await page.getByRole("button", { name: "Build payload" }).click();
  await page.getByRole("button", { name: "Sign with current profile" }).click();
  await profileSelect.selectOption({ label: "111128UKva8io9yqMGYR" });
  await page.getByLabel("Password", { exact: true }).fill(PASSWORD);
  await page.getByRole("button", { name: "Switch profile" }).click();
  await page.waitForTimeout(3_000);
  await expect(page.getByRole("button", { name: "Sign with current profile" })).toBeEnabled({
    timeout: 30_000,
  });
  await page.getByRole("button", { name: "Sign with current profile" }).click();
  await page.getByRole("button", { name: "Submit" }).click();
  await expect(pauseStat).toContainText("false", { timeout: 120_000 });
});
