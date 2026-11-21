import { CircleAlert, CircleCheck, CircleStop, LoaderCircle, Square, Trash2 } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useI18n } from "../i18n";
import type { OutputLine } from "../lib/engine";
import { formatDuration, isRunning, toneOf, type LineTone, type RunState } from "../lib/gradle-run";
import { codegenLines } from "./CodegenLines";
import { errorMessage } from "./ErrorNotice";

type Tab = "gradle" | "ai";

const toneClass: Record<LineTone, string> = {
  error: "text-error",
  warn: "text-warn",
  ok: "text-ok font-semibold",
  task: "text-ink-faint",
  plain: "text-ink-muted",
};

type Props = {
  run: RunState;
  lines: OutputLine[];
  dropped: number;
  onStop: () => void;
  onClear: () => void;
};

function RunStatus({ run }: { run: RunState }) {
  const { m } = useI18n();
  const c = m.console;
  if (run.status === "idle") return null;
  const task = c.taskNames[run.task] ?? run.task;

  let icon = <LoaderCircle className="size-3.5 animate-spin" aria-hidden />;
  let text = c.starting;
  let tone = "text-ink-muted";
  if (run.status === "running") text = c.running(task);
  if (run.status === "done") {
    if (run.outcome === "success") {
      icon = <CircleCheck className="size-3.5" aria-hidden />;
      text = c.success(task, formatDuration(run.durationMs));
      tone = "text-ok";
    } else if (run.outcome === "failed") {
      icon = <CircleAlert className="size-3.5" aria-hidden />;
      text = c.failed(task, String(run.exitCode ?? "?"));
      tone = "text-error";
    } else {
      icon = <CircleStop className="size-3.5" aria-hidden />;
      text = c.cancelled(task);
    }
  }
  if (run.status === "error") {
    icon = <CircleAlert className="size-3.5" aria-hidden />;
    text = `${c.startFailed(task)}: ${errorMessage(run.error, m)}`;
    tone = "text-error";
  }
  return (
    <span role="status" className={`flex min-w-0 items-center gap-1.5 truncate text-small ${tone}`}>
      <span key={run.status} className="grid shrink-0 animate-pop place-items-center">
        {icon}
      </span>
      <span key={text} className="animate-fade truncate">
        {text}
      </span>
    </span>
  );
}

/**
 * Señal de redstone en el borde inferior de la cabecera: recorre el cable mientras Gradle trabaja
 * y lo llena al terminar (verde si salió bien, rojo si falló). Es decorativa: el texto de estado ya lo dice.
 */
function RunSignal({ run }: { run: RunState }) {
  const tone = run.status !== "idle" && run.task === "runClient" ? "text-signal" : "text-brand";
  let line = null;
  if (isRunning(run)) line = <span className={`signal ${tone}`} />;
  else if (run.status === "done" && run.outcome === "success") line = <span className="signal-done text-ok" />;
  else if (run.status === "error" || (run.status === "done" && run.outcome === "failed"))
    line = <span className="signal-done text-error" />;
  if (!line) return null;
  return (
    <span className="pointer-events-none absolute inset-x-0 -bottom-px" aria-hidden data-testid="run-signal">
      {line}
    </span>
  );
}

export function ConsolePanel({ run, lines, dropped, onStop, onClear }: Props) {
  const { m } = useI18n();
  const [tab, setTab] = useState<Tab>("gradle");
  const scroller = useRef<HTMLDivElement>(null);
  const stickToBottom = useRef(true);
  const running = isRunning(run);

  // Al lanzar una tarea, la consola de Gradle pasa al frente y vuelve a seguir el final.
  useEffect(() => {
    if (run.status === "starting") {
      setTab("gradle");
      stickToBottom.current = true;
    }
  }, [run.status]);

  // Sigue el final de la salida, salvo que la persona haya subido para leer.
  useLayoutEffect(() => {
    const el = scroller.current;
    if (el && stickToBottom.current) el.scrollTop = el.scrollHeight;
  }, [lines, tab]);

  const onScroll = () => {
    const el = scroller.current;
    if (el) stickToBottom.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
  };

  const tabs: { id: Tab; label: string }[] = [
    { id: "gradle", label: m.console.gradle },
    { id: "ai", label: m.console.ai },
  ];

  return (
    <section className="flex h-full flex-col bg-ground">
      <div className="relative flex h-9 shrink-0 items-center gap-3 border-b border-line px-2">
        <RunSignal run={run} />
        <div role="tablist" className="flex shrink-0 items-center gap-1">
          {tabs.map((t) => (
            <button
              key={t.id}
              id={`console-tab-${t.id}`}
              type="button"
              role="tab"
              aria-selected={tab === t.id}
              aria-controls="console-panel"
              onClick={() => setTab(t.id)}
              className="h-7 rounded-ui px-2.5 text-small text-ink-faint transition-colors duration-150 hover:text-ink aria-selected:bg-raised aria-selected:text-ink"
            >
              {t.label}
            </button>
          ))}
        </div>

        {tab === "gradle" && (
          <>
            <div className="min-w-0 flex-1">
              <RunStatus run={run} />
            </div>
            {running && (
              <button
                type="button"
                onClick={onStop}
                className="flex h-7 shrink-0 animate-pop items-center gap-1.5 rounded-ui px-2.5 text-small text-ink-muted transition-[color,background-color,transform] duration-150 hover:bg-hover hover:text-ink active:translate-y-px"
              >
                <Square className="size-3 fill-current" aria-hidden />
                {m.topBar.stop}
              </button>
            )}
            <button
              type="button"
              onClick={onClear}
              disabled={lines.length === 0 && run.status === "idle"}
              aria-label={m.console.clear}
              title={m.console.clear}
              className="grid size-7 shrink-0 place-items-center rounded-ui text-ink-faint transition-colors duration-150 hover:bg-hover hover:text-ink disabled:opacity-40 disabled:hover:bg-transparent"
            >
              <Trash2 className="size-3.5" aria-hidden />
            </button>
          </>
        )}
      </div>

      <div
        id="console-panel"
        role="tabpanel"
        aria-labelledby={`console-tab-${tab}`}
        ref={scroller}
        onScroll={onScroll}
        className="min-h-0 flex-1 overflow-auto px-4 py-2 font-mono text-[0.78rem] leading-[1.5] select-text"
      >
        {tab === "ai" ? (
          <p key="ai" className="animate-fade py-1 text-ink-faint">
            {m.console.aiEmpty}
          </p>
        ) : lines.length === 0 ? (
          <div key="empty" className="animate-fade py-1 text-ink-faint">
            <p>{m.console.gradleEmpty}</p>
            {running && <p className="mt-1 animate-fade">{m.console.firstRun}</p>}
          </div>
        ) : (
          <div key="output" data-testid="gradle-output" className="animate-fade">
            {dropped > 0 && <p className="pb-1 text-ink-faint">{m.console.hidden(dropped)}</p>}
            {lines.map((line, i) =>
              line.stream === "mcgod" ? (
                codegenLines(line.codegen, m).map((l, j) => (
                  <div key={`${i}-${j}`} data-source="mcgod" className={`whitespace-pre-wrap break-all ${toneClass[l.tone]}`}>
                    <span className="text-brand">{m.codegen.prefix} · </span>
                    {l.text}
                  </div>
                ))
              ) : (
                <div key={i} className={`whitespace-pre-wrap break-all ${toneClass[toneOf(line.text)]}`}>
                  {line.text || " "}
                </div>
              ),
            )}
          </div>
        )}
      </div>
    </section>
  );
}
