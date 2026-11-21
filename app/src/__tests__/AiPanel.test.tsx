import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { AiPanel } from "../components/AiPanel";
import { I18nProvider, type Lang } from "../i18n";

function renderPanel(lang: Lang = "es") {
  return render(
    <I18nProvider lang={lang}>
      <AiPanel />
    </I18nProvider>,
  );
}

describe("panel de IA", () => {
  it("ofrece Anthropic, OpenAI, Gemini, DeepSeek, Ollama y la opción compatible con OpenAI", () => {
    renderPanel();
    const names = screen.getAllByRole("radio").map((r) => r.textContent);
    expect(names).toEqual([
      expect.stringContaining("Anthropic"),
      expect.stringContaining("OpenAI"),
      expect.stringContaining("Google Gemini"),
      expect.stringContaining("DeepSeek"),
      expect.stringContaining("Ollama"),
      expect.stringContaining("Compatible con OpenAI"),
    ]);
  });

  it("al elegir DeepSeek queda marcado y explica cuándo llega la conexión", async () => {
    const user = userEvent.setup();
    renderPanel();
    expect(screen.queryByRole("status")).not.toBeInTheDocument();

    const deepseek = screen.getByRole("radio", { name: /DeepSeek/ });
    await user.click(deepseek);

    expect(deepseek).toHaveAttribute("aria-checked", "true");
    expect(screen.getAllByRole("radio").filter((r) => r.getAttribute("aria-checked") === "true")).toHaveLength(1);
    expect(screen.getByRole("status")).toHaveTextContent(/fase 2/);
  });

  it("cambiar de proveedor desmarca el anterior", async () => {
    const user = userEvent.setup();
    renderPanel();
    await user.click(screen.getByRole("radio", { name: /Anthropic/ }));
    await user.click(screen.getByRole("radio", { name: /Ollama/ }));
    expect(screen.getByRole("radio", { name: /Anthropic/ })).toHaveAttribute("aria-checked", "false");
    expect(screen.getByRole("radio", { name: /Ollama/ })).toHaveAttribute("aria-checked", "true");
  });

  it("el cuadro de mensaje y el botón de enviar están desactivados sin proveedor conectado", () => {
    renderPanel();
    expect(screen.getByLabelText("Mensaje para la IA")).toBeDisabled();
    expect(screen.getByRole("button", { name: "Enviar" })).toBeDisabled();
  });

  it("en inglés traduce la opción genérica y las descripciones, no las marcas", () => {
    renderPanel("en");
    expect(screen.getByRole("radio", { name: /OpenAI-compatible/ })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: /DeepSeek models · API key/ })).toBeInTheDocument();
  });
});
