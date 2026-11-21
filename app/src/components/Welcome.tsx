import { CircleAlert, CircleCheck, House, LoaderCircle } from "lucide-react";
import { useI18n } from "../i18n";
import type { Messages } from "../i18n/locales/es";
import { REQUIRED_JAVA, type EngineState } from "../lib/engine";

export type Check = { id: "engine" | "java"; label: string; detail: string; state: "ok" | "warn" | "pending" };

/** Traduce el estado del motor a la lista de requisitos que muestra la bienvenida. */
export function buildChecks(engine: EngineState, m: Messages["welcome"]["checks"]): Check[] {
  const javaLabel = m.java(REQUIRED_JAVA);
  switch (engine.status) {
    case "loading":
      return [
        { id: "engine", label: m.engine, detail: m.connecting, state: "pending" },
        { id: "java", label: javaLabel, detail: m.searching, state: "pending" },
      ];
    case "offline":
      return [
        { id: "engine", label: m.engine, detail: m.engineOffline, state: "warn" },
        { id: "java", label: javaLabel, detail: m.javaOffline, state: "warn" },
      ];
    case "error":
      return [{ id: "engine", label: m.engine, detail: m.engineError(engine.message), state: "warn" }];
    case "ready": {
      const { java, appVersion, os } = engine.env;
      const javaOk = java?.major != null && java.major >= REQUIRED_JAVA;
      return [
        { id: "engine", label: m.engine, detail: m.engineReady(appVersion, os), state: "ok" },
        {
          id: "java",
          label: javaLabel,
          detail: !java
            ? m.javaMissing(REQUIRED_JAVA)
            : javaOk
              ? `${java.vendor ?? "JDK"} ${java.version}`
              : m.javaOld(java.version, REQUIRED_JAVA),
          state: javaOk ? "ok" : "warn",
        },
      ];
    }
  }
}

function CheckIcon({ state, labels }: { state: Check["state"]; labels: Messages["welcome"]["checks"] }) {
  if (state === "ok") return <CircleCheck className="size-5 shrink-0" strokeWidth={2} aria-label={labels.ok} />;
  if (state === "pending")
    return <LoaderCircle className="size-5 shrink-0 animate-spin" strokeWidth={2} aria-label={labels.pending} />;
  return <CircleAlert className="size-5 shrink-0" strokeWidth={2} aria-label={labels.warn} />;
}

export function Welcome({ engine }: { engine: EngineState }) {
  const { m } = useI18n();
  const w = m.welcome;
  const checks = buildChecks(engine, w.checks);

  return (
    <section className="flex h-full flex-col bg-surface" aria-labelledby="welcome-title">
      <div role="tablist" className="flex h-9 shrink-0 items-end border-b border-line bg-ground px-2">
        <div
          role="tab"
          aria-selected
          className="flex h-8 items-center gap-2 rounded-t-ui border border-b-0 border-line bg-brand px-3 text-small font-medium text-brand-ink"
        >
          <House className="size-3.5" aria-hidden />
          {w.tab}
        </div>
      </div>

      <div className="@container relative min-h-0 flex-1 overflow-auto bg-brand text-brand-ink">
        <div
          aria-hidden
          className="pixel-grid pointer-events-none absolute inset-y-0 right-0 w-[46%] [mask-image:linear-gradient(to_left,black_35%,transparent)]"
        />

        <div className="relative mx-auto flex min-h-full max-w-[1180px] flex-col px-[clamp(1.5rem,5cqi,4rem)] pb-8 pt-[clamp(1.75rem,5cqi,4.5rem)]">
          <h1
            id="welcome-title"
            className="rise-in font-display text-[clamp(2.75rem,9.5cqi,6.75rem)] font-[820] leading-[0.92] tracking-[-0.035em] [font-stretch:122%]"
          >
            {w.title}
          </h1>

          <div className="mt-7 grid gap-x-10 gap-y-7 @[680px]:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <p className="max-w-[46ch] text-[1.0625rem] leading-[1.6] text-brand-ink-muted">{w.intro}</p>

            <div>
              <h2 className="font-display text-h2 font-[700] [font-stretch:112%]">{w.requirements}</h2>
              <ul className="mt-3 border-t border-brand-deep/60">
                {checks.map((c) => (
                  <li key={c.id} data-check={c.id} data-state={c.state} className="flex items-start gap-3 border-b border-brand-deep/60 py-3">
                    <CheckIcon state={c.state} labels={w.checks} />
                    <div className="min-w-0">
                      <p className="font-semibold leading-tight">{c.label}</p>
                      <p className="mt-0.5 text-small text-brand-ink-muted">{c.detail}</p>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          </div>

          <div className="mt-auto pt-10">
            <h2 className="font-display text-h2 font-[700] [font-stretch:112%]">{w.roadmap}</h2>
            <p className="mt-1 text-small text-brand-ink-muted">{w.roadmapIntro}</p>
            <ol className="mt-5 grid grid-cols-1 gap-x-6 gap-y-5 @[640px]:grid-cols-2 @[1000px]:grid-cols-3">
              {w.phases.map((p, i) => (
                <li key={p.name} className="border-t-2 border-brand-ink pt-3">
                  <p className="font-display text-[0.95rem] font-[750] [font-stretch:112%]">
                    <span className="tabular-nums">{i + 1}</span> · {p.name}
                  </p>
                  <p className="mt-1 text-small text-brand-ink-muted">{p.text}</p>
                </li>
              ))}
            </ol>
          </div>
        </div>
      </div>
    </section>
  );
}
