import { expect, test } from "@playwright/test";
import { createWallet, faucetAndFund } from "./helpers";

test("a guest funds an account and books a catalogue listing", async ({ page }) => {
  await createWallet(page, "e2e-guest");
  await faucetAndFund(page, "0.1");

  await page.getByRole("link", { name: "Explore" }).click();
  await page.getByRole("link", { name: /Demo robot/ }).click();
  await expect(page.getByRole("heading", { name: /listing:client/ })).toBeVisible({
    timeout: 30_000,
  });

  await page.locator(".slot:not(.closed)").first().click();
  await page.getByRole("button", { name: "Get signed quote" }).click();
  await expect(page.getByRole("button", { name: "Book and escrow" })).toBeVisible({
    timeout: 120_000,
  });
  await page.getByRole("button", { name: "Book and escrow" }).click();
  await expect(page).toHaveURL(/#\/booking\//, { timeout: 180_000 });
  await expect(page.getByRole("heading", { name: "Lifecycle" })).toBeVisible();
});
