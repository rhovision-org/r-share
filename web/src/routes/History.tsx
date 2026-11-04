import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Empty, Panel, StatusPill } from "../components/ui";
import { list } from "../lib/journal";
import type { JournalEntry } from "../lib/types";

export default function History() {
  const [entries, setEntries] = useState<JournalEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const reload = async () => {
    setLoading(true);
    setEntries(await list());
    setLoading(false);
  };

  useEffect(() => {
    void reload();
  }, []);

  return (
    <Panel>
      <div className="panel-head">
        <h1>Activity</h1>
        <span className="muted">
          A device journal, not a chain index. Reconcile a deploy status before treating an
          entry as final.
        </span>
        <button className="ghost" onClick={() => void reload()}>
          Refresh
        </button>
      </div>
      {loading && <Empty>Loading…</Empty>}
      {!loading && entries.length === 0 && <Empty>No activity on this device.</Empty>}
      <div className="list">
        {entries.map((entry) => (
          <div className="list-row" key={`${entry.id}-${entry.createdAt}`}>
            <div>
              <strong>{entry.kind}</strong>
              <div className="muted small">
                {entry.summary} · {new Date(entry.createdAt).toLocaleString()}
              </div>
              <div className="muted small mono">
                {entry.opId ?? ""} {entry.deployId ? `deploy ${entry.deployId.slice(0, 18)}…` : ""}
              </div>
            </div>
            <div className="row">
              <StatusPill status={entry.status} />
              {entry.bookingUri && <Link to={`/booking/${entry.bookingUri}`}>Booking</Link>}
              {entry.listingUri && <Link to={`/listing/${entry.listingUri}`}>Listing</Link>}
            </div>
          </div>
        ))}
      </div>
    </Panel>
  );
}
