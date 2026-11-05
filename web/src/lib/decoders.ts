// One decoder per WASM envelope and chain view. These are the only places that know the
// wire field names; the routes consume the typed views from `types.ts`.

import {
  DecodeError,
  array,
  boolean,
  chainInt,
  enumOf,
  int,
  object,
  optional,
  record,
  required,
  string,
  unknownEnum,
  unknownValue,
  type Decoder,
} from "./decode";
import type {
  AccountView,
  AdminKeys,
  AdminPayload,
  Amount,
  BondInfo,
  BookingStatus,
  BookingView,
  Catalogue,
  CatalogueEntry,
  CatalogueProvider,
  ClaimView,
  DeployReceipt,
  GuestOp,
  ListingStatus,
  ListingView,
  NodeStatus,
  NonceView,
  PaymentsHealth,
  PendingAdminAction,
  ProviderListingEntry,
  ProviderStatus,
  ProviderView,
  QuoteSummary,
  ReadEnvelope,
  ReadMeta,
  ResourceKind,
  Signature,
  SlotSpec,
  SlotState,
  Terms,
  TicketView,
  UnitStatus,
  UnitView,
  WriteEnvelope,
} from "./types";

const count = int({ min: 0 });

export const resourceKind: Decoder<ResourceKind> = enumOf(
  "lodging",
  "event",
  "bike",
  "car",
  "robot",
);

export function isResourceKind(value: string): value is ResourceKind {
  return (
    value === "lodging" ||
    value === "event" ||
    value === "bike" ||
    value === "car" ||
    value === "robot"
  );
}

export const listingStatus: Decoder<ListingStatus> = unknownEnum(
  "draft",
  "active",
  "paused",
  "delisted",
);

export const unitStatus: Decoder<UnitStatus> = unknownEnum("active", "paused");

export const providerStatus: Decoder<ProviderStatus> = unknownEnum("active", "paused");

export const bookingStatus: Decoder<BookingStatus> = unknownEnum(
  "requested",
  "confirmed",
  "checkedIn",
  "checkedOut",
  "redeemed",
  "settled",
  "rejected",
  "cancelled",
  "expired",
  "noShow",
  "noShowClaimed",
  "noShowDisputed",
  "noShowRefunded",
  "disputed",
);

export const terms: Decoder<Terms> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    kind: required(raw, "kind", resourceKind, path),
    assetId: required(raw, "assetId", string, path),
    pricePerSlot: required(raw, "pricePerSlot", chainInt, path),
    deposit: required(raw, "deposit", chainInt, path),
    minSlots: required(raw, "minSlots", count, path),
    maxSlots: required(raw, "maxSlots", count, path),
    instantBook: required(raw, "instantBook", boolean, path),
    contiguous: required(raw, "contiguous", boolean, path),
    holdBlocks: required(raw, "holdBlocks", count, path),
    depositReviewBlocks: required(raw, "depositReviewBlocks", count, path),
    disputeWindowBlocks: required(raw, "disputeWindowBlocks", count, path),
    noShowBlocks: required(raw, "noShowBlocks", count, path),
    fullRefundBeforeBlocks: required(raw, "fullRefundBeforeBlocks", count, path),
    partialRefundBps: required(raw, "partialRefundBps", int({ min: 0, max: 10_000 }), path),
    partialRefundBeforeBlocks: required(raw, "partialRefundBeforeBlocks", count, path),
    feeBps: required(raw, "feeBps", int({ min: 0, max: 10_000 }), path),
    maxLeadBlocks: required(raw, "maxLeadBlocks", count, path),
    maxSlotBlocks: required(raw, "maxSlotBlocks", count, path),
    maxUnits: required(raw, "maxUnits", count, path),
    eventTransferCutoffBlocks: required(raw, "eventTransferCutoffBlocks", count, path),
  };
};

export const slotSpec: Decoder<SlotSpec> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    slotId: required(raw, "slotId", string, path),
    startHeight: required(raw, "startHeight", chainInt, path),
    endHeight: required(raw, "endHeight", chainInt, path),
  };
};

/** Slot input for a write stays a JSON number: the WASM argument is an integer. */
export const slotSpecInput: Decoder<{
  slotId: string;
  startHeight: number;
  endHeight: number;
}> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    slotId: required(raw, "slotId", string, path),
    startHeight: required(raw, "startHeight", count, path),
    endHeight: required(raw, "endHeight", count, path),
  };
};

export const providerListingEntry: Decoder<ProviderListingEntry> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    uri: required(raw, "uri", string, path),
    kind: required(raw, "kind", resourceKind, path),
    termsHash: required(raw, "termsHash", string, path),
    status: required(raw, "status", listingStatus, path),
  };
};

