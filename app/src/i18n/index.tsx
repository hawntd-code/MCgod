import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { en } from "./locales/en";
import { es, type Messages } from "./locales/es";

export type Lang = "es" | "en";

export const LANGS: Lang[] = ["es", "en"];
export const messages: Record<Lang, Messages> = { es, en };

const STORAGE_KEY = "mcgod:lang";

function readStored(): Lang | null {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    return value === "es" || value === "en" ? value : null;
  } catch {
    return null;
  }
}

function writeStored(lang: Lang) {
  try {
    localStorage.setItem(STORAGE_KEY, lang);
  } catch {
    // Sin almacenamiento (modo privado o bloqueado): el idioma vale solo para esta sesión.
  }
}

/** Idioma inicial: el elegido antes, o el del sistema si es español; inglés en otro caso. */
export function initialLang(): Lang {
  const stored = readStored();
  if (stored) return stored;
  const system = typeof navigator !== "undefined" ? navigator.language : "";
  return system.toLowerCase().startsWith("es") ? "es" : "en";
}

type I18n = { lang: Lang; m: Messages; setLang: (lang: Lang) => void };

const I18nContext = createContext<I18n | null>(null);

export function I18nProvider({ children, lang: forced }: { children: ReactNode; lang?: Lang }) {
  const [lang, setLangState] = useState<Lang>(() => forced ?? initialLang());

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  const setLang = useCallback((next: Lang) => {
    setLangState(next);
    writeStored(next);
  }, []);

  const value = useMemo(() => ({ lang, m: messages[lang], setLang }), [lang, setLang]);
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18n {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n debe usarse dentro de <I18nProvider>");
  return ctx;
}
