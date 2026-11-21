import { describe, expect, it } from "vitest";
import { buildChecks } from "../components/Onboarding";
import { javaStatus } from "../components/StatusBar";
import { en } from "../i18n/locales/en";
import { es } from "../i18n/locales/es";
import type { EngineState, JavaInfo } from "../lib/engine";

const ready = (java: JavaInfo | null): EngineState => ({
  status: "ready",
  env: { appVersion: "0.1.0", os: "Windows x86_64", java },
});

const temurin21: JavaInfo = { version: "21.0.13.1", major: 21, vendor: "Temurin", home: null };
const java17: JavaInfo = { version: "17.0.9", major: 17, vendor: "Temurin", home: null };
const java25: JavaInfo = { version: "25.0.1", major: 25, vendor: null, home: null };

describe("requisitos del primer arranque", () => {
  const c = es.onboarding.checks;

  it("mientras carga, ambos requisitos están pendientes", () => {
    const checks = buildChecks({ status: "loading" }, c);
    expect(checks.map((x) => x.state)).toEqual(["pending", "pending"]);
  });

  it("en el navegador avisa que hay que abrir la app de escritorio", () => {
    const [engine, java] = buildChecks({ status: "offline" }, c);
    expect(engine).toMatchObject({ state: "warn", detail: c.engineOffline });
    expect(java).toMatchObject({ state: "warn", detail: c.javaOffline });
  });

  it("si el motor falla, muestra el mensaje de error", () => {
    const checks = buildChecks({ status: "error", message: "sin respuesta" }, c);
    expect(checks).toHaveLength(1);
    expect(checks[0].detail).toContain("sin respuesta");
  });

  it("con Java 21 todo está listo", () => {
    const [engine, java] = buildChecks(ready(temurin21), c);
    expect(engine).toMatchObject({ state: "ok", detail: "Conectado · versión 0.1.0 · Windows x86_64" });
    expect(java).toMatchObject({ state: "ok", detail: "Temurin 21.0.13.1" });
  });

  it("acepta versiones de Java más nuevas que la mínima", () => {
    const [, java] = buildChecks(ready(java25), c);
    expect(java).toMatchObject({ state: "ok", detail: "JDK 25.0.1" });
  });

  it("con Java 17 avisa que NeoForge necesita Java 21", () => {
    const [, java] = buildChecks(ready(java17), c);
    expect(java.state).toBe("warn");
    expect(java.detail).toBe("Tienes Java 17.0.9; los mods de NeoForge necesitan Java 21.");
  });

  it("sin Java explica qué instalar", () => {
    const [, java] = buildChecks(ready(null), c);
    expect(java).toMatchObject({ state: "warn", detail: "No encontrado. Instala un JDK 21 para compilar mods." });
  });

  it("en inglés usa los textos en inglés", () => {
    const [, java] = buildChecks(ready(java17), en.onboarding.checks);
    expect(java.detail).toBe("You have Java 17.0.9; NeoForge mods need Java 21.");
  });
});

describe("Java en la barra de estado", () => {
  const s = es.status;

  it("sin motor no afirma nada", () => {
    expect(javaStatus({ status: "offline" }, s)).toEqual({ text: "Java: —", warn: false });
  });

  it("muestra la versión detectada", () => {
    expect(javaStatus(ready(temurin21), s)).toEqual({ text: "Java 21.0.13.1", warn: false });
  });

  it("marca en amarillo una versión vieja o la falta de Java", () => {
    expect(javaStatus(ready(java17), s).warn).toBe(true);
    expect(javaStatus(ready(null), s)).toEqual({ text: "Java no encontrado", warn: true });
  });
});
