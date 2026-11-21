import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Panel, PanelGroup, PanelResizeHandle, type ImperativePanelHandle } from "react-resizable-panels";
import { ActivityRail, type Activity } from "./components/ActivityRail";
import { AiPanel } from "./components/AiPanel";
import { ConsolePanel } from "./components/ConsolePanel";
import { EditorArea, OVERVIEW } from "./components/EditorArea";
import { Onboarding } from "./components/Onboarding";
import { ProjectsView } from "./components/ProjectsView";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { TopBar, type PanelKey } from "./components/TopBar";
import { useI18n } from "./i18n";
import { BackendProvider } from "./lib/backend-context";
import {
  getBackend,
  readEnvironment,
  toEngineError,
  type AppSettings,
  type Backend,
  type EngineError,
  type EngineState,
  type WorkspaceSummary,
} from "./lib/engine";

function ResizeHandle({ vertical = false }: { vertical?: boolean }) {
  return (
    <PanelResizeHandle
      className={
        vertical
          ? "relative h-px bg-line transition-colors duration-150 after:absolute after:inset-x-0 after:-top-1 after:-bottom-1 after:content-[''] hover:bg-brand data-[resize-handle-state=drag]:bg-brand"
          : "relative w-px bg-line transition-colors duration-150 after:absolute after:inset-y-0 after:-left-1 after:-right-1 after:content-[''] hover:bg-brand data-[resize-handle-state=drag]:bg-brand"
      }
    />
  );
}

