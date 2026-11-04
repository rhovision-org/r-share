import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate, useParams } from "react-router-dom";
import {
  CopyButton,
  Empty,
  ErrorBox,
  Field,
  Money,
  Panel,
  ReadBadge,
  StatusPill,
  Uri,
} from "../components/ui";
import { parseRead, parseWrite, shareUrl } from "../lib/api";
import { formatUnits } from "../lib/format";
import { record } from "../lib/journal";
import { useWallet } from "../lib/store";
import { TERM_FIELDS } from "../lib/termsDefaults";
import type {
  ListingView,
  ProviderView,
  QuoteSummary,
  ReadMeta,
  SlotState,
  UnitSlotRequest,
} from "../lib/types";

export default function Listing() {
  const { uri } = useParams<{ uri: string }>();
  const { client, status: walletStatus } = useWallet();
  const navigate = useNavigate();
  const [selected, setSelected] = useState<Record<string, string[]>>({});
  const [quote, setQuote] = useState<{ value: QuoteSummary; meta: ReadMeta } | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);

  const info = useQuery({
    queryKey: ["listing", uri],
    enabled: Boolean(client && uri),
    queryFn: async () =>
      parseRead<ListingView>(await client!.listing_info(uri!)),
    staleTime: 5_000,
  });

  const listing = info.data?.data;

  const provider = useQuery({
    queryKey: ["provider", listing?.providerUri],
    enabled: Boolean(client && listing),
    queryFn: async () =>
      parseRead<ProviderView>(await client!.provider_info(listing!.providerUri)),
    staleTime: 5_000,
  });

  const height = useQuery({
    queryKey: ["height"],
    enabled: Boolean(client),
    queryFn: async () => Number(await client!.height()),
    refetchInterval: 5_000,
  });

  const summary = useMemo(() => unitSlotRequests(selected), [selected]);
  const allRequests = useMemo(() => {
    if (!listing) return [];
    return Object.values(listing.units).map((unit) => ({
      unitId: unit.unitId,
      slots: Object.keys(unit.schedule),
    }));
  }, [listing]);

  const probe = useQuery({
    queryKey: ["probe", uri, listing?.termsHash],
    enabled: Boolean(client && listing && allRequests.length > 0),
    queryFn: async () =>
      parseRead<Record<string, Record<string, SlotState>>>(
        await client!.probe(uri!, JSON.stringify(allRequests)),
      ),
    staleTime: 3_000,
    refetchInterval: 10_000,
  });

  const toggle = (unitId: string, slotId: string) => {
    if (!listing) return;
    setError(null);
    const current = selected[unitId] ?? [];
    const isSelected = current.includes(slotId);
    if (isSelected) {
      setSelected({ ...selected, [unitId]: current.filter((slot) => slot !== slotId) });
      return;
    }
    const total = Object.values(selected).reduce((sum, slots) => sum + slots.length, 0);
    if (total >= listing.terms.maxSlots) {
      setError(new Error(`At most ${listing.terms.maxSlots} slots per booking.`));
      return;
    }
    const next = { ...selected, [unitId]: [...current, slotId] };
    if (Object.keys(next).filter((key) => next[key]!.length > 0).length > listing.terms.maxUnits) {
      setError(new Error(`At most ${listing.terms.maxUnits} units per booking.`));
      return;
    }
    setSelected(next);
  };

  const getQuote = async () => {
    if (!client || !uri) return;
    setBusy(true);
    setError(null);
    try {
      const requests = unitSlotRequests(selected);
      const read = parseRead<QuoteSummary>(await client.quote(uri, JSON.stringify(requests)));
      setQuote({ value: read.data, meta: read.read });
    } catch (caught) {
      setError(caught);
    } finally {
      setBusy(false);
    }
  };

  const book = async () => {
    if (!client || !uri || !listing || !quote) return;
    if (walletStatus !== "unlocked") {
      setError(new Error("Unlock a wallet to book."));
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const requests = unitSlotRequests(selected);
      const opId = client.new_op_id("book", BigInt(Date.now()));
      const raw = await client.book(uri, JSON.stringify(requests), opId, listing.termsHash);
      const write = parseWrite<{
        bookingId: string;
        bookingUri: string;
        ledgerUri: string;
        guestId: string;
      }>(raw);
      await record({
        id: opId,
        kind: "book",
        listingUri: uri,
        bookingUri: write.data.bookingUri,
        opId,
        deployId: write.deploy.deployId,
        status: write.deploy.finalized ? "finalized" : "processed",
        summary: `Booked ${requests.length} unit request(s) for ${formatUnits(quote.value.rent)} REV`,
      });
      navigate(`/booking/${write.data.bookingUri}`);
    } catch (caught) {
      setError(caught);
    } finally {
      setBusy(false);
    }
  };

  if (!uri) return <ErrorBox error={new Error("No listing URI.")} />;
  if (!client) {
    return <Panel><Empty>Waiting for the wallet and deployment manifest.</Empty></Panel>;
  }

  return (
    <div className="stack">
      {info.isLoading && <Panel><Empty>Loading listing…</Empty></Panel>}
      <ErrorBox error={info.error} />
      {listing && (
        <>
          <Panel>
            <div className="panel-head">
              <div>
                <h1>{listing.listingId}</h1>
                <span className="muted">
                  <StatusPill status={listing.status} /> {listing.kind} ·{" "}
                  <Link to={`/provider/${listing.providerUri}`}>provider</Link>
                </span>
              </div>
              <div className="row">
                <ReadBadge meta={info.data?.read} />
                <CopyButton text={shareUrl("listing", uri)} />
              </div>
            </div>
            <div className="grid two">
              <div>
                <Field label="Listing URI">
                  <input readOnly value={uri} />
                </Field>
                <Uri value={listing.providerUri} label="provider" />
                <p className="mono small">termsHash {listing.termsHash}</p>
                {provider.data && (
                  <p className="muted">
                    {String(provider.data.data.profile.name ?? "provider")} ·{" "}
                    <StatusPill status={provider.data.data.status} />
                  </p>
                )}
              </div>
              <div>
                <Field label="Finalized height">
                  <input readOnly value={height.data ?? "…"} />
                </Field>
              </div>
            </div>
            <TermsTable listing={listing} />
          </Panel>

          <Panel>
            <div className="panel-head">
              <h2>Availability</h2>
              <span className="muted">
                Free reads use explore-deploy. Held records show the reclaim height.
              </span>
            </div>
            <ReadBadge meta={probe.data?.read} />
            <ErrorBox error={probe.error} />
            {Object.values(listing.units).length === 0 && (
              <Empty>This listing has no units. The provider must add one.</Empty>
            )}
            <div className="units">
              {Object.values(listing.units).map((unit) => (
                <div className="unit" key={unit.unitId}>
                  <div className="unit-head">
                    <strong>{unit.unitId}</strong>
                    <span className="muted">
                      <Money value={unit.pricePerSlot} /> per slot ·{" "}
                      <StatusPill status={unit.status} />
                    </span>
                  </div>
                  <div className="slots">
                    {Object.values(unit.schedule)
                      .sort((a, b) => a.startHeight - b.startHeight)
                      .map((slot) => {
                        const state = probe.data?.data[unit.unitId]?.[slot.slotId];
                        const current = height.data ?? 0;
                        const open = isOpen(state, current);
                        const checked = (selected[unit.unitId] ?? []).includes(slot.slotId);
                        return (
                          <label
                            className={`slot ${open ? "" : "closed"} ${checked ? "selected" : ""}`}
                            key={slot.slotId}
                          >
                            <input
                              type="checkbox"
                              disabled={!open || unit.status !== "active"}
                              checked={checked}
                              onChange={() => toggle(unit.unitId, slot.slotId)}
                            />
                            <span className="mono">{slot.slotId}</span>
                            <small>
                              {slot.startHeight}–{slot.endHeight}
                            </small>
                            <small>{describe(state, current)}</small>
                          </label>
                        );
                      })}
                  </div>
                </div>
              ))}
            </div>
          </Panel>

          <Panel>
            <div className="panel-head">
              <h2>Quote and book</h2>
              <span className="muted">
                A quote is a signed read and costs phlo; booking revalidates atomically.
              </span>
            </div>
            <div className="row">
              <button
                className="ghost"
                disabled={busy || summary.length === 0}
                onClick={getQuote}
              >
                Get signed quote
              </button>
              <span className="muted">
                {summary.reduce((sum, request) => sum + request.slots.length, 0)} slot(s)
                selected across {summary.length} unit(s)
              </span>
            </div>
            <ErrorBox error={error} />
            {quote && (
              <div className="quote">
                <ReadBadge meta={quote.meta} />
                <div className="grid three">
                  <div className="stat"><span className="muted">Rent</span><Money value={quote.value.rent} /></div>
                  <div className="stat"><span className="muted">Deposit</span><Money value={listing.terms.deposit} /></div>
                  <div className="stat"><span className="muted">Total</span><Money value={quote.value.rent + listing.terms.deposit} /></div>
                  <div className="stat"><span className="muted">Check-in</span><strong>{quote.value.checkIn}</strong></div>
                  <div className="stat"><span className="muted">Check-out</span><strong>{quote.value.checkOut}</strong></div>
                  <div className="stat"><span className="muted">Slots</span><strong>{quote.value.count}</strong></div>
                </div>
                <button className="primary" disabled={busy} onClick={book}>
                  Book and escrow
                </button>
                <p className="muted small">
                  The guest ledger creates the Bond from your r-cash account. Fund it in My
                  bookings first.
                </p>
              </div>
            )}
            {walletStatus !== "unlocked" && (
              <p className="muted">Unlock a wallet to sign the quote and the booking.</p>
            )}
          </Panel>
        </>
      )}
    </div>
  );
}

