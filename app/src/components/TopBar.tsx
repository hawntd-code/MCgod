import {
  ChevronDown,
  Hammer,
  PanelBottom,
  PanelLeft,
  PanelRight,
  Play,
} from "lucide-react";
import { BrandMark } from "./BrandMark";

export type PanelKey = "sidebar" | "console" | "ai";

const toggles: { key: PanelKey; label: string; shortcut: string; Icon: typeof PanelLeft }[] = [
  { key: "sidebar", label: "Panel de elementos", shortcut: "Ctrl+B", Icon: PanelLeft },
  { key: "console", label: "Consola", shortcut: "Ctrl+J", Icon: PanelBottom },
  { key: "ai", label: "Asistente de IA", shortcut: "Ctrl+L", Icon: PanelRight },
];

export function TopBar({
  open,
  onToggle,
}: {
  open: Record<PanelKey, boolean>;
  onToggle: (key: PanelKey) => void;
}) {
  const noProject = "Abre o crea un proyecto primero";

  return (
    <header className="flex h-11 shrink-0 items-center gap-3 border-b border-line bg-ground px-3">
      <div className="flex items-center gap-2 pr-2">
        <BrandMark className="size-5" />
        <span className="font-display text-[0.95rem] font-[750] tracking-[-0.01em] [font-stretch:118%]">
          MCgod
        </span>
      </div>

      <button
        type="button"
        disabled
        title="Los proyectos llegan en la fase 1"
        className="flex h-7 items-center gap-2 rounded-ui border border-line px-2.5 text-small text-ink-muted disabled:cursor-not-allowed"
      >
        Sin proyecto abierto
        <ChevronDown className="size-3.5" aria-hidden />
      </button>

      <div className="ml-auto flex items-center gap-1.5">
        <button
          type="button"
          disabled
          title={noProject}
          className="flex h-7 items-center gap-1.5 rounded-ui px-2.5 text-small font-medium text-ink-muted transition-colors duration-150 hover:bg-hover disabled:cursor-not-allowed disabled:text-ink-faint disabled:hover:bg-transparent"
        >
          <Hammer className="size-3.5" aria-hidden />
          Compilar
        </button>
        <button
          type="button"
          disabled
          title={noProject}
          className="flex h-7 items-center gap-1.5 rounded-ui bg-signal px-3 text-small font-semibold text-signal-ink transition-[filter,opacity] duration-150 hover:brightness-110 active:brightness-95 disabled:cursor-not-allowed disabled:opacity-45 disabled:hover:brightness-100"
        >
          <Play className="size-3.5 fill-current" aria-hidden />
          Probar en Minecraft
        </button>

        <span className="mx-2 h-5 w-px bg-line" aria-hidden />

        {toggles.map(({ key, label, shortcut, Icon }) => (
          <button
            key={key}
            type="button"
            onClick={() => onToggle(key)}
            aria-pressed={open[key]}
            title={`${open[key] ? "Ocultar" : "Mostrar"} ${label.toLowerCase()} (${shortcut})`}
            className="grid size-7 place-items-center rounded-ui text-ink-faint transition-colors duration-150 hover:bg-hover hover:text-ink aria-pressed:text-ink"
          >
            <Icon className="size-4" aria-hidden />
            <span className="sr-only">{label}</span>
          </button>
        ))}
      </div>
    </header>
  );
}
