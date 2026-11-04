// Offline smoke test for the wasm boundary. No devnet, no network.
// Build first: npm run wasm:build
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const webDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const pkgDir = resolve(webDir, "src/wasm/pkg");
const { default: init, Rshare, parity_sign, version } = await import(
  pathToFileURL(resolve(pkgDir, "rshare_wasm.js")).href
);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const wasm = readFileSync(resolve(pkgDir, "rshare_wasm_bg.wasm"));
await init({ module_or_path: wasm });

assert(version() === "0.1.0", `unexpected version ${version()}`);

// Parity with tools/sign_payload.py:
// secret 01*32, payload "rshare:admin:v1|admin|1000|pauseAll|".
const payload = "rshare:admin:v1|admin|1000|pauseAll|";
const signature = parity_sign("01".repeat(32), payload);
assert(
  signature ===
    "3045022100c9a7a85a7e3c234f75d567a415ac4228e7a9a0eb37086cb021bd022c8bcb71d702204f0040d02a94f941903c301518d4a6032ce4a59b01965386fefe5ec0eb09f7f9",
  `unexpected signature ${signature}`,
);

const manifest = JSON.stringify({ root_uri: "rho:id:smoke-root" });
const wallet = new Rshare("http://127.0.0.1:40403", manifest);
const document = wallet.create_wallet("smoke-test-password");
const address = wallet.address();
assert(address && address.startsWith("1111"), `bad address ${address}`);
assert(document.includes('"version":3'), "keystore is not v3");
assert(!document.includes("secret"), "keystore leaked a secret field");
assert(wallet.is_unlocked(), "wallet did not unlock");

const exported = wallet.export_keystore("second-password");
assert(exported.includes('"crypto"'), "export is not a keystore");
assert(!exported.includes("secret"), "export leaked a secret field");

const opId = wallet.new_op_id("book", 1_700_000_000_000n);
assert(/^book-1700000000000-[0-9a-f]{32}$/.test(opId), `bad opId ${opId}`);

const digest = wallet.hash_text("evidence");
assert(/^[0-9a-f]{64}$/.test(digest), `bad digest ${digest}`);

const preview = JSON.parse(
  wallet.preview_ensure_unit(
    "rho:id:smoke-listing",
    "unit:1",
    '{"name":"robot"}',
    '[{"slotId":"s1","startHeight":1,"endHeight":2}]',
    100n,
  ),
);
assert(preview.bytes > 0 && preview.bytes <= 56 * 1024, `bad preview size ${preview.bytes}`);
assert(preview.term.includes("ensureUnit"), "preview is not an ensureUnit term");

wallet.lock();
assert(!wallet.is_unlocked(), "lock did not clear the key");

let rejected = false;
try {
  wallet.unlock(document, "wrong-password");
} catch {
  rejected = true;
}
assert(rejected, "wrong password was accepted");

const tampered = document.replace('"mac":"', '"mac":"00');
rejected = false;
try {
  new Rshare("http://127.0.0.1:40403", manifest).unlock(tampered, "smoke-test-password");
} catch {
  rejected = true;
}
assert(rejected, "tampered keystore was accepted");

const unlocked = wallet.unlock(document, "smoke-test-password");
assert(unlocked === address, "unlock returned a different address");

console.log("WASM SMOKE: PASS");
