import { BookOpen, Boxes, Image, Search, Settings, Variable } from "lucide-react";

export type Activity = "elements" | "resources" | "variables" | "search" | "knowledge";

export const activities: { id: Activity; label: string; Icon: typeof Boxes }[] = [
  { id: "elements", label: "Elementos del mod", Icon: Boxes },
  { id: "resources", label: "Texturas, sonidos y modelos", Icon: Image },
  { id: "variables", label: "Variables", Icon: Variable },
  { id: "search", label: "Buscar en el proyecto", Icon: Search },
  { id: "knowledge", label: "Base de conocimiento", Icon: BookOpen },
];

export function ActivityRail({
  active,
  onChange,
}: {
  active: Activity | null;
  onChange: (a: Activity) => void;
}) {
  return (
    <nav
      aria-label="Secciones"
      className="flex w-12 shrink-0 flex-col items-center gap-1 border-r border-line bg-ground py-2"
    >
      {activities.map(({ id, label, Icon }) => (
        <button
          key={id}
          type="button"
          title={label}
          aria-label={label}
          aria-current={active === id ? "page" : undefined}
          onClick={() => onChange(id)}
          className="grid size-9 place-items-center rounded-ui text-ink-faint transition-colors duration-150 hover:bg-hover hover:text-ink aria-[current=page]:bg-raised aria-[current=page]:text-brand-bright"
        >
          <Icon className="size-[18px]" strokeWidth={1.75} aria-hidden />
        </button>
      ))}

      <button
        type="button"
        title="Ajustes"
        aria-label="Ajustes"
        disabled
        className="mt-auto grid size-9 place-items-center rounded-ui text-ink-faint transition-colors duration-150 hover:bg-hover hover:text-ink disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:text-ink-faint"
      >
        <Settings className="size-[18px]" strokeWidth={1.75} aria-hidden />
      </button>
    </nav>
  );
}
