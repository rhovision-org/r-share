// IndexedDB storage for encrypted keystores, the local activity journal and pending
// multisig actions. No plaintext secret, mnemonic or password is ever written here.

import { pendingAdminAction } from "./decoders";
import type { JournalEntry, PendingAdminAction } from "./types";

const DB_NAME = "rshare";
const DB_VERSION = 1;

export interface StoredKeystore {
  address: string;
  document: string;
  label: string;
  createdAt: number;
}

function open(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains("keystores")) {
        db.createObjectStore("keystores", { keyPath: "address" });
      }
      if (!db.objectStoreNames.contains("journal")) {
        db.createObjectStore("journal", { keyPath: "id" });
      }
      if (!db.objectStoreNames.contains("admin-actions")) {
        db.createObjectStore("admin-actions", { keyPath: "id" });
      }
      if (!db.objectStoreNames.contains("prefs")) {
        db.createObjectStore("prefs", { keyPath: "key" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

async function transaction<T>(
  store: string,
  mode: IDBTransactionMode,
  run: (objectStore: IDBObjectStore) => IDBRequest<T>,
): Promise<T> {
  const db = await open();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, mode);
    const request = run(tx.objectStore(store));
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
    tx.oncomplete = () => db.close();
  });
}

export function saveKeystore(
  address: string,
  document: string,
  label = "",
): Promise<unknown> {
  const entry: StoredKeystore = { address, document, label, createdAt: Date.now() };
  return transaction("keystores", "readwrite", (store) => store.put(entry));
}

export function loadKeystore(address: string): Promise<StoredKeystore | undefined> {
  return transaction("keystores", "readonly", (store) => store.get(address));
}

export function listKeystores(): Promise<StoredKeystore[]> {
  return transaction("keystores", "readonly", (store) => store.getAll());
}

export function addJournal(entry: JournalEntry): Promise<unknown> {
  return transaction("journal", "readwrite", (store) => store.put(entry));
}

export function listJournal(): Promise<JournalEntry[]> {
  return transaction("journal", "readonly", (store) => store.getAll());
}

export function putAdminAction(action: PendingAdminAction): Promise<unknown> {
  return transaction("admin-actions", "readwrite", (store) => store.put(action));
}

export async function listAdminActions(): Promise<PendingAdminAction[]> {
  const rows = await transaction<unknown[]>("admin-actions", "readonly", (store) =>
    store.getAll(),
  );
  // Normalize old records (numeric nonce) through the same decoder used for new ones.
  return rows.map((row, index) => pendingAdminAction(row, `$.adminActions[${index}]`));
}

export function deleteAdminAction(id: string): Promise<unknown> {
  return transaction("admin-actions", "readwrite", (store) => store.delete(id));
}

export function getPref(key: string): Promise<{ key: string; value: string } | undefined> {
  return transaction("prefs", "readonly", (store) => store.get(key));
}

export function setPref(key: string, value: string): Promise<unknown> {
  return transaction("prefs", "readwrite", (store) => store.put({ key, value }));
}
