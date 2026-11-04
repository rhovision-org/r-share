// Wire types returned by the rshare-wasm facade. Amounts and heights are integers in
// base units/blocks; the UI formats them with exact BigInt helpers.

export type ResourceKind = "lodging" | "event" | "bike" | "car" | "robot";

export interface Terms {
  kind: ResourceKind;
  assetId: string;
  pricePerSlot: number;
  deposit: number;
  minSlots: number;
  maxSlots: number;
  instantBook: boolean;
  contiguous: boolean;
  holdBlocks: number;
  depositReviewBlocks: number;
  disputeWindowBlocks: number;
  noShowBlocks: number;
  fullRefundBeforeBlocks: number;
  partialRefundBps: number;
  partialRefundBeforeBlocks: number;
  feeBps: number;
  maxLeadBlocks: number;
  maxSlotBlocks: number;
  maxUnits: number;
  eventTransferCutoffBlocks: number;
}

export type ListingStatus = "draft" | "active" | "paused" | "delisted";
export type UnitStatus = "active" | "paused";
export type ProviderStatus = "active" | "paused";
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
  | "disputed";

export interface ReadMeta {
  atHeight: number;
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
  blockNumber: number;
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
  status: string;
}

export interface ProviderView {
  providerId: string;
  profile: Record<string, unknown>;
  status: ProviderStatus;
  listings: Record<string, ProviderListingEntry>;
  admins: string[];
  threshold: number;
  nonce: number;
}

export interface SlotSpec {
  slotId: string;
  startHeight: number;
  endHeight: number;
}

export interface UnitView {
  unitId: string;
  kind: ResourceKind;
  attrs: Record<string, unknown>;
  pricePerSlot: number;
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
  nonce: number;
}

export type SlotState =
  | ["free", number]
  | ["held", string, number]
  | ["booked", string]
  | ["invalid"];

export interface UnitSlotRequest {
  unitId: string;
  slots: string[];
}

export interface QuoteSummary {
  count: number;
  rent: number;
  minStart: number;
  maxEnd: number;
  checkIn: number;
  checkOut: number;
}

export interface TicketView {
  holderId: string;
  redeemed: boolean;
  redeemedAtHeight: number;
  transferCutoffHeight: number;
}

export interface ClaimView {
  amount: number;
  evidenceHash: string;
  atHeight: number;
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
  rent: number;
  deposit: number;
  feeBps: number;
  bondUri: string;
  holdUntilHeight: number;
  checkInHeight: number;
  checkOutHeight: number;
  status: BookingStatus;
  cancellation: Terms;
  admins: string[];
  threshold: number;
  checkInAtHeight: number;
  checkOutAtHeight: number;
  depositReleaseHeight: number;
  disputeAtHeight: number;
  rejectReason: string;
  ticket: TicketView;
  claim: ClaimView;
  ops: Record<string, unknown>;
  nonce: number;
}

export interface BondInfo {
  paid: boolean;
  expected: number;
  remaining: number;
}

export interface AccountView {
  ownerId: string;
  revBaseUnits: number;
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
  nonce: number;
  action: string;
  args: string;
  payload: string;
  digest: string;
  signatures: Signature[];
  createdAt: number;
}