function unitSlotRequests(selected: Record<string, string[]>): UnitSlotRequest[] {
  return Object.entries(selected)
    .filter(([, slots]) => slots.length > 0)
    .map(([unitId, slots]) => ({ unitId, slots }));
}

function isOpen(state: SlotState | undefined, height: number): boolean {
  if (!state) return false;
  if (state[0] === "free") return true;
  if (state[0] === "held") return state[2] <= height;
  return false;
}

function describe(state: SlotState | undefined, height: number): string {
  if (!state) return "unknown";
  if (state[0] === "free") return "free";
  if (state[0] === "held") {
    return state[2] <= height ? `reclaimable (held to ${state[2]})` : `held to ${state[2]}`;
  }
  if (state[0] === "booked") return "booked";
  return "invalid record";
}

function TermsTable({ listing }: { listing: ListingView }) {
  return (
    <div className="terms-read">
      <div className="grid three">
        <div className="stat"><span className="muted">Kind</span><strong>{listing.kind}</strong></div>
        <div className="stat"><span className="muted">Asset</span><strong>{listing.terms.assetId}</strong></div>
        <div className="stat"><span className="muted">Price</span><Money value={listing.terms.pricePerSlot} /></div>
        {TERM_FIELDS.filter((field) => !field.money).map((field) => (
          <div className="stat" key={field.key}>
            <span className="muted">{field.label}</span>
            <strong>{String(listing.terms[field.key])}</strong>
          </div>
        ))}
      </div>
    </div>
  );
}
