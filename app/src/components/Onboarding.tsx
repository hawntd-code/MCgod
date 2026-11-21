import { ArrowRight, CircleAlert, CircleCheck, LoaderCircle } from "lucide-react";
import { useState } from "react";
import { LANGS, useI18n } from "../i18n";
import type { Messages } from "../i18n/locales/es";
import { REQUIRED_JAVA, type EngineState } from "../lib/engine";
import { delay, stagger } from "../lib/motion";
import { BrandMark } from "./BrandMark";

type Checks = Messages["onboarding"]["checks"];
export type Check = { id: "engine" | "java"; label: string; detail: string; state: "ok" | "warn" | "pending" };

/** Traduce el estado del motor a la lista de requisitos del primer arranque. */
export function buildChecks(engine: EngineState, m: Checks): Check[] {
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

function CheckIcon({ state, labels }: { state: Check["state"]; labels: Checks }) {
  if (state === "ok")
    return <CircleCheck className="size-5 shrink-0 animate-pop" strokeWidth={2} aria-label={labels.ok} />;
  if (state === "pending")
    return <LoaderCircle className="size-5 shrink-0 animate-spin" strokeWidth={2} aria-label={labels.pending} />;
  return <CircleAlert className="size-5 shrink-0 animate-pop" strokeWidth={2} aria-label={labels.warn} />;
}

/** Pantalla de bienvenida: solo aparece la primera vez que se abre la app. */
export function Onboarding({ engine, onStart }: { engine: EngineState; onStart: () => Promise<void> }) {
  const { m, lang, setLang } = useI18n();
  const o = m.onboarding;
  const checks = buildChecks(engine, o.checks);
  const [starting, setStarting] = useState(false);

  const start = async () => {
    setStarting(true);
    try {
      await onStart();
    } finally {
      setStarting(false);
    }
  };

  return (
    <main
      className="@container relative h-full overflow-auto bg-brand text-brand-ink"
      aria-labelledby="onboarding-title"
    >
      <div
        aria-hidden
        className="pixel-grid pointer-events-none absolute inset-y-0 right-0 w-[46%] animate-grid [mask-image:linear-gradient(to_left,black_35%,transparent)]"
      />

      <div className="relative mx-auto flex min-h-full max-w-[1180px] flex-col px-[clamp(1.5rem,5cqi,4.5rem)] py-[clamp(1.5rem,4cqi,3.5rem)]">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <BrandMark className="size-7" />
            <span className="font-display text-[1.15rem] font-[780] [font-stretch:118%]">MCgod</span>
          </div>
          <div role="group" aria-label={m.language.label} className="flex items-center gap-1 text-small font-semibold">
            {LANGS.map((l) => (
              <button
                key={l}
                type="button"
                lang={l}
                aria-pressed={lang === l}
                onClick={() => setLang(l)}
                className="rounded-ui px-2.5 py-1 transition-colors duration-150 hover:bg-brand-deep/25 aria-pressed:bg-brand-ink aria-pressed:text-brand"
              >
                {m.language[l]}
              </button>
            ))}
          </div>
        </div>

        <h1
          id="onboarding-title"
          className="mt-[clamp(2.5rem,9cqi,7rem)] animate-rise font-display text-[clamp(3rem,10.5cqi,8rem)] font-[820] leading-[0.92] tracking-[-0.035em] [font-stretch:122%]"
          style={delay(80)}
        >
          {o.title}
        </h1>

        <div className="mt-8 grid gap-x-12 gap-y-8 @[760px]:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
          <div>
            <p
              className="max-w-[44ch] animate-enter text-[1.125rem] leading-[1.6] text-brand-ink-muted"
              style={delay(260)}
            >
              {o.intro}
            </p>
            <div className="animate-enter" style={delay(340)}>
              <button
                type="button"
                onClick={start}
                disabled={starting}
                className="group mt-8 inline-flex h-12 items-center gap-2.5 rounded-ui bg-brand-ink px-6 font-display text-[1.05rem] font-[700] text-brand transition-[transform,filter] duration-150 [font-stretch:110%] hover:brightness-125 active:translate-y-px disabled:opacity-60"
              >
                {o.start}
                {starting ? (
                  <LoaderCircle className="size-5 animate-spin" strokeWidth={2.25} aria-hidden />
                ) : (
                  <ArrowRight
                    className="size-5 transition-transform duration-200 ease-out-expo group-hover:translate-x-1"
                    strokeWidth={2.25}
                    aria-hidden
                  />
                )}
              </button>
              <p className="mt-3 text-small text-brand-ink-muted">{o.languageHint}</p>
            </div>
          </div>

          <div>
            <h2 className="animate-enter font-display text-h2 font-[700] [font-stretch:112%]" style={delay(300)}>
              {o.requirements}
            </h2>
            <ul className="mt-3 border-t border-brand-deep/60">
              {checks.map((c, i) => (
                <li
                  key={c.id}
                  data-check={c.id}
                  data-state={c.state}
                  className="flex animate-enter items-start gap-3 border-b border-brand-deep/60 py-3"
                  style={stagger(i, { base: 380, step: 80 })}
                >
                  <CheckIcon key={c.state} state={c.state} labels={o.checks} />
                  <div className="min-w-0">
                    <p className="font-semibold leading-tight">{c.label}</p>
                    <p className="mt-0.5 text-small text-brand-ink-muted">{c.detail}</p>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    </main>
  );
}
