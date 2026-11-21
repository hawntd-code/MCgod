import { FileJson, LayoutDashboard, X } from "lucide-react";
import { useI18n } from "../i18n";
import type { WorkspaceSummary } from "../lib/engine";
import { ElementView } from "./ElementView";
import { WorkspaceOverview } from "./WorkspaceOverview";

export const OVERVIEW = "__overview__";

type Props = {
  workspace: WorkspaceSummary;
  tabs: string[];
  active: string;
  onActivate: (tab: string) => void;
  onCloseTab: (name: string) => void;
  onOpenElement: (name: string) => void;
  onCloseProject: () => void;
};

/** Área central con pestañas: el resumen del proyecto y un elemento por pestaña. */
export function EditorArea({ workspace, tabs, active, onActivate, onCloseTab, onOpenElement, onCloseProject }: Props) {
  const { m } = useI18n();
  const element = workspace.elements.find((e) => e.name === active);

  const tabClass = (on: boolean) =>
    `flex h-8 items-center gap-2 rounded-t-ui border border-b-0 px-3 text-small transition-colors duration-150 ${
      on ? "border-line bg-surface text-ink shadow-[inset_0_2px_0_var(--color-brand)]" : "border-transparent text-ink-faint hover:text-ink"
    }`;

  return (
    <section className="flex h-full flex-col bg-surface">
      <div role="tablist" className="flex h-9 shrink-0 items-end gap-px overflow-x-auto border-b border-line bg-ground px-2">
        <button
          type="button"
          role="tab"
          aria-selected={active === OVERVIEW}
          onClick={() => onActivate(OVERVIEW)}
          className={tabClass(active === OVERVIEW)}
        >
          <LayoutDashboard className="size-3.5" aria-hidden />
          {m.workspace.overview}
        </button>
        {tabs.map((name) => (
          <div key={name} className={`${tabClass(active === name)} pr-1`}>
            <button
              type="button"
              role="tab"
              aria-selected={active === name}
              onClick={() => onActivate(name)}
              className="flex items-center gap-2"
            >
              <FileJson className="size-3.5" aria-hidden />
              {name}
            </button>
            <button
              type="button"
              onClick={() => onCloseTab(name)}
              aria-label={m.workspace.closeTab(name)}
              title={m.workspace.closeTab(name)}
              className="grid size-5 place-items-center rounded-[3px] text-ink-faint hover:bg-hover hover:text-ink"
            >
              <X className="size-3" aria-hidden />
            </button>
          </div>
        ))}
      </div>

      <div role="tabpanel" className="min-h-0 flex-1 overflow-auto">
        {element ? (
          <ElementView key={element.name} element={element} />
        ) : (
          <WorkspaceOverview workspace={workspace} onOpenElement={onOpenElement} onClose={onCloseProject} />
        )}
      </div>
    </section>
  );
}