export const providerView: Decoder<ProviderView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    providerId: required(raw, "providerId", string, path),
    profile: required(raw, "profile", object, path),
    status: required(raw, "status", providerStatus, path),
    listings: required(raw, "listings", record(providerListingEntry), path),
    admins: required(raw, "admins", array(string), path),
    threshold: required(raw, "threshold", count, path),
    nonce: required(raw, "nonce", chainInt, path),
  };
};

export const unitView: Decoder<UnitView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    unitId: required(raw, "unitId", string, path),
    kind: required(raw, "kind", resourceKind, path),
    attrs: required(raw, "attrs", object, path),
    pricePerSlot: required(raw, "pricePerSlot", chainInt, path),
    maxHoldBlocks: required(raw, "maxHoldBlocks", count, path),
    status: required(raw, "status", unitStatus, path),
    schedule: required(raw, "schedule", record(slotSpec), path),
  };
};

export const listingView: Decoder<ListingView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    listingId: required(raw, "listingId", string, path),
    providerUri: required(raw, "providerUri", string, path),
    providerId: required(raw, "providerId", string, path),
    kind: required(raw, "kind", resourceKind, path),
    terms: required(raw, "terms", terms, path),
    termsHash: required(raw, "termsHash", string, path),
    status: required(raw, "status", listingStatus, path),
    units: required(raw, "units", record(unitView), path),
    admins: required(raw, "admins", array(string), path),
    threshold: required(raw, "threshold", count, path),
    nonce: required(raw, "nonce", chainInt, path),
  };
};

export const unitSlotRequest: Decoder<{ unitId: string; slots: string[] }> = (
  value,
  path = "$",
) => {
  const raw = object(value, path);
  return {
    unitId: required(raw, "unitId", string, path),
    slots: required(raw, "slots", array(string), path),
  };
};

export const ticketView: Decoder<TicketView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    holderId: required(raw, "holderId", string, path),
    redeemed: required(raw, "redeemed", boolean, path),
    redeemedAtHeight: required(raw, "redeemedAtHeight", chainInt, path),
    transferCutoffHeight: required(raw, "transferCutoffHeight", chainInt, path),
  };
};

export const claimView: Decoder<ClaimView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    amount: required(raw, "amount", chainInt, path),
    evidenceHash: required(raw, "evidenceHash", string, path),
    atHeight: required(raw, "atHeight", chainInt, path),
  };
};

export const bookingView: Decoder<BookingView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    bookingId: required(raw, "bookingId", string, path),
    guestId: required(raw, "guestId", string, path),
    providerId: required(raw, "providerId", string, path),
    listingId: required(raw, "listingId", string, path),
    listingUri: required(raw, "listingUri", string, path),
    kind: required(raw, "kind", resourceKind, path),
    unitSlots: required(raw, "unitSlots", array(unitSlotRequest), path),
    assetId: required(raw, "assetId", string, path),
    rent: required(raw, "rent", chainInt, path),
    deposit: required(raw, "deposit", chainInt, path),
    feeBps: required(raw, "feeBps", int({ min: 0, max: 10_000 }), path),
    bondUri: required(raw, "bondUri", string, path),
    holdUntilHeight: required(raw, "holdUntilHeight", chainInt, path),
    checkInHeight: required(raw, "checkInHeight", chainInt, path),
    checkOutHeight: required(raw, "checkOutHeight", chainInt, path),
    status: required(raw, "status", bookingStatus, path),
    cancellation: required(raw, "cancellation", terms, path),
    admins: required(raw, "admins", array(string), path),
    threshold: required(raw, "threshold", count, path),
    checkInAtHeight: optional(raw, "checkInAtHeight", chainInt, path) ?? 0n,
    checkOutAtHeight: optional(raw, "checkOutAtHeight", chainInt, path) ?? 0n,
    depositReleaseHeight: optional(raw, "depositReleaseHeight", chainInt, path) ?? 0n,
    disputeAtHeight: optional(raw, "disputeAtHeight", chainInt, path) ?? 0n,
    noShowClaimAtHeight: optional(raw, "noShowClaimAtHeight", chainInt, path) ?? 0n,
    rejectReason: optional(raw, "rejectReason", string, path) ?? "",
    ticket: required(raw, "ticket", ticketView, path),
    claim: required(raw, "claim", claimView, path),
    ops: required(raw, "ops", record(unknownValue), path),
    nonce: required(raw, "nonce", chainInt, path),
  };
};

export const bondInfo: Decoder<BondInfo> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    paid: required(raw, "paid", boolean, path),
    expected: required(raw, "expected", chainInt, path),
    remaining: required(raw, "remaining", chainInt, path),
  };
};

