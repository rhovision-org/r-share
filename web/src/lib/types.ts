// Wire types returned by the rshare-wasm facade. Chain integers (amounts, heights and
// nonces) are `bigint`; counts and basis points stay `number`. The UI formats exact
// bigint values without floating-point arithmetic.
//
// Today the Rust facade emits amounts, heights and nonces as JSON numbers. The decoders in
// `decoders.ts` also accept decimal strings so the later Rust wave can switch the wire
// shape without changing the UI.

export type Amount = bigint;
export type Height = bigint;
export type Nonce = bigint;
export type BasisPoints = number;

export type ResourceKind = "lodging" | "event" | "bike" | "car" | "robot";

/** A status value from a newer contract version than this client understands. */
export interface UnknownStatus {
  kind: "unknown";
  raw: string;
}

export interface Terms {
  kind: ResourceKind;
  assetId: string;
  pricePerSlot: Amount;
  deposit: Amount;
  minSlots: number;
  maxSlots: number;
  instantBook: boolean;
  contiguous: boolean;
  holdBlocks: number;
  depositReviewBlocks: number;
  disputeWindowBlocks: number;
  noShowBlocks: number;
  fullRefundBeforeBlocks: number;
  partialRefundBps: BasisPoints;
  partialRefundBeforeBlocks: number;
  feeBps: BasisPoints;
  maxLeadBlocks: number;
  maxSlotBlocks: number;
  maxUnits: number;
  eventTransferCutoffBlocks: number;
}

export type ListingStatus =
  | "draft"
  | "active"
  | "paused"
  | "delisted"
  | UnknownStatus;
export type UnitStatus = "active" | "paused" | UnknownStatus;
export type ProviderStatus = "active" | "paused" | UnknownStatus;
export type BookingStatus =
  | "requested"
  | "confirmed"
  | "checkedIn"
  | "checkedOut"
  | "redeemed"
  | "settled"
  | "rejected"
  | "cancelled"
  | "expired"
  | "noShow"
  | "noShowClaimed"
  | "noShowDisputed"
  | "noShowRefunded"
  | "disputed"
  | UnknownStatus;

export interface ReadMeta {
  atHeight: Height;
  blockHash: string;
  source: "explore" | "signedDeploy";
  deployId: string | null;
}

export interface ReadEnvelope<T> {
  ok: true;
  data: T;
  read: ReadMeta;
}

export interface DeployReceipt {
  deployId: string;
  finalized: boolean;
  blockHash: string | null;
  blockNumber: Height;
  termBytes: number;
  status: string;
}

export interface WriteEnvelope<T> {
  ok: true;
  data: T;
  deploy: DeployReceipt;
}

export interface ProviderListingEntry {
  uri: string;
  kind: ResourceKind;
  termsHash: string;
  status: ListingStatus;
}

export interface ProviderView {
  providerId: string;
  profile: Record<string, unknown>;
  status: ProviderStatus;
  listings: Record<string, ProviderListingEntry>;
  admins: string[];
  threshold: number;
  nonce: Nonce;
}

export interface SlotSpec {
  slotId: string;
  startHeight: Height;
  endHeight: Height;
}

export interface UnitView {
  unitId: string;
  kind: ResourceKind;
  attrs: Record<string, unknown>;
  pricePerSlot: Amount;
  maxHoldBlocks: number;
  status: UnitStatus;
  schedule: Record<string, SlotSpec>;
}

export interface ListingView {
  listingId: string;
  providerUri: string;
  providerId: string;
  kind: ResourceKind;
  terms: Terms;
  termsHash: string;
  status: ListingStatus;
  units: Record<string, UnitView>;
  admins: string[];
  threshold: number;
  nonce: Nonce;
}

export type SlotState =
  | ["free", number]
  | ["held", string, Height]
  | ["booked", string]
  | ["invalid"];

export interface UnitSlotRequest {
  unitId: string;
  slots: string[];
}

export interface QuoteSummary {
  count: number;
  rent: Amount;
  minStart: Height;
  maxEnd: Height;
  checkIn: Height;
  checkOut: Height;
}

export interface TicketView {
  holderId: string;
  redeemed: boolean;
  redeemedAtHeight: Height;
  transferCutoffHeight: Height;
}

export interface ClaimView {
  amount: Amount;
  evidenceHash: string;
  atHeight: Height;
}

export interface BookingView {
  bookingId: string;
  guestId: string;
  providerId: string;
  listingId: string;
  listingUri: string;
  kind: ResourceKind;
  unitSlots: UnitSlotRequest[];
  assetId: string;
  rent: Amount;
  deposit: Amount;
  feeBps: BasisPoints;
  bondUri: string;
  holdUntilHeight: Height;
  checkInHeight: Height;
  checkOutHeight: Height;
  status: BookingStatus;
  cancellation: Terms;
  admins: string[];
  threshold: number;
  checkInAtHeight: Height;
  checkOutAtHeight: Height;
  depositReleaseHeight: Height;
  disputeAtHeight: Height;
  noShowClaimAtHeight: Height;
  rejectReason: string;
  ticket: TicketView;
  claim: ClaimView;
  ops: Record<string, unknown>;
  nonce: Nonce;
}

export interface BondInfo {
  paid: boolean;
  expected: Amount;
  remaining: Amount;
}

export interface AccountView {
  ownerId: string;
  revBaseUnits: Amount;
}

export interface GuestOp {
  pending: boolean;
  ok: boolean;
  bookingId: string | null;
  uri: string | null;
}

export interface CatalogueEntry {
  uri: string;
  listingId: string;
  label: string;
  kind: ResourceKind;
}

export interface CatalogueProvider {
  uri: string;
  ownerId: string;
  label: string;
  kind: ResourceKind;
  listings: CatalogueEntry[];
}

export interface Catalogue {
  generatedAt: string;
  providers: CatalogueProvider[];
}

export interface JournalEntry {
  id: string;
  kind: string;
  listingUri?: string;
  bookingUri?: string;
  opId?: string;
  deployId?: string;
  status:
    | "drafted"
    | "submitted"
    | "processed"
    | "finalized"
    | "refused"
    | "failed"
    | "unknown";
  refusal?: string;
  summary: string;
  createdAt: number;
}

export interface Signature {
  publicKey: string;
  signature: string;
}

export interface PendingAdminAction {
  id: string;
  target: string;
  uri?: string;
  /** Decimal string. Older records stored a JSON number; loading normalizes them. */
  nonce: string;
  action: string;
  args: string;
  payload: string;
  digest: string;
  signatures: Signature[];
  createdAt: number;
}

/** Node status is an open JSON document from the node; keep the raw record too. */
export interface NodeStatus {
  latestBlockNumber: Height | null;
  raw: Record<string, unknown>;
}

export interface PaymentsHealth {
  ok: boolean;
  components: Record<string, string>;
  raw: unknown;
}

export interface AdminKeys {
  admins: string[];
  threshold: number;
}

/** `share_admin_nonce` / `share_root_nonce` return the stored nonce and its successor. */
export interface NonceView {
  current: Nonce;
  next: Nonce;
}

export interface AdminPayload {
  payload: string;
  digest: string;
}
