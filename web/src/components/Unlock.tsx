import { useEffect, useState } from "react";
import { useWallet } from "../lib/store";
import { message } from "../lib/format";

type Mode = "unlock" | "create" | "import";

export default function Unlock() {
  const {
    status,
    address,
    profiles,
    unlockSaved,
    createWallet,
    unlock,
    refreshProfiles,
  } = useWallet();
  const [mode, setMode] = useState<Mode>("unlock");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [document, setDocument] = useState("");
  const [selected, setSelected] = useState("");
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshProfiles();
  }, [refreshProfiles]);

  useEffect(() => {
    if (!selected && profiles[0]) setSelected(profiles[0].address);
  }, [profiles, selected]);

  if (
    status === "unlocked" ||
    status === "loading" ||
    status === "idle" ||
    status === "error"
  ) {
    return null;
  }

  const run = async (action: () => Promise<void>) => {
    setError(null);
    try {
      await action();
      setPassword("");
      setConfirm("");
      setDocument("");
      setError(null);
    } catch (caught) {
      setError(message(caught));
    }
  };

  const create = () =>
    run(async () => {
      if (password.length < 8) throw new Error("Use at least 8 characters.");
      if (password !== confirm) throw new Error("The passwords do not match.");
      const keystore = await createWallet(password, label);
      download(keystore, "rshare-keystore.json");
    });

  const unlockSavedNow = () =>
    run(async () => {
      if (!selected) throw new Error("No saved profile.");
      await unlockSaved(selected, password);
    });

  const importKeystore = () =>
    run(async () => {
      if (!document.trim()) throw new Error("Paste a keystore document.");
      await unlock(document, password);
    });

  return (
    <div className="overlay">
      <section className="panel unlock">
        <div className="panel-head">
          <h2>Wallet</h2>
          <span className="muted">The key stays in WASM; only an encrypted keystore is stored.</span>
        </div>
        <div className="segmented">
          {(["unlock", "create", "import"] as Mode[]).map((value) => (
            <button
              key={value}
              className={mode === value ? "active" : ""}
              onClick={() => setMode(value)}
            >
              {value === "unlock" ? "Unlock" : value === "create" ? "Create" : "Import"}
            </button>
          ))}
        </div>

        {error && <div className="error">{error}</div>}

        {mode === "unlock" && (
          <>
            {profiles.length === 0 ? (
              <p className="muted">No saved profile. Create one or import a keystore.</p>
            ) : (
              <label className="field">
                <span>Profile</span>
                <select value={selected} onChange={(event) => setSelected(event.target.value)}>
                  {profiles.map((profile) => (
                    <option key={profile.address} value={profile.address}>
                      {profile.label || profile.address.slice(0, 18)}
                    </option>
                  ))}
                </select>
              </label>
            )}
            <label className="field">
              <span>Password</span>
              <input
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                autoComplete="current-password"
              />
            </label>
            <button className="primary" onClick={unlockSavedNow} disabled={!selected}>
              Unlock
            </button>
          </>
        )}

        {mode === "create" && (
          <>
            <label className="field">
              <span>Label (optional)</span>
              <input value={label} onChange={(event) => setLabel(event.target.value)} />
            </label>
            <label className="field">
              <span>Password</span>
              <input
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                autoComplete="new-password"
              />
            </label>
            <label className="field">
              <span>Confirm</span>
              <input
                type="password"
                value={confirm}
                onChange={(event) => setConfirm(event.target.value)}
                autoComplete="new-password"
              />
            </label>
            <button className="primary" onClick={create}>
              Create and download backup
            </button>
          </>
        )}

        {mode === "import" && (
          <>
            <label className="field">
              <span>Keystore JSON</span>
              <textarea
                rows={7}
                value={document}
                onChange={(event) => setDocument(event.target.value)}
                placeholder='{"version":3,...}'
              />
            </label>
            <label className="field">
              <span>Password</span>
              <input
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                autoComplete="current-password"
              />
            </label>
            <button className="primary" onClick={importKeystore}>
              Import
            </button>
          </>
        )}

        {address && <p className="mono small">Unlocked {address}</p>}
      </section>
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
