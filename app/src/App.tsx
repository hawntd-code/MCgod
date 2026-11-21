import { useCallback, useEffect, useRef, useState } from "react";
import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
  type ImperativePanelHandle,
} from "react-resizable-panels";
import { ActivityRail, type Activity } from "./components/ActivityRail";
import { AiPanel } from "./components/AiPanel";
import { ConsolePanel } from "./components/ConsolePanel";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { TopBar, type PanelKey } from "./components/TopBar";
import { Welcome } from "./components/Welcome";
import { readEnvironment, type EngineState } from "./lib/engine";

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

export function App() {
  const [engine, setEngine] = useState<EngineState>({ status: "loading" });
  const [activity, setActivity] = useState<Activity>("elements");
  const [open, setOpen] = useState<Record<PanelKey, boolean>>({
    sidebar: true,
    console: true,
    ai: true,
  });

  const refs = {
    sidebar: useRef<ImperativePanelHandle>(null),
    console: useRef<ImperativePanelHandle>(null),
    ai: useRef<ImperativePanelHandle>(null),
  };

  useEffect(() => {
    readEnvironment().then(setEngine);
  }, []);

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

  return (
    <div className="flex h-full flex-col">
      <TopBar open={open} onToggle={toggle} />

      <div className="flex min-h-0 flex-1">
        <ActivityRail
          active={open.sidebar ? activity : null}
          onChange={chooseActivity}
        />

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
            <Sidebar activity={activity} />
          </Panel>
          <ResizeHandle />

          <Panel id="center" order={2} minSize={34}>
            <PanelGroup direction="vertical" autoSaveId="mcgod:layout:v">
              <Panel id="editor" order={1} minSize={35}>
                <Welcome engine={engine} />
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

      <StatusBar engine={engine} />
    </div>
  );
}
