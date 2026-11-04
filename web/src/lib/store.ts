// Wallet state. The private key lives in the wasm module; this store holds only the
// unlocked client handle, public identifiers and non-secret configuration.

import { create } from "zustand";
import { initWasm, Rshare } from "../wasm";
import * as db from "./db";
import { message } from "./format";
import { networkForNode } from "./network";

const NODE_KEY = "rshare.node";
const MANIFEST_KEY = "rshare.manifest";
const DEFAULT_NODE =
  (import.meta.env.VITE_RSHARE_NODE as string | undefined) ?? "http://localhost:40403";

export type WalletStatus = "idle" | "loading" | "locked" | "unlocked" | "error";

interface WalletState {
  status: WalletStatus;
  address: string | null;
  publicKey: string | null;
  client: Rshare | null;
  node: string;
  manifest: string | null;
  profiles: db.StoredKeystore[];
  error: string | null;
  boot: () => Promise<void>;
  setNode: (node: string) => void;
  setManifest: (json: string) => void;
  refreshProfiles: () => Promise<void>;
  createWallet: (password: string, label?: string) => Promise<string>;
  unlock: (document: string, password: string) => Promise<void>;
  unlockSaved: (address: string, password: string) => Promise<void>;
  switchProfile: (address: string, password: string) => Promise<void>;
  lock: () => void;
  exportKeystore: (password: string) => Promise<string>;
  clientOrThrow: () => Rshare;
}

export const useWallet = create<WalletState>((set, get) => ({
  status: "idle",
  address: null,
  publicKey: null,
  client: null,
  node: localStorage.getItem(NODE_KEY) ?? DEFAULT_NODE,
  manifest: localStorage.getItem(MANIFEST_KEY),
  profiles: [],
  error: null,

  boot: async () => {
    set({ status: "loading", error: null });
    try {
      await initWasm();
      let manifest = localStorage.getItem(MANIFEST_KEY);
      if (!manifest) {
        const response = await fetch(networkForNode(get().node).manifestPath);
        if (response.ok) manifest = await response.text();
      }
      if (!manifest) {
        set({
          status: "error",
          error:
            "No deployment manifest. Install the contracts, then paste deployments/devnet.json in Settings.",
        });
        return;
      }
      const client = new Rshare(get().node, manifest);
      const profile = networkForNode(get().node);
      client.set_network(profile.shard, BigInt(profile.phloLimit), BigInt(profile.phloPrice));
      localStorage.setItem(MANIFEST_KEY, manifest);
      const profiles = await db.listKeystores();
      set({ manifest, client, profiles, status: "locked" });
    } catch (error) {
      set({ status: "error", error: message(error) });
    }
  },

  setNode: (node) => {
    localStorage.setItem(NODE_KEY, node);
    get().lock();
    set({ node });
  },

  setManifest: (json) => {
    try {
      const client = new Rshare(get().node, json);
      const profile = networkForNode(get().node);
      client.set_network(profile.shard, BigInt(profile.phloLimit), BigInt(profile.phloPrice));
      localStorage.setItem(MANIFEST_KEY, json);
      set({
        manifest: json,
        client,
        status: "locked",
        address: null,
        publicKey: null,
        error: null,
      });
    } catch (error) {
      set({ error: message(error) });
      throw error;
    }
  },

  refreshProfiles: async () => {
    set({ profiles: await db.listKeystores() });
  },

  createWallet: async (password, label = "") => {
    const { node, manifest } = get();
    if (!manifest) throw new Error("no deployment manifest");
    const client = new Rshare(node, manifest);
    const profile = networkForNode(node);
    client.set_network(profile.shard, BigInt(profile.phloLimit), BigInt(profile.phloPrice));
    const document = client.create_wallet(password);
    const address = client.address();
    if (!address) throw new Error("wallet creation failed");
    await db.saveKeystore(address, document, label);
    set({
      client,
      address,
      publicKey: client.public_key() ?? null,
      status: "unlocked",
      error: null,
      profiles: await db.listKeystores(),
    });
    return document;
  },

  unlock: async (document, password) => {
    const { node, manifest } = get();
    if (!manifest) throw new Error("no deployment manifest");
    const client = new Rshare(node, manifest);
    const profile = networkForNode(node);
    client.set_network(profile.shard, BigInt(profile.phloLimit), BigInt(profile.phloPrice));
    const address = client.unlock(document, password);
    await db.saveKeystore(address, document);
    set({
      client,
      address,
      publicKey: client.public_key() ?? null,
      status: "unlocked",
      error: null,
      profiles: await db.listKeystores(),
    });
  },

  unlockSaved: async (address, password) => {
    const stored = await db.loadKeystore(address);
    if (!stored) throw new Error("no saved keystore for that address");
    await get().unlock(stored.document, password);
  },

  switchProfile: async (address, password) => {
    get().lock();
    await get().unlockSaved(address, password);
  },

  lock: () => {
    try {
      get().client?.lock();
    } catch {
      // A locked client already has no key.
    }
    set({ client: get().client, address: null, publicKey: null, status: "locked" });
  },

  exportKeystore: async (password) => {
    return get().clientOrThrow().export_keystore(password);
  },

  clientOrThrow: () => {
    const client = get().client;
    if (!client) throw new Error("wallet is locked");
    return client;
  },
}));
