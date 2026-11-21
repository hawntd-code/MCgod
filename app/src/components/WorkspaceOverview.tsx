import { CircleAlert, LogOut } from "lucide-react";
import { useI18n } from "../i18n";
import type { WorkspaceSummary } from "../lib/engine";
import { generatorLabel, groupByType, needsReview } from "../lib/format";

type Props = {
  workspace: WorkspaceSummary;
  onOpenElement: (name: string) => void;
  onClose: () => void;
};

export function WorkspaceOverview({ workspace, onOpenElement, onClose }: Props) {
  const { m, lang } = useI18n();
  const w = m.workspace;
  const groups = groupByType(workspace.elements, m, lang).sort((a, b) => b.elements.length - a.elements.length);
  const max = Math.max(1, ...groups.map((g) => g.elements.length));
  const review = workspace.elements.filter(needsReview);

  const facts: { label: string; value: string; mono?: boolean }[] = [
    { label: w.modid, value: workspace.modid, mono: true },
    { label: w.version, value: workspace.version ?? "—" },
    { label: w.generator, value: generatorLabel(workspace.generator) ?? w.noGenerator },
    { label: w.author, value: workspace.author ?? "—" },
  ];
  const counts: { label: string; value: number | string }[] = [
    { label: w.elements, value: workspace.elements.length },
    { label: w.variables, value: workspace.variables },
    { label: w.sounds, value: workspace.sounds },
    { label: w.tags, value: workspace.tags },
    { label: w.languages, value: workspace.languages.join(", ") || "—" },
  ];

  return (
    <div className="mx-auto max-w-[1080px] px-[clamp(1.5rem,4vw,3rem)] py-10">
      <div className="flex flex-wrap items-start justify-between gap-x-6 gap-y-4">
        <div className="min-w-0">
          <h1 className="font-display text-[2.75rem] font-[800] leading-[0.95] tracking-[-0.028em] [font-stretch:118%]">
            {workspace.name}
          </h1>
          {workspace.description && <p className="mt-3 max-w-[64ch] text-ink-muted">{workspace.description}</p>}
        </div>
        <button
          type="button"
          onClick={onClose}
          className="inline-flex h-8 items-center gap-2 rounded-ui border border-line px-3 text-small text-ink-muted transition-colors duration-150 hover:bg-hover hover:text-ink"
        >
          <LogOut className="size-3.5" aria-hidden />
          {w.close}
        </button>
      </div>

      <dl className="mt-8 grid grid-cols-2 gap-x-8 gap-y-4 border-y border-line py-5 lg:grid-cols-4">
        {facts.map((f) => (
          <div key={f.label} className="min-w-0">
            <dt className="text-label text-ink-faint">{f.label}</dt>
            <dd className={`mt-1 truncate font-semibold text-ink ${f.mono ? "font-mono text-small" : ""}`}>{f.value}</dd>
          </div>
        ))}
      </dl>
      <p className="mt-2 truncate font-mono text-label text-ink-faint" title={workspace.file}>
        {workspace.file}
      </p>

      <dl className="mt-6 flex flex-wrap gap-x-8 gap-y-3">
        {counts.map((c) => (
          <div key={c.label} className="flex items-baseline gap-2">
            <dt className="text-small text-ink-muted">{c.label}</dt>
            <dd className="font-semibold tabular-nums text-ink">{c.value}</dd>
          </div>
        ))}
      </dl>

      <div className="mt-10 grid gap-x-12 gap-y-10 lg:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)]">
        <section aria-labelledby="by-type">
          <h2 id="by-type" className="font-display text-h3 font-[680] [font-stretch:110%]">
            {w.byType}
          </h2>
          <ul className="mt-3 space-y-2">
            {groups.map((g) => (
              <li key={g.type} className="grid grid-cols-[minmax(0,11rem)_1fr_2rem] items-center gap-3 text-small">
                <span className="truncate text-ink">{g.label}</span>
                <span className="h-2 rounded-[2px] bg-raised" aria-hidden>
                  <span
                    className="block h-full rounded-[2px] bg-brand"
                    style={{ width: `${(g.elements.length / max) * 100}%` }}
                  />
                </span>
                <span className="text-right tabular-nums text-ink-muted">{g.elements.length}</span>
              </li>
            ))}
          </ul>
        </section>

        <section aria-labelledby="to-review">
          <h2 id="to-review" className="font-display text-h3 font-[680] [font-stretch:110%]">
            {w.attention}
          </h2>
          {review.length === 0 ? (
            <p className="mt-3 text-small text-ink-muted">{w.allGood}</p>
          ) : (
            <ul className="mt-3 divide-y divide-line border-y border-line">
              {review.map((e) => (
                <li key={e.name}>
                  <button
                    type="button"
                    onClick={() => onOpenElement(e.name)}
                    className="flex w-full items-start gap-2.5 px-1 py-2.5 text-left transition-colors duration-150 hover:bg-hover"
                  >
                    <CircleAlert className="mt-0.5 size-4 shrink-0 text-warn" aria-hidden />
                    <span className="min-w-0">
                      <span className="block font-medium text-ink">{e.name}</span>
                      <span className="block text-label text-ink-muted">
                        {[!e.hasDefinition && w.missingDefinition, !e.compiles && w.notCompiling]
                          .filter(Boolean)
                          .join(" · ")}
                      </span>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}