export function App({ backend: injected }: { backend?: Backend }) {
  const backend = useMemo(() => injected ?? getBackend(), [injected]);
  const { m } = useI18n();

  const [engine, setEngine] = useState<EngineState>({ status: "loading" });
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceSummary | null>(null);
  const [openError, setOpenError] = useState<EngineError | null>(null);
  const [busy, setBusy] = useState(false);
  const [tabs, setTabs] = useState<string[]>([]);
  const [activeTab, setActiveTab] = useState<string>(OVERVIEW);

  const [activity, setActivity] = useState<Activity>("elements");
  const [open, setOpen] = useState<Record<PanelKey, boolean>>({ sidebar: true, console: true, ai: true });
  const refs = {
    sidebar: useRef<ImperativePanelHandle>(null),
    console: useRef<ImperativePanelHandle>(null),
    ai: useRef<ImperativePanelHandle>(null),
  };

  useEffect(() => {
    readEnvironment().then(setEngine);
    backend
      .settings()
      .then(setSettings)
      .catch(() => setSettings({ onboardingDone: false, recent: [] }));
  }, [backend]);

  // --- Proyectos -------------------------------------------------------------

  const openPath = useCallback(
    async (path: string) => {
      setBusy(true);
      setOpenError(null);
      try {
        const summary = await backend.openWorkspace(path);
        setWorkspace(summary);
        setTabs([]);
        setActiveTab(OVERVIEW);
        setSettings(await backend.settings());
      } catch (err) {
        setOpenError(toEngineError(err));
      } finally {
        setBusy(false);
      }
    },
    [backend],
  );

  const pickAndOpen = useCallback(async () => {
    setOpenError(null);
    try {
      const path = await backend.pickWorkspace({ title: m.projects.pickerTitle, filter: m.projects.pickerFilter });
      if (path) await openPath(path);
    } catch (err) {
      setOpenError(toEngineError(err));
    }
  }, [backend, m.projects.pickerTitle, m.projects.pickerFilter, openPath]);

  const closeProject = useCallback(async () => {
    await backend.closeWorkspace().catch(() => undefined);
    setWorkspace(null);
    setTabs([]);
    setActiveTab(OVERVIEW);
  }, [backend]);

  const forgetRecent = useCallback(
    async (path: string) => {
      try {
        setSettings(await backend.forgetRecent(path));
      } catch (err) {
        setOpenError(toEngineError(err));
      }
    },
    [backend],
  );

  const finishOnboarding = useCallback(async () => {
    try {
      setSettings(await backend.finishOnboarding());
    } catch {
      // Si no se pudo guardar, igual se entra a la app; la bienvenida volverá a salir la próxima vez.
      setSettings((s) => ({ recent: s?.recent ?? [], onboardingDone: true }));
    }
  }, [backend]);

  // --- Pestañas de elementos ---------------------------------------------------

  const openElement = useCallback((name: string) => {
    setTabs((prev) => (prev.includes(name) ? prev : [...prev, name]));
    setActiveTab(name);
  }, []);

  const closeTab = useCallback((name: string) => {
    setTabs((prev) => {
      const index = prev.indexOf(name);
      const next = prev.filter((t) => t !== name);
      setActiveTab((current) => (current !== name ? current : (next[index - 1] ?? next[index] ?? OVERVIEW)));
      return next;
    });
  }, []);

  // --- Paneles -----------------------------------------------------------------

  const toggle = useCallback((key: PanelKey) => {
    const panel = refs[key].current;
    if (!panel) return;
    if (panel.isCollapsed()) panel.expand();
    else panel.collapse();
    // Los refs son estables entre renders, por eso no hay dependencias.
  }, []);

  // Atajos al estilo de los IDE: Ctrl+B árbol, Ctrl+J consola, Ctrl+L asistente
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.ctrlKey || e.altKey || e.shiftKey || e.metaKey) return;
      const map: Record<string, PanelKey> = { b: "sidebar", j: "console", l: "ai" };
      const key = map[e.key.toLowerCase()];
      if (!key) return;
      e.preventDefault();
      toggle(key);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [toggle]);

  const setPanelOpen = (key: PanelKey, value: boolean) =>
    setOpen((prev) => (prev[key] === value ? prev : { ...prev, [key]: value }));

  const chooseActivity = (next: Activity) => {
    const panel = refs.sidebar.current;
    if (next === activity && panel && !panel.isCollapsed()) {
      panel.collapse();
      return;
    }
    setActivity(next);
    if (panel?.isCollapsed()) panel.expand();
  };

  // --- Vista ---------------------------------------------------------------------

  if (!settings) return <div className="h-full bg-ground" aria-busy="true" />;

  if (!settings.onboardingDone) {
    return (
      <BackendProvider backend={backend}>
        <Onboarding engine={engine} onStart={finishOnboarding} />
      </BackendProvider>
    );
  }

  return (
    <BackendProvider backend={backend}>
      <div className="flex h-full flex-col">
        <TopBar open={open} onToggle={toggle} projectName={workspace?.name ?? null} />

        <div className="flex min-h-0 flex-1">
          <ActivityRail active={open.sidebar ? activity : null} onChange={chooseActivity} />

          <PanelGroup direction="horizontal" autoSaveId="mcgod:layout:h" className="min-w-0 flex-1">
            <Panel
              ref={refs.sidebar}
              id="sidebar"
              order={1}
              defaultSize={19}
              minSize={13}
              maxSize={32}
              collapsible
              collapsedSize={0}
              onCollapse={() => setPanelOpen("sidebar", false)}
              onExpand={() => setPanelOpen("sidebar", true)}
            >
              <Sidebar
                activity={activity}
                workspace={workspace}
                activeElement={activeTab === OVERVIEW ? null : activeTab}
                onOpenElement={openElement}
              />
            </Panel>
            <ResizeHandle />

            <Panel id="center" order={2} defaultSize={56} minSize={34}>
              <PanelGroup direction="vertical" autoSaveId="mcgod:layout:v">
                <Panel id="editor" order={1} defaultSize={82} minSize={35}>
                  {workspace ? (
                    <EditorArea
                      workspace={workspace}
                      tabs={tabs}
                      active={activeTab}
                      onActivate={setActiveTab}
                      onCloseTab={closeTab}
                      onOpenElement={openElement}
                      onCloseProject={closeProject}
                    />
                  ) : (
                    <ProjectsView
                      settings={settings}
                      busy={busy}
                      error={openError}
                      onPick={pickAndOpen}
                      onOpen={openPath}
                      onForget={forgetRecent}
                      onDismissError={() => setOpenError(null)}
                    />
                  )}
                </Panel>
                <ResizeHandle vertical />
                <Panel
                  ref={refs.console}
                  id="console"
                  order={2}
                  defaultSize={18}
                  minSize={10}
                  collapsible
                  collapsedSize={0}
                  onCollapse={() => setPanelOpen("console", false)}
                  onExpand={() => setPanelOpen("console", true)}
                >
                  <ConsolePanel />
                </Panel>
              </PanelGroup>
            </Panel>
            <ResizeHandle />

            <Panel
              ref={refs.ai}
              id="ai"
              order={3}
              defaultSize={25}
              minSize={19}
              maxSize={42}
              collapsible
              collapsedSize={0}
              onCollapse={() => setPanelOpen("ai", false)}
              onExpand={() => setPanelOpen("ai", true)}
            >
              <AiPanel />
            </Panel>
          </PanelGroup>
        </div>

        <StatusBar engine={engine} workspace={workspace} />
      </div>
    </BackendProvider>
  );
}
