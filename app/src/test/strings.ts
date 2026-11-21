/** Recorre un objeto de textos y devuelve cada texto con su ruta (`welcome.title`, `ai.abilities.0`…). */
export function collectStrings(value: unknown, path = ""): { path: string; text: string }[] {
  if (typeof value === "string") return [{ path, text: value }];
  if (typeof value === "function") {
    // Los textos con variables se prueban con valores de ejemplo.
    const args = Array.from({ length: value.length }, (_, i) => (i === 0 ? "21.0.1" : 21));
    return [{ path, text: String(value(...args)) }];
  }
  if (Array.isArray(value)) return value.flatMap((v, i) => collectStrings(v, path ? `${path}.${i}` : String(i)));
  if (value && typeof value === "object")
    return Object.entries(value).flatMap(([k, v]) => collectStrings(v, path ? `${path}.${k}` : k));
  return [];
}

/** Estructura de un objeto de textos sin los textos: sirve para comparar idiomas. */
export function shapeOf(value: unknown): unknown {
  if (typeof value === "function") return `función/${value.length}`;
  if (Array.isArray(value)) return value.map(shapeOf);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([k, v]) => [k, shapeOf(v)]),
    );
  return typeof value;
}
