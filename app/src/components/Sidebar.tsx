import { Search } from "lucide-react";
import type { Activity } from "./ActivityRail";

const content: Record<Activity, { title: string; emptyTitle: string; emptyText: string; phase: string }> = {
  elements: {
    title: "Elementos",
    emptyTitle: "Ningún proyecto abierto",
    emptyText:
      "Aquí verás los ítems, bloques, criaturas, dimensiones y procedures de tu mod, agrupados por tipo y con su vista previa.",
    phase: "Abrir workspaces de MCreator llega en la fase 1.",
  },
  resources: {
    title: "Recursos",
    emptyTitle: "Sin recursos todavía",
    emptyText:
      "Texturas, sonidos, modelos 3D y estructuras del proyecto. Podrás pintarlos, importarlos y previsualizarlos sin salir de la app.",
    phase: "El editor de texturas llega en la fase 2.",
  },
  variables: {
    title: "Variables",
    emptyTitle: "Sin variables",
    emptyText:
      "Las variables globales, de mundo y de jugador que usan tus procedures para guardar datos entre partidas.",
    phase: "Llegan con los procedures en la fase 3.",
  },
  search: {
    title: "Buscar",
    emptyTitle: "Nada que buscar aún",
    emptyText: "Busca elementos, campos, referencias rotas y texto dentro de todo el proyecto.",
    phase: "La búsqueda global llega en la fase 5.",
  },
  knowledge: {
    title: "Base de conocimiento",
    emptyTitle: "Qué hace cada opción",
    emptyText:
      "Guías de modding y la explicación de cada campo de cada editor, en español. La misma información que consulta la IA.",
    phase: "Se integra en la fase 6, con 729 textos de ayuda de MCreator como punto de partida.",
  },
};

export function Sidebar({ activity }: { activity: Activity }) {
  const c = content[activity];

  return (
    <aside className="flex h-full flex-col bg-surface">
      <div className="flex h-10 shrink-0 items-center px-4">
        <h2 className="font-display text-h3 font-[650] [font-stretch:110%]">{c.title}</h2>
      </div>

      {activity === "elements" && (
        <div className="px-3 pb-3">
          <label className="flex h-8 items-center gap-2 rounded-ui border border-line bg-ground px-2.5 text-ink-faint">
            <Search className="size-3.5 shrink-0" aria-hidden />
            <span className="sr-only">Filtrar elementos</span>
            <input
              disabled
              placeholder="Filtrar elementos"
              className="w-full bg-transparent text-small text-ink outline-none placeholder:text-ink-faint disabled:cursor-not-allowed"
            />
          </label>
        </div>
      )}

      <div className="flex flex-1 flex-col justify-center px-6 pb-16">
        <h3 className="text-h3 font-semibold text-ink">{c.emptyTitle}</h3>
        <p className="mt-2 text-small text-ink-muted">{c.emptyText}</p>
        <p className="mt-4 text-label text-ink-faint">{c.phase}</p>
      </div>
    </aside>
  );
}
