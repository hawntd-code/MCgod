import type { CSSProperties } from "react";

/**
 * Retraso de entrada para filas de una lista: cada una llega un poco después de la anterior.
 * El tope evita que una lista larga tarde en completarse; desde la fila `max` entran juntas.
 * Con movimiento reducido, styles.css anula el retraso.
 */
export function stagger(index: number, { step = 28, base = 0, max = 10 } = {}): CSSProperties {
  return { animationDelay: `${base + Math.min(index, max) * step}ms` };
}

/** Retraso fijo para encadenar bloques de una misma pantalla. */
export function delay(ms: number): CSSProperties {
  return { animationDelay: `${ms}ms` };
}
