import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate, useParams } from "react-router-dom";
import TermsForm from "../components/TermsForm";
import {
  CopyButton,
  Empty,
  ErrorBox,
  Field,
  Money,
  Panel,
  StatusPill,
  Uri,
} from "../components/ui";
import { parseRead, parseWrite, shareUrl } from "../lib/api";
import { array, object, parseJson, string } from "../lib/decode";
import {
  emptyData,
  listingCreation,
  listingView,
  providerRegistration,
  providerView,
  slotSpecInput,
  unitCreation,
} from "../lib/decoders";
import { jsonValue, parseUnits } from "../lib/format";
import { record } from "../lib/journal";
import { useWallet } from "../lib/store";
import { defaultTerms } from "../lib/termsDefaults";
import type { ListingStatus, ListingView, Terms } from "../lib/types";

export default function Provider() {
  const { uri } = useParams<{ uri: string }>();
  const { client, address, status } = useWallet();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [profileJson, setProfileJson] = useState('{"name":"my provider"}');
  const [terms, setTerms] = useState<Terms>(() => defaultTerms("robot"));
  const [listingId, setListingId] = useState("");
  const [unitListing, setUnitListing] = useState("");
  const [unitId, setUnitId] = useState("");
  const [unitPrice, setUnitPrice] = useState("1");
  const [unitAttrs, setUnitAttrs] = useState('{"name":"unit"}');
  const [unitSlots, setUnitSlots] = useState(
    '[{"slotId":"s1","startHeight":0,"endHeight":0}]',
  );
  const [bookingUri, setBookingUri] = useState("");

  const myProvider = useQuery({
    queryKey: ["my-provider", uri, address],
    enabled: Boolean(client && (uri || address)),
    queryFn: async () => {
      if (uri) return uri;
      if (!address) return null;
      if (!client) throw new Error("wallet is locked");
      try {
        return parseRead(await client.provider_of(address), string).data;
      } catch (caught) {
        if (String(caught).includes("no provider")) return null;
        throw caught;
      }
    },
    staleTime: 5_000,
  });

  const providerUri = myProvider.data ?? null;

  const info = useQuery({
    queryKey: ["provider", providerUri],
    enabled: Boolean(client && providerUri),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      if (!providerUri) throw new Error("no provider URI");
      return parseRead(await client.provider_info(providerUri), providerView);
    },
    staleTime: 5_000,
  });

  const statuses = useQuery({
    queryKey: ["provider-status", providerUri, info.data?.data.nonce.toString()],
    enabled: Boolean(client && info.data),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      const data = info.data?.data;
      if (!data) throw new Error("no provider");
      const entries = Object.values(data.listings);
      const out: Record<string, ListingStatus> = {};
      for (const entry of entries) {
        out[entry.uri] = parseRead(await client.listing_info(entry.uri), listingView).data.status;
      }
      return out;
    },
    staleTime: 3_000,
  });

  useEffect(() => {
    if (info.data && !unitListing) {
      const first = Object.values(info.data.data.listings)[0];
      if (first) setUnitListing(first.uri);
    }
  }, [info.data, unitListing]);

  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await action();
      await queryClient.invalidateQueries({ queryKey: ["provider", providerUri] });
      await queryClient.invalidateQueries({ queryKey: ["provider-units", providerUri] });
      await queryClient.invalidateQueries({ queryKey: ["provider-status", providerUri] });
      await queryClient.invalidateQueries({ queryKey: ["my-provider", uri, address] });
    } catch (caught) {
      setError(caught);
    } finally {
      setBusy(false);
    }
  };

  const register = () =>
    run(async () => {
      if (!client) throw new Error("wallet is locked");
      const profile = parseJson(profileJson, object, "$.profile");
      const write = parseWrite(
        await client.register_provider(JSON.stringify(profile)),
        providerRegistration,
      );
      await record({
        id: `register-${Date.now()}`,
        kind: "provider.register",
        status: write.deploy.finalized ? "finalized" : "processed",
        deployId: write.deploy.deployId,
        summary: `Registered provider ${write.data.providerUri}`,
      });
      navigate(`/provider/${write.data.providerUri}`);
    });

  const createListing = () =>
    run(async () => {
      if (!client) throw new Error("wallet is locked");
      if (!providerUri) throw new Error("Register a provider first.");
      if (!listingId.trim()) throw new Error("Choose a listing id.");
      const write = parseWrite(
        await client.create_listing(
          providerUri,
          listingId.trim(),
          terms.kind,
          jsonValue(terms),
        ),
        listingCreation,
      );
      await record({
        id: `listing-${Date.now()}`,
        kind: "listing.create",
        status: write.deploy.finalized ? "finalized" : "processed",
        deployId: write.deploy.deployId,
        summary: `Created listing ${write.data.listingUri}`,
      });
      setListingId("");
      await queryClient.invalidateQueries({ queryKey: ["provider", providerUri] });
    });

  const listingAction = (listingUri: string, action: string) =>
    run(async () => {
      if (!client) throw new Error("wallet is locked");
      const raw =
        action === "activate"
          ? await client.activate_listing(listingUri)
          : action === "pause"
            ? await client.pause_listing(listingUri)
            : action === "resume"
              ? await client.resume_listing(listingUri)
              : await client.delist_listing(listingUri);
      const write = parseWrite(raw, emptyData);
      await record({
        id: `${action}-${Date.now()}`,
        kind: `listing.${action}`,
        listingUri,
        status: write.deploy.finalized ? "finalized" : "processed",
        deployId: write.deploy.deployId,
        summary: `${action} ${listingUri}`,
      });
    });

  const addUnit = () =>
    run(async () => {
      if (!client) throw new Error("wallet is locked");
      if (!unitListing) throw new Error("Choose a listing.");
      if (!unitId.trim()) throw new Error("Choose a unit id.");
      const attrs = parseJson(unitAttrs, object, "$.attrs");
      const slots = parseJson(unitSlots, array(slotSpecInput), "$.slots");
      const write = parseWrite(
        await client.ensure_unit(
          unitListing,
          unitId.trim(),
          JSON.stringify(attrs),
          JSON.stringify(slots),
          parseUnits(unitPrice),
        ),
        unitCreation,
      );
      await record({
        id: `unit-${Date.now()}`,
        kind: "unit.ensure",
        listingUri: unitListing,
        status: write.deploy.finalized ? "finalized" : "processed",
        deployId: write.deploy.deployId,
        summary: `Added unit ${write.data.unitId}`,
      });
      setUnitId("");
    });

  if (!client) return <Panel><Empty>Waiting for the deployment manifest.</Empty></Panel>;
  if (status !== "unlocked") {
    return (
      <Panel>
        <Empty>Unlock a wallet to register a provider or manage listings.</Empty>
      </Panel>
    );
  }

  return (
    <div className="stack">
      <ErrorBox error={myProvider.error} />
      <ErrorBox error={error} />
      {myProvider.isLoading && <Panel><Empty>Resolving your provider…</Empty></Panel>}

      {myProvider.data === null && (
        <Panel>
          <div className="panel-head">
            <h1>Register a provider</h1>
            <span className="muted">
              The provider profile is public on-chain. Do not put personal data or secrets
              in it.
            </span>
          </div>
          <Field label="Profile JSON">
            <textarea rows={4} value={profileJson} onChange={(event) => setProfileJson(event.target.value)} />
          </Field>
          <button className="primary" disabled={busy} onClick={register}>
            Register provider
          </button>
        </Panel>
      )}

      {providerUri && (
        <>
          <Panel>
            <div className="panel-head">
              <div>
                <h1>Provider</h1>
                <span className="muted">
                  {info.data ? <StatusPill status={info.data.data.status} /> : "loading"} ·{" "}
                  <Uri value={providerUri} />
                </span>
              </div>
              <CopyButton text={shareUrl("provider", providerUri)} />
            </div>
            <ErrorBox error={info.error} />
            {info.data && (
              <pre className="json">{JSON.stringify(info.data.data.profile, null, 2)}</pre>
            )}
            <Field label="Paste a booking link from a guest">
              <input value={bookingUri} onChange={(event) => setBookingUri(event.target.value)} placeholder="rho:id:…" />
            </Field>
            <button
              className="ghost"
              disabled={!bookingUri.startsWith("rho:id:")}
              onClick={() => navigate(`/booking/${bookingUri.trim()}`)}
            >
              Open booking
            </button>
          </Panel>

          <Panel>
            <div className="panel-head">
              <h2>Listings</h2>
              <span className="muted">Terms are immutable; create a new listing to change them.</span>
            </div>
            {info.data && Object.values(info.data.data.listings).length === 0 && (
              <Empty>No listings yet.</Empty>
            )}
            <div className="list">
              {info.data &&
                Object.entries(info.data.data.listings).map(([id, entry]) => {
                  const actual = statuses.data?.[entry.uri] ?? entry.status;
                  return (
                    <div className="list-row" key={id}>
                      <div>
                        <strong>{id}</strong>
                        <div className="muted small">
                          {entry.kind} · <StatusPill status={actual} /> ·{" "}
                          <Uri value={entry.uri} />
                        </div>
                      </div>
                      <div className="row">
                        <button className="ghost" onClick={() => navigate(`/listing/${entry.uri}`)}>
                          Open
                        </button>
                        {actual !== "active" && actual !== "delisted" && (
                          <button className="ghost" disabled={busy} onClick={() => listingAction(entry.uri, "activate")}>
                            Activate
                          </button>
                        )}
                        {actual === "active" && (
                          <button className="ghost" disabled={busy} onClick={() => listingAction(entry.uri, "pause")}>
                            Pause
                          </button>
                        )}
                        {actual === "paused" && (
                          <button className="ghost" disabled={busy} onClick={() => listingAction(entry.uri, "resume")}>
                            Resume
                          </button>
                        )}
                        {actual !== "delisted" && (
                          <button className="ghost" disabled={busy} onClick={() => listingAction(entry.uri, "delist")}>
                            Delist
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
            </div>
          </Panel>

          <Panel>
            <div className="panel-head">
              <h2>Create listing</h2>
            </div>
            <Field label="Listing id (unique within this provider)">
              <input value={listingId} onChange={(event) => setListingId(event.target.value)} placeholder="harbour-flat" />
            </Field>
            <TermsForm value={terms} onChange={setTerms} />
            <button className="primary" disabled={busy} onClick={createListing}>
              Create listing
            </button>
          </Panel>

          <Panel>
            <div className="panel-head">
              <h2>Add a unit</h2>
              <span className="muted">
                Slots are block-height windows. Use future heights; the chain refuses a past
                start.
              </span>
            </div>
            <div className="grid two">
              <Field label="Listing">
                <select value={unitListing} onChange={(event) => setUnitListing(event.target.value)}>
                  <option value="">Select…</option>
                  {info.data &&
                    Object.values(info.data.data.listings).map((entry) => (
                      <option key={entry.uri} value={entry.uri}>
                        {entry.uri}
                      </option>
                    ))}
                </select>
              </Field>
              <Field label="Unit id">
                <input value={unitId} onChange={(event) => setUnitId(event.target.value)} placeholder="unit:1" />
              </Field>
              <Field label="Price per slot (REV)">
                <input value={unitPrice} onChange={(event) => setUnitPrice(event.target.value)} />
              </Field>
              <Field label="Attributes JSON">
                <input value={unitAttrs} onChange={(event) => setUnitAttrs(event.target.value)} />
              </Field>
            </div>
            <Field label="Slots JSON" hint='[{"slotId":"s1","startHeight":3000,"endHeight":3100}]'>
              <textarea rows={4} value={unitSlots} onChange={(event) => setUnitSlots(event.target.value)} />
            </Field>
            <button className="primary" disabled={busy} onClick={addUnit}>
              Add unit
            </button>
          </Panel>

          {info.data && (
            <Panel>
              <div className="panel-head">
                <h2>Units</h2>
              </div>
              <UnitList providerUri={providerUri} />
            </Panel>
          )}
        </>
      )}
    </div>
  );
}

function UnitList({ providerUri }: { providerUri: string }) {
  const { client } = useWallet();
  const listings = useQuery({
    queryKey: ["provider-units", providerUri],
    enabled: Boolean(client),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      const provider = parseRead(await client.provider_info(providerUri), providerView);
      const entries = Object.values(provider.data.listings);
      const out: Array<{ uri: string; listing: ListingView }> = [];
      for (const entry of entries) {
        out.push({
          uri: entry.uri,
          listing: parseRead(await client.listing_info(entry.uri), listingView).data,
        });
      }
      return out;
    },
    staleTime: 5_000,
  });
  if (listings.isLoading) return <Empty>Loading units…</Empty>;
  if (!listings.data) return null;
  return (
    <div className="list">
      {listings.data.map(({ uri, listing }) => (
        <div className="card" key={listing.listingId}>
          <div className="card-head">
            <strong>{listing.listingId}</strong>
            <StatusPill status={listing.status} />
          </div>
          {Object.values(listing.units).map((unit) => (
            <div className="list-row" key={unit.unitId}>
              <div>
                <span className="mono">{unit.unitId}</span>
                <div className="muted small">
                  <Money value={unit.pricePerSlot} /> · <StatusPill status={unit.status} /> ·{" "}
                  {Object.keys(unit.schedule).join(", ")}
                </div>
              </div>
              <button className="ghost" onClick={() => window.location.assign(`#/listing/${uri}`)}>
                Manage on listing
              </button>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}
