import { useEffect, useState } from "react";
import { formatUnits, parseUnits } from "../lib/format";
import { isResourceKind } from "../lib/decoders";
import { defaultTerms, KINDS, TERM_FIELDS } from "../lib/termsDefaults";
import type { Terms } from "../lib/types";
import { Field } from "./ui";

export default function TermsForm({
  value,
  onChange,
}: {
  value: Terms;
  onChange: (terms: Terms) => void;
}) {
  const update = <K extends keyof Terms>(key: K, next: Terms[K]) =>
    onChange({ ...value, [key]: next });

  return (
    <div className="terms-form">
      <div className="grid two">
        <Field label="Resource kind">
          <select
            value={value.kind}
            onChange={(event) => {
              const next = event.target.value;
              if (isResourceKind(next)) onChange(defaultTerms(next));
            }}
          >
            {KINDS.map((kind) => (
              <option key={kind} value={kind}>
                {kind}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Asset">
          <input
            value={value.assetId}
            onChange={(event) => update("assetId", event.target.value)}
          />
        </Field>
      </div>

      <div className="grid three">
        {TERM_FIELDS.filter((field) => !field.advanced).map((field) => {
          if (field.kind === "boolean") {
            return (
              <label className="check" key={field.key}>
                <input
                  type="checkbox"
                  checked={value[field.key]}
                  onChange={(event) => update(field.key, event.target.checked)}
                />
                <span>{field.label}</span>
              </label>
            );
          }
          if (field.kind === "money") {
            return (
              <Field key={field.key} label={field.label}>
                <MoneyInput
                  value={value[field.key]}
                  onChange={(next) => update(field.key, next)}
                />
              </Field>
            );
          }
          return (
            <Field key={field.key} label={field.label}>
              <input
                type="number"
                min={0}
                value={value[field.key]}
                onChange={(event) => update(field.key, Number(event.target.value))}
              />
            </Field>
          );
        })}
      </div>

      <details>
        <summary>Advanced policy windows</summary>
        <div className="grid three">
          {TERM_FIELDS.filter((field) => field.advanced).map((field) => {
            if (field.kind !== "number") return null;
            return (
              <Field key={field.key} label={field.label}>
                <input
                  type="number"
                  min={0}
                  value={value[field.key]}
                  onChange={(event) => update(field.key, Number(event.target.value))}
                />
              </Field>
            );
          })}
        </div>
      </details>
    </div>
  );
}

function MoneyInput({
  value,
  onChange,
}: {
  value: bigint;
  onChange: (value: bigint) => void;
}) {
  const [text, setText] = useState(() => formatUnits(value));
  useEffect(() => setText(formatUnits(value)), [value]);
  return (
    <input
      inputMode="decimal"
      value={text}
      onChange={(event) => {
        setText(event.target.value);
        try {
          onChange(parseUnits(event.target.value));
        } catch {
          // Keep the text while it is incomplete; the chain validates the final value.
        }
      }}
      placeholder="0.0"
    />
  );
}
