import type { ResourceKind, Terms } from "./types";

/** Conservative starter policy. Review every value before publishing. */
export function defaultTerms(kind: ResourceKind): Terms {
  return {
    kind,
    assetId: "REV",
    pricePerSlot: 100n,
    deposit: 50n,
    minSlots: 1,
    maxSlots: 4,
    instantBook: true,
    contiguous: false,
    holdBlocks: 1000,
    depositReviewBlocks: 0,
    disputeWindowBlocks: 10,
    noShowBlocks: 10,
    fullRefundBeforeBlocks: 0,
    partialRefundBps: 0,
    partialRefundBeforeBlocks: 0,
    feeBps: 0,
    maxLeadBlocks: 1_000_000,
    maxSlotBlocks: 1000,
    maxUnits: 2,
    eventTransferCutoffBlocks: 0,
  };
}

export const KINDS: ResourceKind[] = ["lodging", "event", "bike", "car", "robot"];

type BooleanTermKey = "instantBook" | "contiguous";
type MoneyTermKey = "pricePerSlot" | "deposit";
type NumberTermKey = Exclude<
  keyof Omit<Terms, "kind" | "assetId">,
  BooleanTermKey | MoneyTermKey
>;

export type TermField =
  | { key: BooleanTermKey; label: string; kind: "boolean"; advanced?: boolean }
  | { key: MoneyTermKey; label: string; kind: "money"; advanced?: boolean }
  | { key: NumberTermKey; label: string; kind: "number"; advanced?: boolean };

export const TERM_FIELDS: TermField[] = [
  { key: "pricePerSlot", label: "Price per slot", kind: "money" },
  { key: "deposit", label: "Deposit", kind: "money" },
  { key: "minSlots", label: "Minimum slots", kind: "number" },
  { key: "maxSlots", label: "Maximum slots", kind: "number" },
  { key: "maxUnits", label: "Maximum units", kind: "number" },
  { key: "instantBook", label: "Instant book", kind: "boolean" },
  { key: "contiguous", label: "Slots must be contiguous", kind: "boolean" },
  { key: "holdBlocks", label: "Hold blocks", kind: "number" },
  { key: "feeBps", label: "Platform fee (bps)", kind: "number" },
  { key: "partialRefundBps", label: "Partial refund (bps)", kind: "number", advanced: true },
  {
    key: "partialRefundBeforeBlocks",
    label: "Partial refund before (blocks)",
    kind: "number",
    advanced: true,
  },
  {
    key: "fullRefundBeforeBlocks",
    label: "Full refund before (blocks)",
    kind: "number",
    advanced: true,
  },
  {
    key: "depositReviewBlocks",
    label: "Deposit review (blocks)",
    kind: "number",
    advanced: true,
  },
  {
    key: "disputeWindowBlocks",
    label: "Dispute window (blocks)",
    kind: "number",
    advanced: true,
  },
  { key: "noShowBlocks", label: "No-show after (blocks)", kind: "number", advanced: true },
  {
    key: "eventTransferCutoffBlocks",
    label: "Event transfer cutoff (blocks)",
    kind: "number",
    advanced: true,
  },
  {
    key: "maxLeadBlocks",
    label: "Maximum booking lead (blocks)",
    kind: "number",
    advanced: true,
  },
  { key: "maxSlotBlocks", label: "Maximum slot length (blocks)", kind: "number", advanced: true },
];
