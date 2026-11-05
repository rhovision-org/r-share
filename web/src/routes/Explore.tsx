import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router-dom";
import { Empty, ErrorBox, Panel, StatusPill, Uri } from "../components/ui";
import { chainInt, parseJson } from "../lib/decode";
import { catalogue as catalogueDecoder, nodeStatus } from "../lib/decoders";
import { useWallet } from "../lib/store";

export default function Explore() {
  const { client, node } = useWallet();
  const navigate = useNavigate();
  const [paste, setPaste] = useState("");

  const catalogue = useQuery({
    queryKey: ["catalogue"],
    queryFn: async () => {
      const response = await fetch("/catalogue.json");
      if (!response.ok) return null;
      return parseJson(await response.text(), catalogueDecoder, "$.catalogue");
    },
    staleTime: 60_000,
  });

  const chain = useQuery({
    queryKey: ["chain"],
    queryFn: async () => {
      if (!client) throw new Error("wallet is not initialised");
      const status = parseJson(await client.status(), nodeStatus, "$.status");
      const height = chainInt(await client.height(), "$.height");
      return { status, height };
    },
    enabled: Boolean(client),
    refetchInterval: 10_000,
  });

  const open = (kind: "listing" | "provider") => {
    const value = paste.trim();
    if (!value.startsWith("rho:id:")) return;
    navigate(`/${kind}/${value}`);
  };

  return (
    <div className="stack">
      <Panel>
        <div className="panel-head">
          <h1>Explore</h1>
          <span className="muted">
            Curated catalogue and live node state. There is no indexer; every card is a
            share link you can open directly.
          </span>
        </div>
        <div className="grid two">
          <div className="stat">
            <span className="muted">Node</span>
            <strong className="mono">{node}</strong>
          </div>
          <div className="stat">
            <span className="muted">Finalized height</span>
            <strong>
              {chain.data
                ? chain.data.height.toString()
                : chain.error
                  ? "unavailable"
                  : "…"}
            </strong>
          </div>
        </div>
        <ErrorBox error={chain.error} />
        <div className="paste-row">
          <input
            value={paste}
            onChange={(event) => setPaste(event.target.value)}
            placeholder="rho:id:… listing or provider"
          />
          <button className="ghost" onClick={() => open("listing")}>
            Open listing
          </button>
          <button className="ghost" onClick={() => open("provider")}>
            Open provider
          </button>
        </div>
      </Panel>

      <Panel>
        <div className="panel-head">
          <h2>Catalogue</h2>
          <span className="muted">
            A static curated list, not on-chain discovery. Cards show live status when you
            open them.
          </span>
        </div>
        {catalogue.isLoading && <Empty>Loading catalogue…</Empty>}
        {!catalogue.isLoading && !catalogue.data && (
          <Empty>
            No catalogue. Run <code>make client-fixtures</code> or paste a listing URI
            above.
          </Empty>
        )}
        {catalogue.data?.providers.length === 0 && <Empty>No providers in the catalogue.</Empty>}
        <div className="grid two">
          {catalogue.data?.providers.map((provider) => (
            <div className="card" key={provider.uri}>
              <div className="card-head">
                <strong>{provider.label || short(provider.uri)}</strong>
                <StatusPill status={provider.kind} />
              </div>
              <Uri value={provider.uri} label="provider" />
              <div className="list">
                {provider.listings.map((listing) => (
                  <Link className="list-row" key={listing.uri} to={`/listing/${listing.uri}`}>
                    <span>{listing.label || listing.listingId}</span>
                    <span className="muted">{listing.kind}</span>
                  </Link>
                ))}
              </div>
            </div>
          ))}
        </div>
      </Panel>
    </div>
  );
}

function short(value: string): string {
  return value.length > 24 ? `${value.slice(0, 12)}…${value.slice(-8)}` : value;
}
