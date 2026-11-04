import { expect, test } from "@playwright/test";
import { createWallet, faucetAndFund, goToMe, nodeHeight, unlockSaved } from "./helpers";

test("a provider publishes a listing and accepts a guest booking", async ({ browser }) => {
  const providerContext = await browser.newContext();
  const provider = await providerContext.newPage();
  await createWallet(provider, "e2e-provider");
  await goToMe(provider);
  await provider.getByRole("button", { name: "Request test REV" }).click();
  await provider.waitForTimeout(5_000);

  await provider.getByRole("link", { name: "Provider" }).click();
  await provider.getByLabel("Profile JSON").fill('{"name":"e2e provider"}');
  await provider.getByRole("button", { name: "Register provider" }).click();
  await expect(provider.getByRole("heading", { name: "Provider" })).toBeVisible({
    timeout: 120_000,
  });

  const listingId = `e2e-${Date.now()}`;
  await provider.getByLabel("Listing id (unique within this provider)").fill(listingId);
  await provider.getByLabel("Instant book").uncheck();
  await provider.getByRole("button", { name: "Create listing" }).click();
  await expect(provider.getByText(listingId).first()).toBeVisible({ timeout: 120_000 });

  const height = await nodeHeight(provider);
  const start = height + 200;
  const end = start + 100;
  await provider.getByLabel("Unit id").fill("unit:1");
  await provider.getByLabel("Price per slot (REV)").fill("0.01");
  await provider.getByLabel("Attributes JSON").fill('{"name":"e2e unit"}');
  await provider.getByLabel("Slots JSON").fill(
    JSON.stringify([{ slotId: "s1", startHeight: start, endHeight: end }]),
  );
  await provider.getByRole("button", { name: "Add unit" }).click();
  await provider.waitForTimeout(6_000);

  await provider.getByRole("button", { name: "Activate" }).first().click();
  await provider.waitForTimeout(6_000);

  const row = provider.locator(".list-row", { hasText: listingId });
  await row.getByRole("button", { name: "Open" }).click();
  await expect(provider.getByRole("heading", { name: listingId })).toBeVisible();
  const listingUri = new URL(provider.url()).hash.replace("#/listing/", "");

  const guestContext = await browser.newContext();
  const guest = await guestContext.newPage();
  await createWallet(guest, "e2e-guest-provider");
  await faucetAndFund(guest, "0.1");
  await guest.goto(`http://localhost:5173/#/listing/${listingUri}`);
  await guest.locator(".slot:not(.closed)").first().click();
  await guest.getByRole("button", { name: "Get signed quote" }).click();
  await expect(guest.getByRole("button", { name: "Book and escrow" })).toBeVisible({
    timeout: 120_000,
  });
  await guest.getByRole("button", { name: "Book and escrow" }).click();
  await expect(guest).toHaveURL(/#\/booking\//, { timeout: 180_000 });
  const bookingUrl = guest.url();

  await provider.goto(bookingUrl);
  await unlockSaved(provider);
  await expect(provider.getByRole("button", { name: "Accept" })).toBeVisible({
    timeout: 60_000,
  });
  await provider.getByRole("button", { name: "Accept" }).click();
  await expect(provider.getByText("confirmed")).toBeVisible({ timeout: 120_000 });

  await providerContext.close();
  await guestContext.close();
});
