// Local activity journal. It is a device log, not a chain index.

import * as db from "./db";
import type { JournalEntry } from "./types";

export async function record(
  entry: Omit<JournalEntry, "createdAt"> & { createdAt?: number },
): Promise<void> {
  await db.addJournal({ ...entry, createdAt: entry.createdAt ?? Date.now() });
}

export async function list(): Promise<JournalEntry[]> {
  const entries = await db.listJournal();
  return entries.sort((a, b) => b.createdAt - a.createdAt);
}

export async function mark(
  id: string,
  status: JournalEntry["status"],
  extra: Partial<JournalEntry> = {},
): Promise<void> {
  const entries = await db.listJournal();
  const previous = entries.find((entry) => entry.id === id);
  if (!previous) return;
  await db.addJournal({ ...previous, ...extra, status });
}
