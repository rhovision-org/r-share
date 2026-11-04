// Copy the generated deployment manifest and catalogue into public/.
// Both are outputs of the devnet install/fixture commands and are gitignored.
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const webDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repo = resolve(webDir, "..");
const publicDir = resolve(webDir, "public");
mkdirSync(publicDir, { recursive: true });

const manifest = resolve(repo, "deployments/devnet.json");
if (existsSync(manifest)) {
  copyFileSync(manifest, resolve(publicDir, "devnet.json"));
  console.log("sync-manifest: copied deployments/devnet.json");
} else {
  console.warn("sync-manifest: no deployments/devnet.json; paste a manifest in Settings");
}

const catalogue = resolve(repo, "deployments/catalogue.json");
if (existsSync(catalogue)) {
  copyFileSync(catalogue, resolve(publicDir, "catalogue.json"));
  console.log("sync-manifest: copied deployments/catalogue.json");
} else {
  console.warn("sync-manifest: no deployments/catalogue.json; the Explore page will be empty");
}
