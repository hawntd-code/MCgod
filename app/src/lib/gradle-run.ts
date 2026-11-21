import { useCallback, useRef, useState } from "react";
import { toEngineError, type Backend, type EngineError, type GradleOutcome, type GradleTask, type OutputLine } from "./engine";

/** Líneas que se guardan como máximo; Minecraft en desarrollo escribe muchísimo. */
export const MAX_LINES = 4000;

export type RunState =
  | { status: "idle" }
  | { status: "starting"; task: GradleTask }
  | { status: "running"; task: GradleTask; startedAt: number }
  | { status: "done"; task: GradleTask; outcome: GradleOutcome; exitCode: number | null; durationMs: number }
  | { status: "error"; task: GradleTask; error: EngineError };

export type LineTone = "error" | "warn" | "ok" | "task" | "plain";

/** Color de cada línea según lo que dice: errores, avisos, tareas de Gradle y el resultado. */
export function toneOf(text: string): LineTone {
  if (/BUILD SUCCESSFUL/.test(text)) return "ok";
  if (/BUILD FAILED|FAILURE:|\b(ERROR|FATAL)\b|^\S.*\.java:\d+: error:|Exception\b|^\s+at [\w$.]+\(/.test(text))
    return "error";
  if (/\bWARN(ING)?\b|warning:|Deprecated Gradle features/.test(text)) return "warn";
  if (/^> Task :/.test(text)) return "task";
  return "plain";
}

export function isRunning(run: RunState): run is Extract<RunState, { status: "starting" | "running" }> {
  return run.status === "starting" || run.status === "running";
}

/** `125 300 ms` → `2 min 05 s`; `9 400 ms` → `9 s`. */
export function formatDuration(ms: number): string {
  const total = Math.max(0, Math.round(ms / 1000));
  const min = Math.floor(total / 60);
  const sec = total % 60;
  return min > 0 ? `${min} min ${String(sec).padStart(2, "0")} s` : `${sec} s`;
}

/** Estado de una tarea de Gradle y su salida, con un tope de líneas. */
export function useGradleRun(backend: Backend) {
  const [run, setRun] = useState<RunState>({ status: "idle" });
  const [lines, setLines] = useState<OutputLine[]>([]);
  const [dropped, setDropped] = useState(0);
  const all = useRef<OutputLine[]>([]);
  const droppedCount = useRef(0);
  const pending = useRef<OutputLine[]>([]);
  const frame = useRef<number | null>(null);

  // Las líneas se juntan y se pintan como mucho una vez por cuadro.
  const flush = useCallback(() => {
    frame.current = null;
    const incoming = pending.current;
    pending.current = [];
    if (incoming.length === 0) return;
    let next = all.current.concat(incoming);
    const extra = next.length - MAX_LINES;
    if (extra > 0) {
      next = next.slice(extra);
      droppedCount.current += extra;
    }
    all.current = next;
    setLines(next);
    setDropped(droppedCount.current);
  }, []);

  const reset = () => {
    pending.current = [];
    all.current = [];
    droppedCount.current = 0;
    setLines([]);
    setDropped(0);
  };

  const schedule = useCallback(() => {
    if (frame.current !== null) return;
    frame.current = typeof requestAnimationFrame === "function" ? requestAnimationFrame(flush) : (setTimeout(flush, 16) as unknown as number);
  }, [flush]);

  const start = useCallback(
    async (task: GradleTask) => {
      reset();
      setRun({ status: "starting", task });
      try {
        await backend.runGradle(task, (event) => {
          if (event.kind === "lines") {
            pending.current.push(...event.lines);
            schedule();
          } else if (event.kind === "codegen") {
            // La consola escribe estos avisos en el idioma de la persona.
            pending.current.push({ stream: "mcgod", text: "", codegen: event });
            schedule();
          } else {
            flush();
            setRun({ status: "done", task, outcome: event.outcome, exitCode: event.exitCode, durationMs: event.durationMs });
          }
        });
        setRun((current) => (current.status === "starting" ? { status: "running", task, startedAt: Date.now() } : current));
      } catch (err) {
        setRun({ status: "error", task, error: toEngineError(err) });
      }
    },
    [backend, flush, schedule],
  );

  const stop = useCallback(() => backend.stopGradle().catch(() => undefined), [backend]);

  const clear = useCallback(() => {
    reset();
    setRun((current) => (isRunning(current) ? current : { status: "idle" }));
  }, []);

  return { run, lines, dropped, start, stop, clear };
}
