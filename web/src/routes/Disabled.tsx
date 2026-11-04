import { Panel } from "../components/ui";

export default function Disabled({ title, reason }: { title: string; reason: string }) {
  return (
    <Panel>
      <div className="panel-head">
        <h1>{title}</h1>
        <span className="pill warn">not implemented</span>
      </div>
      <p className="muted">{reason}</p>
      <p className="muted small">
        This screen is intentionally honest: it does not show mock data or a control that
        cannot work.
      </p>
    </Panel>
  );
}