export const quoteSummary: Decoder<QuoteSummary> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    count: required(raw, "count", count, path),
    rent: required(raw, "rent", chainInt, path),
    minStart: required(raw, "minStart", chainInt, path),
    maxEnd: required(raw, "maxEnd", chainInt, path),
    checkIn: required(raw, "checkIn", chainInt, path),
    checkOut: required(raw, "checkOut", chainInt, path),
  };
};

export const accountView: Decoder<AccountView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    ownerId: required(raw, "ownerId", string, path),
    revBaseUnits: required(raw, "revBaseUnits", chainInt, path),
  };
};

export const guestOp: Decoder<GuestOp> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    pending: required(raw, "pending", boolean, path),
    ok: required(raw, "ok", boolean, path),
    bookingId: optional(raw, "bookingId", string, path) ?? null,
    uri: optional(raw, "uri", string, path) ?? null,
  };
};

export const guestOps: Decoder<Record<string, GuestOp>> = record(guestOp);

/** Probe results: unit id -> slot id -> state. */
export const probeStates: Decoder<Record<string, Record<string, SlotState>>> = record(
  record(slotStateDecoder()),
);

function slotStateDecoder(): Decoder<SlotState> {
  return (value, path = "$") => {
    if (!Array.isArray(value)) throw new DecodeError(path, "array", "non-array");
    const tag = string(value[0], `${path}[0]`);
    if (tag === "free") {
      return ["free", int({ min: 0 })(value[1], `${path}[1]`)];
    }
    if (tag === "held") {
      return ["held", string(value[1], `${path}[1]`), chainInt(value[2], `${path}[2]`)];
    }
    if (tag === "booked") {
      return ["booked", string(value[1], `${path}[1]`)];
    }
    return ["invalid"];
  };
}

export const slotState: Decoder<SlotState> = (value, path = "$") =>
  slotStateDecoder()(value, path);

export const readSource: Decoder<"explore" | "signedDeploy"> = enumOf(
  "explore",
  "signedDeploy",
);

export const readMeta: Decoder<ReadMeta> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    atHeight: required(raw, "atHeight", chainInt, path),
    blockHash: required(raw, "blockHash", string, path),
    source: required(raw, "source", readSource, path),
    deployId: optional(raw, "deployId", string, path) ?? null,
  };
};

export const deployReceipt: Decoder<DeployReceipt> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    deployId: required(raw, "deployId", string, path),
    finalized: required(raw, "finalized", boolean, path),
    blockHash: optional(raw, "blockHash", string, path) ?? null,
    blockNumber: required(raw, "blockNumber", chainInt, path),
    termBytes: required(raw, "termBytes", count, path),
    status: required(raw, "status", string, path),
  };
};

export function readEnvelope<T>(data: Decoder<T>): Decoder<ReadEnvelope<T>> {
  return (value, path = "$") => {
    const raw = object(value, path);
    const ok = required(raw, "ok", boolean, path);
    if (!ok) throw new DecodeError(`${path}.ok`, "true", "false");
    return {
      ok: true,
      data: required(raw, "data", data, path),
      read: required(raw, "read", readMeta, path),
    };
  };
}

export function writeEnvelope<T>(data: Decoder<T>): Decoder<WriteEnvelope<T>> {
  return (value, path = "$") => {
    const raw = object(value, path);
    const ok = required(raw, "ok", boolean, path);
    if (!ok) throw new DecodeError(`${path}.ok`, "true", "false");
    return {
      ok: true,
      data: required(raw, "data", data, path),
      deploy: required(raw, "deploy", deployReceipt, path),
    };
  };
}

// ---------------------------------------------------------------------------
// Simple read views

export const nativeBalance: Decoder<Amount> = chainInt;

export interface GuestLedgerView {
  uri: string;
  guestId: string;
}

export const guestLedgerView: Decoder<GuestLedgerView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    uri: required(raw, "uri", string, path),
    guestId: required(raw, "guestId", string, path),
  };
};

export const nodeStatus: Decoder<NodeStatus> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    latestBlockNumber: optional(raw, "latestBlockNumber", chainInt, path) ?? null,
    raw,
  };
};

export const paymentsHealth: Decoder<PaymentsHealth> = (value, path = "$") => {
  if (!Array.isArray(value)) throw new DecodeError(path, "array", "non-array");
  const ok = boolean(value[0], `${path}[0]`);
  if (!ok) return { ok: false, components: {}, raw: value };
  const componentsRaw = object(value[1], `${path}[1]`);
  const components: Record<string, string> = {};
  for (const [key, item] of Object.entries(componentsRaw)) {
    components[key] = string(item, `${path}[1].${key}`);
  }
  return { ok, components, raw: value };
};

