import { useEffect, useState } from "react";
import { formatUnits, parseUnits } from "../lib/format";
import { defaultTerms, KINDS, TERM_FIELDS } from "../lib/termsDefaults";
import type { ResourceKind, Terms } from "../lib/types";
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
            onChange={(event) =>
              onChange(defaultTerms(event.target.value as ResourceKind))
            }
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
        {TERM_FIELDS.filter((field) => !field.advanced).map((field) =>
          field.key === "instantBook" || field.key === "contiguous" ? (
            <label className="check" key={field.key}>
              <input
                type="checkbox"
                checked={Boolean(value[field.key])}
                onChange={(event) =>
                  update(field.key, event.target.checked as Terms[typeof field.key])
                }
              />
              <span>{field.label}</span>
            </label>
          ) : (
            <Field key={field.key} label={field.label}>
              {field.money ? (
                <MoneyInput
                  value={Number(value[field.key])}
                  onChange={(next) => update(field.key, next as Terms[typeof field.key])}
                />
              ) : (
                <input
                  type="number"
                  min={0}
                  value={Number(value[field.key])}
                  onChange={(event) =>
                    update(field.key, Number(event.target.value) as Terms[typeof field.key])
                  }
                />
              )}
            </Field>
          ),
        )}
      </div>

      <details>
        <summary>Advanced policy windows</summary>
        <div className="grid three">
          {TERM_FIELDS.filter((field) => field.advanced).map((field) => (
            <Field key={field.key} label={field.label}>
              <input
                type="number"
                min={0}
                value={Number(value[field.key])}
                onChange={(event) =>
                  update(field.key, Number(event.target.value) as Terms[typeof field.key])
                }
              />
            </Field>
          ))}
        </div>
      </details>
    </div>
  );
}

function MoneyInput({
  value,
  onChange,
}: {
  value: number;
  onChange: (value: number) => void;
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
          const parsed = Number(parseUnits(event.target.value));
          if (Number.isSafeInteger(parsed)) onChange(parsed);
        } catch {
          // Keep the text while it is incomplete; the chain validates the final value.
        }
      }}
      placeholder="0.0"
    />
  );
}
