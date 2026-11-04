// Typed re-export of the wasm-pack output. The generated module is in src/wasm/pkg.
import init, { Rshare, version } from "./pkg/rshare_wasm.js";

let ready: Promise<unknown> | null = null;

export async function initWasm(): Promise<void> {
  if (!ready) ready = init();
  await ready;
}

export { Rshare, version };
