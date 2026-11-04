import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Empty, ErrorBox, Field, Panel, StatusPill, Uri } from "../components/ui";
import { parseRead, parseWrite } from "../lib/api";
import * as db from "../lib/db";
import { record } from "../lib/journal";
import { useWallet } from "../lib/store";
import type { BookingView, ListingView, ProviderView, Signature } from "../lib/types";

type Target = "global" | "listing" | "provider" | "booking";

export default function Admin() {
  const { client, address, profiles, switchProfile, refreshProfiles } = useWallet();
  const queryClient = useQueryClient();
  useEffect(() => {
    void refreshProfiles();
  }, [refreshProfiles]);
  const [target, setTarget] = useState<Target>("global");
  const [uri, setUri] = useState("");
  const [action, setAction] = useState("pauseAll");
  const [nonce, setNonce] = useState("");
  const [splitBps, setSplitBps] = useState("0");
  const [payload, setPayload] = useState("");
  const [digest, setDigest] = useState("");
  const [signatures, setSignatures] = useState<Signature[]>([]);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [password, setPassword] = useState("");
  const [switchTo, setSwitchTo] = useState("");
  const chainHeight = useQuery({
    queryKey: ["height"],
    enabled: Boolean(client),
    queryFn: async () => Number(await client!.height()),
    refetchInterval: 10_000,
  });

  const keys = useQuery({
    queryKey: ["admin-keys"],
    enabled: Boolean(client),
    queryFn: async () =>
      parseRead<{ admins: string[]; threshold: number }>(await client!.admin_keys()).data,
    staleTime: 10_000,
  });

  const paused = useQuery({
    queryKey: ["admin-paused"],
    enabled: Boolean(client),
    queryFn: async () => parseRead<boolean>(await client!.share_admin_is_paused()).data,
    staleTime: 10_000,
  });

  const targetInfo = useQuery({
    queryKey: ["admin-target", target, uri],
    enabled: Boolean(client && uri && target !== "global"),
    queryFn: async () => {
      if (target === "listing") return parseRead<ListingView>(await client!.listing_info(uri)).data;
      if (target === "provider") return parseRead<ProviderView>(await client!.provider_info(uri)).data;
      return parseRead<BookingView>(await client!.booking_info(uri)).data;
    },
    staleTime: 5_000,
  });

  const domain = (() => {
    if (target === "global") return "admin";
    const info = targetInfo.data;
    if (!info) return "";
    if (target === "listing" && "listingId" in info) return `listing:${info.listingId}`;
    if (target === "provider" && "providerId" in info) return `provider:${info.providerId}`;
    if (target === "booking" && "bookingId" in info) return `booking:${info.bookingId}`;
    return "";
  })();

  const defaultNonce = (() => {
    if (targetInfo.data && "nonce" in targetInfo.data) return String(targetInfo.data.nonce + 1);
    return "";
  })();
  const effectiveNonce = nonce || defaultNonce || String((chainHeight.data ?? 0) + 1000);
  const payloadAction = target === "booking" ? "split" : action;
  const payloadDomain = target === "booking" ? "arbiter" : domain;
  const args = (() => {
    if (target === "booking" && targetInfo.data && "bookingId" in targetInfo.data) {
      return [targetInfo.data.bookingId, Number(splitBps)];
    }
    return [];
  })();

  const build = async () => {
    if (!client) return;
    setBusy(true);
    setError(null);
    try {
      const read = parseRead<{ payload: string; digest: string }>(
        await client.admin_payload(
          payloadDomain,
          BigInt(effectiveNonce),
          payloadAction,
          JSON.stringify(args),
        ),
      );
      setPayload(read.data.payload);
      setDigest(read.data.digest);
      setSignatures([]);
      const id = `${payloadDomain}:${effectiveNonce}:${payloadAction}`;
      await db.putAdminAction({
        id,
        target,
        uri: target === "global" ? undefined : uri,
        nonce: Number(effectiveNonce),
        action: payloadAction,
        args: JSON.stringify(args),
        payload: read.data.payload,
        digest: read.data.digest,
        signatures: [],
        createdAt: Date.now(),
      });
    } catch (caught) {
      setError(caught);
    } finally {
      setBusy(false);
    }
  };

  const sign = async () => {
    if (!client || !payload) return;
    setBusy(true);
    setError(null);
    try {
      const signature = client.sign_payload(payload);
      const publicKey = client.public_key() ?? address ?? "unknown";
      if (signatures.some((entry) => entry.publicKey === publicKey)) {
        throw new Error("This profile already signed the payload.");
      }
      const next = [...signatures, { publicKey, signature }];
      setSignatures(next);
      await db.putAdminAction({
        id: pendingId(payloadDomain, effectiveNonce, payloadAction),
        target,
        uri: target === "global" ? undefined : uri,
        nonce: Number(effectiveNonce),
        action: payloadAction,
        args: JSON.stringify(args),
        payload,
        digest,
        signatures: next,
        createdAt: Date.now(),
      });
    } catch (caught) {
      setError(caught);
    } finally {
      setBusy(false);
    }
  };

  const submit = async () => {
    if (!client) return;
    const threshold = keys.data?.threshold ?? 1;
    if (signatures.length < threshold) {
      setError(new Error(`Need ${threshold} signatures; have ${signatures.length}.`));
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const sigs = JSON.stringify(signatures.map((entry) => entry.signature));
      const n = BigInt(effectiveNonce);
      let raw: string;
      if (target === "global") {
        raw = await client.admin_submit_share(n, action, "[]", sigs);
      } else if (target === "listing") {
        raw = await client.admin_submit_listing(uri, n, action, "[]", sigs);
      } else if (target === "provider") {
        raw = await client.admin_submit_provider(uri, n, sigs);
      } else {
        const opId = client.new_op_id("arbitrate", BigInt(Date.now()));
        raw = await client.arbitrate(uri, n, BigInt(splitBps), sigs, opId);
        await record({
          id: opId,
          kind: "admin.arbitrate",
          bookingUri: uri,
          opId,
          status: "submitted",
          summary: `Arbitrate split ${splitBps} bps`,
        });
      }
      const write = parseWrite<Record<string, never>>(raw);
      await record({
        id: pendingId(payloadDomain, effectiveNonce, payloadAction),
        kind: "admin.submit",
        status: write.deploy.finalized ? "finalized" : "processed",
        deployId: write.deploy.deployId,
        summary: `${payloadDomain} ${payloadAction}`,
      });
      setSignatures([]);
      setPayload("");
      setDigest("");
      await queryClient.invalidateQueries({ queryKey: ["admin-paused"] });
      await queryClient.invalidateQueries({ queryKey: ["admin-keys"] });
      await queryClient.invalidateQueries({ queryKey: ["admin-target", target, uri] });
    } catch (caught) {
      setError(caught);
    } finally {
      setBusy(false);
    }
  };

  const loadPending = async () => {
    const all = await db.listAdminActions();
    const latest = all.sort((a, b) => b.createdAt - a.createdAt)[0];
    if (!latest) {
      setError(new Error("No pending action."));
      return;
    }
    setTarget(latest.target as Target);
    setAction(latest.action);
    setUri(latest.uri ?? "");
    setNonce(String(latest.nonce));
    setPayload(latest.payload);
    setDigest(latest.digest);
    setSignatures(latest.signatures);
  };

  if (!client) return <Panel><Empty>Waiting for the deployment manifest.</Empty></Panel>;

  return (
    <div className="stack">
      <Panel>
        <div className="panel-head">
          <h1>Admin and arbitration</h1>
          <span className="muted">
            Payloads are produced by the deployed Codec. Sign with enough saved profiles to
            reach {keys.data?.threshold ?? "…"}-of-{keys.data?.admins.length ?? "…"}.
          </span>
        </div>
        <div className="grid three">
          <div className="stat"><span className="muted">Global pause</span><strong>{paused.data === undefined ? "…" : String(paused.data)}</strong></div>
          <div className="stat"><span className="muted">Threshold</span><strong>{keys.data?.threshold ?? "…"}</strong></div>
          <div className="stat"><span className="muted">Signatures</span><strong>{signatures.length}</strong></div>
        </div>
        <ErrorBox error={keys.error ?? paused.error ?? targetInfo.error ?? error} />
        <div className="row wrap">
          <label className="check"><input type="checkbox" checked={paused.data === true} readOnly /><span>paused</span></label>
          <StatusPill status={paused.data ? "paused" : "active"} />
          <button className="ghost" onClick={() => void loadPending()}>Load pending action</button>
        </div>
      </Panel>

      <Panel>
        <div className="grid two">
          <Field label="Target">
            <select
              value={target}
              onChange={(event) => {
                const next = event.target.value as Target;
                setTarget(next);
                setAction(next === "booking" ? "split" : next === "global" ? "pauseAll" : "pause");
                setPayload("");
                setSignatures([]);
              }}
            >
              <option value="global">Global ShareAdmin</option>
              <option value="listing">Listing</option>
              <option value="provider">Provider</option>
              <option value="booking">Booking arbitration</option>
            </select>
          </Field>
          <Field label={target === "global" ? "Domain (fixed)" : "Target URI"}>
            {target === "global" ? (
              <input readOnly value="admin" />
            ) : (
              <input value={uri} onChange={(event) => setUri(event.target.value)} placeholder="rho:id:…" />
            )}
          </Field>
          <Field label="Action">
            <select value={action} onChange={(event) => setAction(event.target.value)}>
              {target === "global" && (
                <>
                  <option value="pauseAll">pauseAll</option>
                  <option value="resumeAll">resumeAll</option>
                </>
              )}
              {target === "listing" && (
                <>
                  <option value="pause">pause</option>
                  <option value="delist">delist</option>
                </>
              )}
              {target === "provider" && <option value="pause">pause</option>}
              {target === "booking" && <option value="split">split (arbitrate)</option>}
            </select>
          </Field>
          <Field label="Nonce" hint={defaultNonce ? `current + 1 = ${defaultNonce}` : "use a height-based nonce"}>
            <input value={nonce} onChange={(event) => setNonce(event.target.value)} placeholder={defaultNonce} />
          </Field>
          {target === "booking" && (
            <Field label="Split to provider (bps)">
              <input value={splitBps} onChange={(event) => setSplitBps(event.target.value)} />
            </Field>
          )}
        </div>
        <div className="stat">
          <span className="muted">Signing domain</span>
          <strong className="mono">{payloadDomain || "…"}</strong>
        </div>
        <div className="row wrap">
          <button disabled={busy || !payloadDomain} onClick={() => void build()}>Build payload</button>
          <button className="ghost" disabled={busy || !payload} onClick={() => void sign()}>Sign with current profile</button>
          <button className="primary" disabled={busy || !payload} onClick={() => void submit()}>Submit</button>
        </div>
        {payload && (
          <div className="payload">
            <div className="row">
              <Uri value={payload} label="payload" />
              <span className="mono small">{digest}</span>
            </div>
            <pre className="json">{payload}</pre>
          </div>
        )}
        {signatures.length > 0 && (
          <div className="list">
            {signatures.map((entry) => (
              <div className="list-row" key={entry.publicKey}>
                <Uri value={entry.publicKey} label="signer" />
                <span className="mono small">{entry.signature.slice(0, 20)}…</span>
              </div>
            ))}
          </div>
        )}
      </Panel>

      <Panel>
        <div className="panel-head">
          <h2>Switch signer</h2>
          <span className="muted">
            Each saved profile can add one signature. The key stays in WASM.
          </span>
        </div>
        {profiles.length === 0 && <Empty>No saved profiles.</Empty>}
        <div className="grid two">
          <Field label="Profile">
            <select value={switchTo} onChange={(event) => setSwitchTo(event.target.value)}>
              <option value="">Select…</option>
              {profiles.map((profile) => (
                <option key={profile.address} value={profile.address}>
                  {profile.label || profile.address.slice(0, 20)}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Password">
            <input type="password" value={password} onChange={(event) => setPassword(event.target.value)} />
          </Field>
        </div>
        <button
          className="ghost"
          disabled={!switchTo || !password}
          onClick={async () => {
            setError(null);
            try {
              await switchProfile(switchTo, password);
              setPassword("");
            } catch (caught) {
              setError(caught);
            }
          }}
        >
          Switch profile
        </button>
      </Panel>
    </div>
  );
}

function pendingId(domain: string, nonce: string, action: string): string {
  return `${domain}:${nonce}:${action}`;
}
