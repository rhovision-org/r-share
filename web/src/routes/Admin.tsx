import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Empty, ErrorBox, Field, Panel, StatusPill, Uri } from "../components/ui";
import { parseRead, parseWrite } from "../lib/api";
import * as db from "../lib/db";
import { boolean, chainInt, int } from "../lib/decode";
import {
  adminKeys,
  adminNonce,
  adminPayload,
  bookingView,
  emptyData,
  listingView,
  providerView,
} from "../lib/decoders";
import { jsonArgs, message } from "../lib/format";
import { record } from "../lib/journal";
import { useWallet } from "../lib/store";
import type { Signature } from "../lib/types";

type Target = "global" | "listing" | "provider" | "booking";

function isTarget(value: string): value is Target {
  return value === "global" || value === "listing" || value === "provider" || value === "booking";
}

function parseNonceInput(text: string): bigint | null {
  try {
    return chainInt(text, "nonce");
  } catch {
    return null;
  }
}

function parseSplit(text: string): number | null {
  try {
    return int({ min: 0, max: 10_000 })(text, "splitBps");
  } catch {
    return null;
  }
}

function isReplay(error: unknown): boolean {
  return /replay/i.test(message(error));
}

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

  const keys = useQuery({
    queryKey: ["admin-keys"],
    enabled: Boolean(client),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      return parseRead(await client.admin_keys(), adminKeys).data;
    },
    staleTime: 10_000,
  });

  const paused = useQuery({
    queryKey: ["admin-paused"],
    enabled: Boolean(client),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      return parseRead(await client.share_admin_is_paused(), boolean).data;
    },
    staleTime: 10_000,
  });

  const globalNonce = useQuery({
    queryKey: ["admin-nonce"],
    enabled: Boolean(client && target === "global"),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      return parseRead(await client.share_admin_nonce(), adminNonce).data;
    },
    staleTime: 5_000,
  });

  const targetInfo = useQuery({
    queryKey: ["admin-target", target, uri],
    enabled: Boolean(client && uri && target !== "global"),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      if (!uri) throw new Error("no target URI");
      if (target === "listing") {
        return parseRead(await client.listing_info(uri), listingView).data;
      }
      if (target === "provider") {
        return parseRead(await client.provider_info(uri), providerView).data;
      }
      return parseRead(await client.booking_info(uri), bookingView).data;
    },
    staleTime: 5_000,
  });

  const info = targetInfo.data;

  const domain = (() => {
    if (target === "global") return "admin";
    if (!info) return "";
    if (target === "listing" && "listingId" in info) return `listing:${info.listingId}`;
    if (target === "provider" && "providerId" in info) return `provider:${info.providerId}`;
    if (target === "booking" && "bookingId" in info) return `booking:${info.bookingId}`;
    return "";
  })();

  // Global nonces come from `share_admin_nonce`; object targets carry their own nonce.
  // Submit `current + 1n`; never fall back to a height or wall clock.
  const defaultNonce =
    target === "global" ? (globalNonce.data?.next ?? null) : info ? info.nonce + 1n : null;
  const typedNonce = nonce.trim();
  const nonceValue = typedNonce ? parseNonceInput(typedNonce) : defaultNonce;
  const splitValue = parseSplit(splitBps);
  const payloadAction = target === "booking" ? "split" : action;
  const payloadDomain = target === "booking" ? "arbiter" : domain;
  const args = (() => {
    if (target === "booking" && info && "bookingId" in info && splitValue !== null) {
      return [info.bookingId, splitValue];
    }
    return [];
  })();

  const build = async () => {
    if (!client || nonceValue === null || !payloadDomain) return;
    setBusy(true);
    setError(null);
    try {
      const read = parseRead(
        await client.admin_payload(
          payloadDomain,
          nonceValue,
          payloadAction,
          jsonArgs(args),
        ),
        adminPayload,
      );
      setPayload(read.data.payload);
      setDigest(read.data.digest);
      setSignatures([]);
      const id = `${payloadDomain}:${nonceValue.toString()}:${payloadAction}`;
      await db.putAdminAction({
        id,
        target,
        uri: target === "global" ? undefined : uri,
        nonce: nonceValue.toString(),
        action: payloadAction,
        args: jsonArgs(args),
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
    if (!client || !payload || nonceValue === null) return;
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
        id: pendingId(payloadDomain, nonceValue.toString(), payloadAction),
        target,
        uri: target === "global" ? undefined : uri,
        nonce: nonceValue.toString(),
        action: payloadAction,
        args: jsonArgs(args),
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
    if (!client || nonceValue === null) return;
    if (target === "booking" && splitValue === null) {
      setError(new Error("Split basis points must be an integer from 0 to 10000."));
      return;
    }
    const threshold = keys.data?.threshold ?? 1;
    if (signatures.length < threshold) {
      setError(new Error(`Need ${threshold} signatures; have ${signatures.length}.`));
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const sigs = JSON.stringify(signatures.map((entry) => entry.signature));
      let raw: string;
      if (target === "global") {
        raw = await client.admin_submit_share(nonceValue, action, "[]", sigs);
      } else if (target === "listing") {
        raw = await client.admin_submit_listing(uri, nonceValue, action, "[]", sigs);
      } else if (target === "provider") {
        raw = await client.admin_submit_provider(uri, nonceValue, sigs);
      } else {
        const opId = client.new_op_id("arbitrate", BigInt(Date.now()));
        raw = await client.arbitrate(
          uri,
          nonceValue,
          BigInt(splitValue ?? 0),
          sigs,
          opId,
        );
        await record({
          id: opId,
          kind: "admin.arbitrate",
          bookingUri: uri,
          opId,
          status: "submitted",
          summary: `Arbitrate split ${splitBps} bps`,
        });
      }
      const write = parseWrite(raw, emptyData);
      await record({
        id: pendingId(payloadDomain, nonceValue.toString(), payloadAction),
        kind: "admin.submit",
        status: write.deploy.finalized ? "finalized" : "processed",
        deployId: write.deploy.deployId,
        summary: `${payloadDomain} ${payloadAction}`,
      });
      setSignatures([]);
      setPayload("");
      setDigest("");
      setNonce("");
      await queryClient.invalidateQueries({ queryKey: ["admin-paused"] });
      await queryClient.invalidateQueries({ queryKey: ["admin-keys"] });
      await queryClient.invalidateQueries({ queryKey: ["admin-nonce"] });
      await queryClient.invalidateQueries({ queryKey: ["admin-target", target, uri] });
    } catch (caught) {
      if (isReplay(caught)) {
        setError(
          new Error(
            "Nonce conflict: the chain refused this nonce as a replay. The current nonce has been refreshed; review it and rebuild the payload before signing again.",
          ),
        );
        setNonce("");
        setPayload("");
        setDigest("");
        setSignatures([]);
        await queryClient.invalidateQueries({ queryKey: ["admin-nonce"] });
        await queryClient.invalidateQueries({ queryKey: ["admin-target", target, uri] });
      } else {
        setError(caught);
      }
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
    if (!isTarget(latest.target)) {
      setError(new Error(`Unknown pending target: ${latest.target}`));
      return;
    }
    setTarget(latest.target);
    setAction(latest.action);
    setUri(latest.uri ?? "");
    setNonce(latest.nonce);
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
          <div className="stat"><span className="muted">Next nonce</span><strong>{defaultNonce === null ? "…" : defaultNonce.toString()}</strong></div>
        </div>
        <ErrorBox error={keys.error ?? paused.error ?? globalNonce.error ?? targetInfo.error ?? error} />
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
                const next = event.target.value;
                if (!isTarget(next)) return;
                setTarget(next);
                setAction(next === "booking" ? "split" : next === "global" ? "pauseAll" : "pause");
                setNonce("");
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
              <input
                value={uri}
                onChange={(event) => {
                  setUri(event.target.value);
                  setNonce("");
                }}
                placeholder="rho:id:…"
              />
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
          <Field
            label="Nonce"
            hint={
              defaultNonce === null
                ? target === "global"
                  ? "reading the global nonce…"
                  : uri
                    ? "reading the target nonce…"
                    : "enter a target URI to read its nonce"
                : `current + 1 = ${defaultNonce.toString()}`
            }
          >
            <input
              value={nonce}
              onChange={(event) => setNonce(event.target.value)}
              placeholder={defaultNonce === null ? "" : defaultNonce.toString()}
            />
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
          <button
            disabled={
              busy ||
              !payloadDomain ||
              nonceValue === null ||
              (target === "booking" && splitValue === null)
            }
            onClick={() => void build()}
          >
            Build payload
          </button>
          <button className="ghost" disabled={busy || !payload} onClick={() => void sign()}>Sign with current profile</button>
          <button className="primary" disabled={busy || !payload || nonceValue === null} onClick={() => void submit()}>Submit</button>
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
