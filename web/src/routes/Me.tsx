import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Empty, ErrorBox, Field, Money, Panel, Uri } from "../components/ui";
import { parseRead, parseWrite } from "../lib/api";
import { accountView, fundResult, guestLedgerView, guestOps, nativeBalance } from "../lib/decoders";
import { parseUnits } from "../lib/format";
import { record } from "../lib/journal";
import { useWallet } from "../lib/store";

export default function Me() {
  const { client, address, status } = useWallet();
  const queryClient = useQueryClient();
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [amount, setAmount] = useState("1");
  const [to, setTo] = useState("");

  const native = useQuery({
    queryKey: ["native", address],
    enabled: Boolean(client && address),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      if (!address) throw new Error("no address");
      return parseRead(await client.native_rev_balance(address), nativeBalance).data;
    },
    refetchInterval: 10_000,
  });

  const account = useQuery({
    queryKey: ["account", address],
    enabled: Boolean(client && address && status === "unlocked"),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      return parseRead(await client.account_balance(), accountView).data;
    },
    retry: 1,
  });

  const ledger = useQuery({
    queryKey: ["guest-ledger", address],
    enabled: Boolean(client && address && status === "unlocked"),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      return parseRead(await client.guest_ledger(), guestLedgerView).data;
    },
    retry: 1,
  });

  const ops = useQuery({
    queryKey: ["guest-ops", ledger.data?.uri],
    enabled: Boolean(client && ledger.data),
    queryFn: async () => {
      if (!client) throw new Error("wallet is locked");
      const ledgerUri = ledger.data?.uri;
      if (!ledgerUri) throw new Error("no guest ledger");
      return parseRead(await client.guest_ops(ledgerUri), guestOps).data;
    },
    staleTime: 10_000,
  });

  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await action();
      await queryClient.invalidateQueries({ queryKey: ["native", address] });
      await queryClient.invalidateQueries({ queryKey: ["account", address] });
      await queryClient.invalidateQueries({ queryKey: ["guest-ops", ledger.data?.uri] });
    } catch (caught) {
      setError(caught);
    } finally {
      setBusy(false);
    }
  };

  if (!client || !address || status !== "unlocked") {
    return (
      <Panel>
        <Empty>Unlock a wallet to see balances, funding and your bookings.</Empty>
      </Panel>
    );
  }

  return (
    <div className="stack">
      <Panel>
        <div className="panel-head">
          <h1>My bookings</h1>
          <span className="muted">The on-chain guest ledger is authoritative.</span>
        </div>
        <div className="grid three">
          <div className="stat"><span className="muted">Address</span><Uri value={address} /></div>
          <div className="stat"><span className="muted">Native REV</span>{native.data !== undefined ? <Money value={native.data} /> : <strong>…</strong>}</div>
          <div className="stat"><span className="muted">Account REV</span>{account.data ? <Money value={account.data.revBaseUnits} /> : <strong>{account.error ? "not created" : "…"}</strong>}</div>
        </div>
        <ErrorBox error={native.error ?? account.error} />
      </Panel>

      <Panel>
        <div className="panel-head">
          <h2>Funding</h2>
          <span className="muted">
            The faucet gives native REV. Moving native REV into the r-cash account is the
            escrow funding step. This is a devnet/reference path, not a fiat rail.
          </span>
        </div>
        <ErrorBox error={error} />
        <div className="row wrap">
          <button
            className="ghost"
            disabled={busy}
            onClick={() =>
              run(async () => {
                await client.faucet(address);
              })
            }
          >
            Request test REV
          </button>
        </div>
        <div className="grid two">
          <div>
            <h3>Move native REV to account</h3>
            <Field label="Amount (REV)">
              <input value={amount} onChange={(event) => setAmount(event.target.value)} />
            </Field>
            <button
              disabled={busy}
              onClick={() =>
                run(async () => {
                  const raw = await client.fund_account_rev(parseUnits(amount));
                  const write = parseWrite(raw, fundResult);
                  await record({
                    id: `fund-${Date.now()}`,
                    kind: "wallet.fund",
                    deployId: write.deploy.deployId,
                    status: write.deploy.finalized ? "finalized" : "processed",
                    summary: `Moved ${amount} REV into the r-cash account`,
                  });
                })
              }
            >
              Move to account
            </button>
          </div>
          <div>
            <h3>Withdraw account REV</h3>
            <Field label="Amount (REV)">
              <input value={amount} onChange={(event) => setAmount(event.target.value)} />
            </Field>
            <Field label="To REV address">
              <input value={to} onChange={(event) => setTo(event.target.value)} placeholder={address} />
            </Field>
            <button
              className="ghost"
              disabled={busy || !to}
              onClick={() =>
                run(async () => {
                  await client.withdraw_rev_to(parseUnits(amount), to);
                })
              }
            >
              Withdraw
            </button>
          </div>
        </div>
      </Panel>

      <Panel>
        <div className="panel-head">
          <h2>Guest ledger</h2>
        </div>
        {ledger.data && <p className="mono small">{ledger.data.uri}</p>}
        <ErrorBox error={ledger.error} />
        {ops.isLoading && <Empty>Loading operations…</Empty>}
        {ops.data && Object.keys(ops.data).length === 0 && (
          <Empty>No bookings yet. Open a listing to book a slot.</Empty>
        )}
        <div className="list">
          {ops.data &&
            Object.entries(ops.data).map(([opId, op]) => (
              <div className="list-row" key={opId}>
                <div>
                  <span className="mono small">{opId}</span>
                  <div className="muted small">
                    {op.pending ? "pending" : op.ok ? "ok" : "refused"}
                    {op.bookingId ? ` · ${op.bookingId}` : ""}
                  </div>
                </div>
                {op.uri ? <Link to={`/booking/${op.uri}`}>Open booking</Link> : <span />}
              </div>
            ))}
        </div>
      </Panel>
    </div>
  );
}
