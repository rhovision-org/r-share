import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useParams } from "react-router-dom";
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
import { chainInt } from "../lib/decode";
import { bondInfo, bookingView, emptyData } from "../lib/decoders";
import { jsonArgs, parseUnits } from "../lib/format";
import { record } from "../lib/journal";
import { useWallet } from "../lib/store";

export default function Booking() {
  const { uri } = useParams<{ uri: string }>();
  const { client, address } = useWallet();
  const queryClient = useQueryClient();
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [claimAmount, setClaimAmount] = useState("");
  const [evidence, setEvidence] = useState("");
  const [reason, setReason] = useState("");
  const [newGuest, setNewGuest] = useState("");

  const info = useQuery({
    queryKey: ["booking", uri],
    enabled: Boolean(client && uri),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      if (!uri) throw new Error("no booking URI");
      return parseRead(await client.booking_info(uri), bookingView);
    },
    staleTime: 5_000,
  });

  const booking = info.data?.data;

  const height = useQuery({
    queryKey: ["height"],
    enabled: Boolean(client),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      return chainInt(await client.height(), "$.height");
    },
    refetchInterval: 5_000,
  });

  const bond = useQuery({
    queryKey: ["bond", booking?.bondUri],
    enabled: Boolean(client && booking),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      const bondUri = booking?.bondUri;
      if (!bondUri) throw new Error("no bond URI");
      return parseRead(await client.bond_info(bondUri), bondInfo);
    },
    staleTime: 5_000,
  });

  const run = async (action: string, args: unknown[] = []) => {
    if (!client || !uri) return;
    setBusy(true);
    setError(null);
    try {
      const opId = client.new_op_id(action, BigInt(Date.now()));
      const raw = await client.booking_action(uri, action, opId, jsonArgs(args));
      const write = parseWrite(raw, emptyData);
      await record({
        id: opId,
        kind: action,
        bookingUri: uri,
        opId,
        deployId: write.deploy.deployId,
        status: write.deploy.finalized ? "finalized" : "processed",
        summary: `${action} on ${booking?.bookingId ?? uri}`,
      });
      await queryClient.invalidateQueries({ queryKey: ["booking", uri] });
      await queryClient.invalidateQueries({ queryKey: ["bond", booking?.bondUri] });
      await queryClient.invalidateQueries({ queryKey: ["height"] });
    } catch (caught) {
      setError(caught);
    } finally {
      setBusy(false);
    }
  };

  if (!uri) return <ErrorBox error={new Error("No booking URI.")} />;
  if (!client) return <Panel><Empty>Waiting for the deployment manifest.</Empty></Panel>;

  const current = height.data ?? 0n;
  const isGuest = address === booking?.guestId;
  const isProvider = address === booking?.providerId;
  const status = booking?.status;
  const isEvent = booking?.kind === "event";

  return (
    <div className="stack">
      {info.isLoading && <Panel><Empty>Loading booking…</Empty></Panel>}
      <ErrorBox error={info.error} />
      {booking && (
        <>
          <Panel>
            <div className="panel-head">
              <div>
                <h1>{booking.bookingId}</h1>
                <span className="muted">
                  <StatusPill status={booking.status} /> {booking.kind} ·{" "}
                  <Link to={`/listing/${booking.listingUri}`}>listing</Link>
                </span>
              </div>
              <div className="row">
                <ReadBadge meta={info.data?.read} />
                <CopyButton text={shareUrl("booking", uri)} label="Share booking link" />
              </div>
            </div>
            <div className="grid three">
              <div className="stat"><span className="muted">Rent</span><Money value={booking.rent} /></div>
              <div className="stat"><span className="muted">Deposit</span><Money value={booking.deposit} /></div>
              <div className="stat"><span className="muted">Fee</span><strong>{(booking.feeBps / 100).toFixed(2)}%</strong></div>
              <div className="stat"><span className="muted">Check-in height</span><strong>{booking.checkInHeight.toString()}</strong></div>
              <div className="stat"><span className="muted">Check-out height</span><strong>{booking.checkOutHeight.toString()}</strong></div>
              <div className="stat"><span className="muted">Hold until</span><strong>{booking.holdUntilHeight.toString()}</strong></div>
              <div className="stat"><span className="muted">Finalized height</span><strong>{current.toString()}</strong></div>
              <div className="stat"><span className="muted">Bond</span><strong>{bond.data ? (bond.data.data.paid ? "paid" : "open") : "…"}</strong></div>
              <div className="stat"><span className="muted">Bond remaining</span>{bond.data ? <Money value={bond.data.data.remaining} /> : <strong>…</strong>}</div>
            </div>
            <div className="grid two">
              <div><Uri value={booking.guestId} label="guest" /></div>
              <div><Uri value={booking.providerId} label="provider" /></div>
            </div>
            {booking.unitSlots.length > 0 && (
              <div className="list">
                {booking.unitSlots.map((unit) => (
                  <div className="list-row" key={unit.unitId}>
                    <span className="mono">{unit.unitId}</span>
                    <span>{unit.slots.join(", ")}</span>
                  </div>
                ))}
              </div>
            )}
            {booking.rejectReason && <p className="error">Rejected: {booking.rejectReason}</p>}
            {isEvent && (
              <p className="muted">
                Ticket holder {booking.ticket.holderId} · redeemed {String(booking.ticket.redeemed)} ·
                transfer cutoff {booking.ticket.transferCutoffHeight.toString()}
              </p>
            )}
          </Panel>

          <Panel>
            <div className="panel-head">
              <h2>Lifecycle</h2>
              <span className="muted">
                Buttons are gated by the contract's heights and your role. A refusal is a
                designed answer, not a crash.
              </span>
            </div>
            <ErrorBox error={error} />
            <div className="row wrap">
              {isProvider && status === "requested" && (
                <>
                  <button disabled={busy} onClick={() => void run("accept")}>Accept</button>
                  <button className="ghost" disabled={busy} onClick={() => void run("reject", ["provider rejected"])}>Reject</button>
                </>
              )}
              {isProvider && status === "confirmed" && (
                <button disabled={busy} onClick={() => void run("checkIn")}>Check in</button>
              )}
              {isProvider && status === "confirmed"
                && booking.cancellation.disputeWindowBlocks > 0
                && current >= booking.checkInHeight + BigInt(booking.cancellation.noShowBlocks) && (
                <button className="ghost" disabled={busy}
                  onClick={() => void run("claimNoShow")}>Claim no-show</button>
              )}
              {isGuest && status === "noShowClaimed" && reason
                && current < booking.noShowClaimAtHeight + BigInt(booking.cancellation.disputeWindowBlocks) && (
                <button className="ghost" disabled={busy}
                  onClick={() => void run("disputeNoShow", [client.hash_text(reason)])}>
                  Contest no-show
                </button>
              )}
              {status === "noShowClaimed"
                && current >= booking.noShowClaimAtHeight + BigInt(booking.cancellation.disputeWindowBlocks) && (
                <button disabled={busy} onClick={() => void run("resolveNoShow")}>
                  Resolve no-show
                </button>
              )}
              {status === "noShowDisputed"
                && current >= booking.disputeAtHeight + BigInt(booking.cancellation.disputeWindowBlocks) && (
                <button disabled={busy} onClick={() => void run("resolveNoShow")}>
                  Resolve contested no-show
                </button>
              )}
              {isProvider && status === "checkedIn" && (
                <button disabled={busy} onClick={() => void run("checkOut")}>Check out</button>
              )}
              {isGuest && status === "checkedIn" && (
                <button disabled={busy} onClick={() => void run("guestCheckOut")}>Guest check-out</button>
              )}
              {isGuest && (status === "requested" || status === "confirmed") && (
                <button className="ghost" disabled={busy} onClick={() => void run("cancel")}>Cancel</button>
              )}
              {isProvider && (status === "requested" || status === "confirmed") && (
                <button className="ghost" disabled={busy} onClick={() => void run("cancelByProvider")}>Cancel by provider</button>
              )}
              {status === "checkedOut" && current >= booking.checkOutHeight && (
                <button disabled={busy} onClick={() => void run("settle")}>Settle</button>
              )}
              {status === "requested" && current > booking.holdUntilHeight && (
                <button className="ghost" disabled={busy} onClick={() => void run("expire")}>Expire</button>
              )}
              {status === "disputed" && (
                <button disabled={busy} onClick={() => void run("resolveDispute")}>Resolve dispute</button>
              )}
              {isEvent && isGuest && status === "confirmed" && (
                <button disabled={busy} onClick={() => void run("redeem")}>Redeem ticket</button>
              )}
              {isEvent && status === "confirmed" && current >= booking.checkOutHeight && (
                <button className="ghost" disabled={busy} onClick={() => void run("closeEvent")}>Close event</button>
              )}
            </div>

            {isProvider && status === "checkedOut" && (
              <div className="action-box">
                <h3>Claim deposit</h3>
                <Field label="Amount (REV)">
                  <input value={claimAmount} onChange={(event) => setClaimAmount(event.target.value)} placeholder="0.0" />
                </Field>
                <Field label="Evidence (hashed locally)">
                  <textarea rows={2} value={evidence} onChange={(event) => setEvidence(event.target.value)} />
                </Field>
                <button
                  disabled={busy || !claimAmount || !evidence}
                  onClick={() =>
                    void run("claimDeposit", [
                      parseUnits(claimAmount),
                      client.hash_text(evidence),
                    ])
                  }
                >
                  Claim
                </button>
              </div>
            )}

            {(isGuest || isProvider) && status !== "settled" && status !== "cancelled"
              && status !== "expired" && status !== "rejected"
              && status !== "noShowClaimed" && status !== "noShowDisputed"
              && status !== "noShowRefunded" && (
              <div className="action-box">
                <h3>Dispute</h3>
                <Field label="Reason (hashed locally)">
                  <textarea rows={2} value={reason} onChange={(event) => setReason(event.target.value)} />
                </Field>
                <button
                  className="ghost"
                  disabled={busy || !reason}
                  onClick={() => void run("dispute", [client.hash_text(reason)])}
                >
                  Open dispute
                </button>
              </div>
            )}

            {isEvent && isGuest && status === "confirmed" && (
              <div className="action-box">
                <h3>Transfer ticket</h3>
                <Field label="New guest id (REV address)">
                  <input value={newGuest} onChange={(event) => setNewGuest(event.target.value)} />
                </Field>
                <button disabled={busy || !newGuest} onClick={() => void run("transferTicket", [newGuest])}>
                  Transfer
                </button>
              </div>
            )}

            {!isGuest && !isProvider && (
              <p className="muted">
                You are not a party to this booking. Permissionless settle, expire and
                resolve actions are still available when their guards pass.
              </p>
            )}
          </Panel>
        </>
      )}
    </div>
  );
}