export const adminKeys: Decoder<AdminKeys> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    admins: required(raw, "admins", array(string), path),
    threshold: required(raw, "threshold", count, path),
  };
};

export const nonceView: Decoder<NonceView> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    current: required(raw, "current", chainInt, path),
    next: required(raw, "next", chainInt, path),
  };
};

/** `share_admin_nonce` and `share_root_nonce` share the `{current,next}` shape. */
export const adminNonce: Decoder<NonceView> = nonceView;
export const rootNonce: Decoder<NonceView> = nonceView;

export const adminPayload: Decoder<AdminPayload> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    payload: required(raw, "payload", string, path),
    digest: required(raw, "digest", string, path),
  };
};

// ---------------------------------------------------------------------------
// Write result payloads

export const emptyData: Decoder<Record<string, never>> = (value, path = "$") => {
  object(value, path);
  return {};
};

export interface ProviderRegistration {
  providerUri: string;
  ownerId: string;
}

export const providerRegistration: Decoder<ProviderRegistration> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    providerUri: required(raw, "providerUri", string, path),
    ownerId: required(raw, "ownerId", string, path),
  };
};

export interface ListingCreation {
  listingUri: string;
}

export const listingCreation: Decoder<ListingCreation> = (value, path = "$") => {
  const raw = object(value, path);
  return { listingUri: required(raw, "listingUri", string, path) };
};

export interface UnitCreation {
  unitId: string;
}

export const unitCreation: Decoder<UnitCreation> = (value, path = "$") => {
  const raw = object(value, path);
  return { unitId: required(raw, "unitId", string, path) };
};

export interface BookingCreation {
  bookingId: string;
  bookingUri: string;
  ledgerUri: string;
  guestId: string;
}

export const bookingCreation: Decoder<BookingCreation> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    bookingId: required(raw, "bookingId", string, path),
    bookingUri: required(raw, "bookingUri", string, path),
    ledgerUri: required(raw, "ledgerUri", string, path),
    guestId: required(raw, "guestId", string, path),
  };
};

export interface FundResult {
  revBaseUnits: Amount;
}

export const fundResult: Decoder<FundResult> = (value, path = "$") => {
  const raw = object(value, path);
  return { revBaseUnits: required(raw, "revBaseUnits", chainInt, path) };
};

export interface WithdrawResult {
  remainingRevBaseUnits: Amount;
}

export const withdrawResult: Decoder<WithdrawResult> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    remainingRevBaseUnits: required(raw, "remainingRevBaseUnits", chainInt, path),
  };
};

// ---------------------------------------------------------------------------
// Static catalogue (not a chain read)

export const catalogueEntry: Decoder<CatalogueEntry> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    uri: required(raw, "uri", string, path),
    listingId: required(raw, "listingId", string, path),
    label: required(raw, "label", string, path),
    kind: required(raw, "kind", resourceKind, path),
  };
};

export const catalogueProvider: Decoder<CatalogueProvider> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    uri: required(raw, "uri", string, path),
    ownerId: required(raw, "ownerId", string, path),
    label: required(raw, "label", string, path),
    kind: required(raw, "kind", resourceKind, path),
    listings: required(raw, "listings", array(catalogueEntry), path),
  };
};

export const catalogue: Decoder<Catalogue> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    generatedAt: required(raw, "generatedAt", string, path),
    providers: required(raw, "providers", array(catalogueProvider), path),
  };
};

// ---------------------------------------------------------------------------
// Local IndexedDB records

export const signature: Decoder<Signature> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    publicKey: required(raw, "publicKey", string, path),
    signature: required(raw, "signature", string, path),
  };
};

/** Accept the current decimal-string nonce and legacy numeric records. */
export const nonceText: Decoder<string> = (value, path = "$") => {
  if (typeof value === "string") {
    if (!/^-?\d+$/.test(value)) throw new DecodeError(path, "decimal string", "bad string");
    return value;
  }
  if (typeof value === "number") {
    return String(int({ min: 0 })(value, path));
  }
  throw new DecodeError(path, "decimal string or integer", "wrong type");
};

export const pendingAdminAction: Decoder<PendingAdminAction> = (value, path = "$") => {
  const raw = object(value, path);
  return {
    id: required(raw, "id", string, path),
    target: required(raw, "target", string, path),
    uri: optional(raw, "uri", string, path),
    nonce: required(raw, "nonce", nonceText, path),
    action: required(raw, "action", string, path),
    args: required(raw, "args", string, path),
    payload: required(raw, "payload", string, path),
    digest: required(raw, "digest", string, path),
    signatures: required(raw, "signatures", array(signature), path),
    createdAt: required(raw, "createdAt", count, path),
  };
};
