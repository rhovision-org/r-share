import { expect, type Page } from "@playwright/test";

export const PASSWORD = "e2e-password-123";
export const NODE = "http://localhost:40403";

export async function createWallet(page: Page, label = "e2e"): Promise<void> {
  await page.goto("/");
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await page.getByLabel("Label (optional)").fill(label);
  await page.getByLabel("Password", { exact: true }).fill(PASSWORD);
  await page.getByLabel("Confirm").fill(PASSWORD);
  await page.getByRole("button", { name: /Create and download/ }).click();
  await expect(page.getByRole("button", { name: "Lock" })).toBeVisible({ timeout: 30_000 });
}

export async function goToMe(page: Page): Promise<void> {
  await page.getByRole("link", { name: "My bookings" }).click();
}

export async function faucetAndFund(page: Page, rev = "0.1"): Promise<void> {
  await goToMe(page);
  await page.getByRole("button", { name: "Request test REV" }).click();
  await page.waitForTimeout(5_000);
  await page.getByLabel("Amount (REV)").first().fill(rev);
  await page.getByRole("button", { name: "Move to account" }).click();
  await page.waitForTimeout(6_000);
}

export async function nodeHeight(page: Page): Promise<number> {
  const response = await page.request.get(`${NODE}/api/v1/status`);
  const status = (await response.json()) as { latestBlockNumber: number };
  return status.latestBlockNumber;
}

export async function unlockSaved(page: Page): Promise<void> {
  if (await page.getByRole("button", { name: "Lock" }).isVisible()) return;
  await page.getByLabel("Password", { exact: true }).fill(PASSWORD);
  await page.getByRole("button", { name: "Unlock", exact: true }).click();
  await expect(page.getByRole("button", { name: "Lock" })).toBeVisible({ timeout: 30_000 });
}
