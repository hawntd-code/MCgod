import { CircleAlert, CircleCheck, House, LoaderCircle } from "lucide-react";
import { REQUIRED_JAVA, type EngineState } from "../lib/engine";

type Check = { label: string; detail: string; state: "ok" | "warn" | "pending" };

function buildChecks(engine: EngineState): Check[] {
  switch (engine.status) {
    case "loading":
      return [
        { label: "Motor en Rust", detail: "Conectando…", state: "pending" },
        { label: `Java ${REQUIRED_JAVA} o superior`, detail: "Buscando…", state: "pending" },
      ];
    case "offline":
      return [
        {
          label: "Motor en Rust",
          detail: "Esta vista corre en el navegador. Abre la app de escritorio para conectarlo.",
          state: "warn",
        },
        { label: `Java ${REQUIRED_JAVA} o superior`, detail: "Se comprueba desde la app de escritorio.", state: "warn" },
      ];
    case "error":
      return [{ label: "Motor en Rust", detail: `No respondió: ${engine.message}`, state: "warn" }];
    case "ready": {
      const { java, appVersion, os } = engine.env;
      const javaOk = java?.major != null && java.major >= REQUIRED_JAVA;
      return [
        { label: "Motor en Rust", detail: `Conectado · versión ${appVersion} · ${os}`, state: "ok" },
        {
          label: `Java ${REQUIRED_JAVA} o superior`,
          detail: !java
            ? `No encontrado. Instala un JDK ${REQUIRED_JAVA} para compilar mods.`
            : javaOk
              ? `${java.vendor ?? "JDK"} ${java.version}`
              : `Tienes Java ${java.version}; los mods de NeoForge necesitan ${REQUIRED_JAVA}.`,
          state: javaOk ? "ok" : "warn",
        },
      ];
    }
  }
}

const phases = [
  { n: 1, name: "Núcleo compatible", text: "Abre workspaces de MCreator, compila el mod y ábrelo en Minecraft." },
  { n: 2, name: "IA en vivo", text: "Conecta tu proveedor y mira a la IA crear ítems, bloques y recetas." },
  { n: 3, name: "Procedures", text: "Los 528 bloques de lógica, con Blockly." },
  { n: 4, name: "Paridad total", text: "Los 37 tipos de elemento, Data Pack, Resource Pack y Bedrock." },
  { n: 5, name: "Editores y visores", text: "Modelos 3D, animaciones, GUIs y editor de código." },
  { n: 6, name: "Conocimiento", text: "Ayuda en cada opción y guías de modding en español." },
];

function CheckIcon({ state }: { state: Check["state"] }) {
  if (state === "ok") return <CircleCheck className="size-5 shrink-0" strokeWidth={2} aria-label="Listo" />;
  if (state === "pending")
    return <LoaderCircle className="size-5 shrink-0 animate-spin" strokeWidth={2} aria-label="Comprobando" />;
  return <CircleAlert className="size-5 shrink-0" strokeWidth={2} aria-label="Atención" />;
}

export function Welcome({ engine }: { engine: EngineState }) {
  const checks = buildChecks(engine);

  return (
    <section className="flex h-full flex-col bg-surface">
      <div role="tablist" className="flex h-9 shrink-0 items-end border-b border-line bg-ground px-2">
        <div
          role="tab"
          aria-selected
          className="flex h-8 items-center gap-2 rounded-t-ui border border-b-0 border-line bg-brand px-3 text-small font-medium text-brand-ink"
        >
          <House className="size-3.5" aria-hidden />
          Inicio
        </div>
      </div>

      <div className="@container relative min-h-0 flex-1 overflow-auto bg-brand text-brand-ink">
        <div
          aria-hidden
          className="pixel-grid pointer-events-none absolute inset-y-0 right-0 w-[46%] [mask-image:linear-gradient(to_left,black_35%,transparent)]"
        />

        <div className="relative mx-auto flex min-h-full max-w-[1180px] flex-col px-[clamp(1.5rem,5cqi,4rem)] pb-8 pt-[clamp(1.75rem,5cqi,4.5rem)]">
          <h1 className="rise-in font-display text-[clamp(2.75rem,9.5cqi,6.75rem)] font-[820] leading-[0.92] tracking-[-0.035em] [font-stretch:122%]">
            Tu mod empieza aquí.
          </h1>

          <div className="mt-7 grid gap-x-10 gap-y-7 @[680px]:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <p className="max-w-[46ch] text-[1.0625rem] leading-[1.6] text-brand-ink-muted">
              Crea ítems, bloques, criaturas y dimensiones para Minecraft. Hazlo tú con los editores o pídeselo a la IA
              del panel derecho y mira cómo lo construye paso a paso.
            </p>

            <div>
              <h2 className="font-display text-h2 font-[700] [font-stretch:112%]">Tu equipo</h2>
              <ul className="mt-3 border-t border-brand-deep/60">
                {checks.map((c) => (
                  <li key={c.label} className="flex items-start gap-3 border-b border-brand-deep/60 py-3">
                    <CheckIcon state={c.state} />
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
            <h2 className="font-display text-h2 font-[700] [font-stretch:112%]">Lo que viene</h2>
            <p className="mt-1 text-small text-brand-ink-muted">
              Estás en la fase 0: la base de la app. Cada fase termina con algo que ya puedes usar.
            </p>
            <ol className="mt-5 grid grid-cols-1 gap-x-6 gap-y-5 @[640px]:grid-cols-2 @[1000px]:grid-cols-3">
              {phases.map((p) => (
                <li key={p.n} className="border-t-2 border-brand-ink pt-3">
                  <p className="font-display text-[0.95rem] font-[750] [font-stretch:112%]">
                    <span className="tabular-nums">{p.n}</span> · {p.name}
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
