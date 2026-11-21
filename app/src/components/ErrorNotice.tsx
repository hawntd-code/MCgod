import { CircleAlert, X } from "lucide-react";
import { useI18n } from "../i18n";
import type { EngineError } from "../lib/engine";

/** Traduce el código de error del motor. El detalle técnico se muestra aparte. */
export function errorMessage(error: EngineError, m: ReturnType<typeof useI18n>["m"]): string {
  const known = m.errors as Record<string, unknown>;
  const text = known[error.code];
  return typeof text === "string" && error.code !== "title" && error.code !== "dismiss" ? text : m.errors.internal;
}

export function ErrorNotice({ error, onDismiss }: { error: EngineError; onDismiss?: () => void }) {
  const { m } = useI18n();
  return (
    <div role="alert" className="flex animate-enter items-start gap-3 rounded-ui border border-error/50 bg-error/10 px-4 py-3">
      <CircleAlert className="mt-0.5 size-4 shrink-0 text-error" aria-hidden />
      <div className="min-w-0 flex-1">
        <p className="font-semibold text-ink">{m.errors.title}</p>
        <p className="mt-0.5 text-small text-ink-muted">{errorMessage(error, m)}</p>
        {error.detail && (
          <p className="mt-1.5 break-all font-mono text-label text-ink-faint" data-testid="error-detail">
            {error.detail}
          </p>
        )}
      </div>
      {onDismiss && (
        <button
          type="button"
          onClick={onDismiss}
          aria-label={m.errors.dismiss}
          className="grid size-6 shrink-0 place-items-center rounded-ui text-ink-faint hover:bg-hover hover:text-ink"
        >
          <X className="size-3.5" aria-hidden />
        </button>
      )}
    </div>
  );
}
