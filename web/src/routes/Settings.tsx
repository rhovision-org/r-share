import { useState } from "react";
import { Empty, ErrorBox, Field, Panel } from "../components/ui";
import { message } from "../lib/format";
import { useWallet } from "../lib/store";
import { version } from "../wasm";

export default function Settings() {
  const {
    node,
    manifest,
    profiles,
    status,
    address,
    setNode,
    setManifest,
    exportKeystore,
    boot,
  } = useWallet();
  const [nodeText, setNodeText] = useState(node);
  const [manifestText, setManifestText] = useState(manifest ?? "");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<unknown>(null);
  const [note, setNote] = useState("");

  return (
    <div className="stack">
      <Panel>
        <div className="panel-head">
          <h1>Settings</h1>
          <span className="muted">Non-secret configuration only. The password is never stored.</span>
        </div>
        <ErrorBox error={error} />
        {note && <p className="muted">{note}</p>}
        <div className="grid two">
          <Field label="Node URL">
            <input value={nodeText} onChange={(event) => setNodeText(event.target.value)} />
          </Field>
          <Field label="WASM client version">
            <input readOnly value={version()} />
          </Field>
        </div>
        <div className="row">
          <button
            onClick={() => {
              setNode(nodeText);
              setNote("Node saved. Reload to reconnect.");
            }}
          >
            Save node
          </button>
          <button className="ghost" onClick={() => void boot()}>
            Reload
          </button>
        </div>
        <Field label="Deployment manifest (deployments/devnet.json)">
          <textarea
            rows={8}
            value={manifestText}
            onChange={(event) => setManifestText(event.target.value)}
          />
        </Field>
        <button
          onClick={() => {
            setError(null);
            try {
              setManifest(manifestText);
              setNote("Manifest saved.");
            } catch (caught) {
              setError(caught);
            }
          }}
        >
          Save manifest
        </button>
      </Panel>

      <Panel>
        <div className="panel-head">
          <h2>Profiles</h2>
          <span className="muted">Encrypted Web3 v3 keystores in IndexedDB.</span>
        </div>
        {profiles.length === 0 && <Empty>No saved profiles.</Empty>}
        <div className="list">
          {profiles.map((profile) => (
            <div className="list-row" key={profile.address}>
              <div>
                <strong>{profile.label || profile.address.slice(0, 20)}</strong>
                <div className="mono small">{profile.address}</div>
              </div>
              <span className="muted small">{new Date(profile.createdAt).toLocaleDateString()}</span>
            </div>
          ))}
        </div>
        <Field label="Export password (for the unlocked profile)">
          <input type="password" value={password} onChange={(event) => setPassword(event.target.value)} />
        </Field>
        <button
          className="ghost"
          disabled={status !== "unlocked" || !password}
          onClick={async () => {
            setError(null);
            try {
              if (!address) throw new Error("No unlocked profile.");
              const document = await exportKeystore(password);
              download(document, `rshare-${address.slice(-8)}.json`);
            } catch (caught) {
              setError(new Error(message(caught)));
            }
          }}
        >
          Export keystore
        </button>
      </Panel>
    </div>
  );
}

function download(document: string, filename: string) {
  const blob = new Blob([document], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = window.document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}
