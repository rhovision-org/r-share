import type { ResourceKind, Terms } from "./types";

/** Conservative starter policy. Review every value before publishing. */
export function defaultTerms(kind: ResourceKind): Terms {
  return {
    kind,
    assetId: "REV",
    pricePerSlot: 100,
    deposit: 50,
    minSlots: 1,
    maxSlots: 4,
    instantBook: true,
    contiguous: false,
    holdBlocks: 1000,
    depositReviewBlocks: 0,
    disputeWindowBlocks: 0,
    noShowBlocks: 0,
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

export const TERM_FIELDS: Array<{
  key: keyof Omit<Terms, "kind" | "assetId">;
  label: string;
  money?: boolean;
  advanced?: boolean;
}> = [
  { key: "pricePerSlot", label: "Price per slot", money: true },
  { key: "deposit", label: "Deposit", money: true },
  { key: "minSlots", label: "Minimum slots" },
  { key: "maxSlots", label: "Maximum slots" },
  { key: "maxUnits", label: "Maximum units" },
  { key: "instantBook", label: "Instant book" },
  { key: "contiguous", label: "Slots must be contiguous" },
  { key: "holdBlocks", label: "Hold blocks" },
  { key: "feeBps", label: "Platform fee (bps)" },
  { key: "partialRefundBps", label: "Partial refund (bps)", advanced: true },
  { key: "partialRefundBeforeBlocks", label: "Partial refund before (blocks)", advanced: true },
  { key: "fullRefundBeforeBlocks", label: "Full refund before (blocks)", advanced: true },
  { key: "depositReviewBlocks", label: "Deposit review (blocks)", advanced: true },
  { key: "disputeWindowBlocks", label: "Dispute window (blocks)", advanced: true },
  { key: "noShowBlocks", label: "No-show after (blocks)", advanced: true },
  { key: "eventTransferCutoffBlocks", label: "Event transfer cutoff (blocks)", advanced: true },
  { key: "maxLeadBlocks", label: "Maximum booking lead (blocks)", advanced: true },
  { key: "maxSlotBlocks", label: "Maximum slot length (blocks)", advanced: true },
];
