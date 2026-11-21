/**
 * Sustituto de react-resizable-panels para jsdom.
 * jsdom no calcula tamaños, así que la librería real no puede plegar paneles en las pruebas.
 * Este sustituto conserva la misma API que usa la app: plegar, desplegar y avisar con onCollapse/onExpand.
 * El comportamiento real se verifica en el navegador.
 */
import { forwardRef, useImperativeHandle, useRef, useState, type ReactNode } from "react";

type PanelProps = {
  id?: string;
  children?: ReactNode;
  onCollapse?: () => void;
  onExpand?: () => void;
  order?: number;
  defaultSize?: number;
  minSize?: number;
  maxSize?: number;
  collapsible?: boolean;
  collapsedSize?: number;
  className?: string;
};

export type ImperativePanelHandle = {
  collapse: () => void;
  expand: () => void;
  isCollapsed: () => boolean;
};

export const Panel = forwardRef<ImperativePanelHandle, PanelProps>(function Panel(
  { id, children, onCollapse, onExpand },
  ref,
) {
  const [collapsed, setCollapsed] = useState(false);
  const collapsedRef = useRef(false);
  useImperativeHandle(ref, () => ({
    collapse() {
      if (collapsedRef.current) return;
      collapsedRef.current = true;
      setCollapsed(true);
      onCollapse?.();
    },
    expand() {
      if (!collapsedRef.current) return;
      collapsedRef.current = false;
      setCollapsed(false);
      onExpand?.();
    },
    isCollapsed: () => collapsedRef.current,
  }));
  return (
    <div data-panel-id={id} data-collapsed={collapsed}>
      {collapsed ? null : children}
    </div>
  );
});

export function PanelGroup({
  children,
}: {
  children?: ReactNode;
  direction?: string;
  autoSaveId?: string;
  className?: string;
}) {
  return <div>{children}</div>;
}

export function PanelResizeHandle(_props: { className?: string }) {
  return <div role="separator" />;
}
