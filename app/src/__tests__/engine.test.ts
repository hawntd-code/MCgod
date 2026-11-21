import { invoke, isTauri } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { readEnvironment, type Environment } from "../lib/engine";

vi.mock("@tauri-apps/api/core", () => ({ isTauri: vi.fn(), invoke: vi.fn() }));

const env: Environment = {
  appVersion: "0.1.0",
  os: "Windows x86_64",
  java: { version: "21.0.13.1", major: 21, vendor: "Temurin", home: null },
};

describe("readEnvironment", () => {
  beforeEach(() => {
    vi.mocked(isTauri).mockReset();
    vi.mocked(invoke).mockReset();
  });

  it("queda sin conexión cuando la interfaz corre fuera de la app de escritorio", async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    expect(await readEnvironment()).toEqual({ status: "offline" });
    expect(invoke).not.toHaveBeenCalled();
  });

  it("pide el entorno al motor con el comando `environment`", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue(env);
    expect(await readEnvironment()).toEqual({ status: "ready", env });
    expect(invoke).toHaveBeenCalledWith("environment");
  });

  it("devuelve el error si el motor falla", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockRejectedValue("comando desconocido");
    expect(await readEnvironment()).toEqual({ status: "error", message: "comando desconocido" });
  });
});
