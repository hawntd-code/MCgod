/** Marca de MCgod: un bloque de cobre de 8×8 píxeles con vetas oxidadas. */
export function BrandMark({ className }: { className?: string }) {
  const light = "oklch(0.78 0.12 55)";
  const base = "oklch(0.7 0.135 50)";
  const deep = "oklch(0.5 0.115 42)";
  // Cada letra es un píxel: L claro, B base, D oscuro
  const rows = ["LLLBBBBD", "LBBBBDBD", "LBDBBBBD", "BBBBLBBD", "BBDBBBDB", "BBBBBBBD", "BDBBDBBD", "DDDDDDDD"];
  const fill = { L: light, B: base, D: deep } as const;

  return (
    <svg viewBox="0 0 8 8" className={className} shapeRendering="crispEdges" aria-hidden>
      {rows.flatMap((row, y) =>
        [...row].map((c, x) => (
          <rect key={`${x}-${y}`} x={x} y={y} width={1} height={1} fill={fill[c as keyof typeof fill]} />
        )),
      )}
    </svg>
  );
}
