import { BookOpen, Boxes, Image, Search, Settings, Variable } from "lucide-react";
import { useI18n } from "../i18n";

export type Activity = "elements" | "resources" | "variables" | "search" | "knowledge";

export const activities: { id: Activity; Icon: typeof Boxes }[] = [
  { id: "elements", Icon: Boxes },
  { id: "resources", Icon: Image },
  { id: "variables", Icon: Variable },
  { id: "search", Icon: Search },
  { id: "knowledge", Icon: BookOpen },
];

export function ActivityRail({
  active,
  onChange,
}: {
  active: Activity | null;
  onChange: (a: Activity) => void;
}) {
  const { m } = useI18n();

  return (
    <nav
      aria-label={m.activities.nav}
      className="flex w-12 shrink-0 flex-col items-center gap-1 border-r border-line bg-ground py-2"
    >
      {activities.map(({ id, Icon }) => (
        <button
          key={id}
          type="button"
          title={m.activities[id]}
          aria-label={m.activities[id]}
          aria-current={active === id ? "page" : undefined}
          onClick={() => onChange(id)}
          className="grid size-9 place-items-center rounded-ui text-ink-faint transition-colors duration-150 hover:bg-hover hover:text-ink aria-[current=page]:bg-raised aria-[current=page]:text-brand-bright"
        >
          <Icon className="size-[18px]" strokeWidth={1.75} aria-hidden />
        </button>
      ))}

      <button
        type="button"
        title={m.activities.settings}
        aria-label={m.activities.settings}
        disabled
        className="mt-auto grid size-9 place-items-center rounded-ui text-ink-faint transition-colors duration-150 hover:bg-hover hover:text-ink disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:text-ink-faint"
      >
        <Settings className="size-[18px]" strokeWidth={1.75} aria-hidden />
      </button>
    </nav>
  );
}
