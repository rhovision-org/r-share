import { NavLink, Outlet } from "react-router-dom";
import { useWallet } from "../lib/store";
import { shortId } from "../lib/format";
import Unlock from "./Unlock";

const TABS = [
  ["/", "Explore"],
  ["/provider", "Provider"],
  ["/me", "My bookings"],
  ["/admin", "Admin"],
  ["/history", "History"],
  ["/settings", "Settings"],
] as const;

export default function Layout() {
  const { status, address, node, error, lock } = useWallet();

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="mark">r</span>
          <div>
            <strong>r-share</strong>
            <small>reference client</small>
          </div>
        </div>
        <nav className="tabs">
          {TABS.map(([to, label]) => (
            <NavLink key={to} to={to} end={to === "/"}>
              {label}
            </NavLink>
          ))}
        </nav>
        <div className="wallet-chip">
          <span className={`dot ${status}`} />
          <span>{address ? shortId(address, 6) : status}</span>
          {address && (
            <button className="ghost" onClick={lock}>
              Lock
            </button>
          )}
        </div>
      </header>
      <div className="node-line">
        <span className="mono">{node}</span>
        {error && <span className="error-inline">{error}</span>}
      </div>
      <main className="content">
        <Outlet />
      </main>
      <footer className="footer">
        <span>
          Devnet only · no real funds · <a href="https://github.com/rhovision-org/r-share">source (AGPL-3.0-or-later)</a>
        </span>
      </footer>
      <Unlock />
    </div>
  );
}
