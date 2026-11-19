/// <reference types="vite/client" />
import rs from "../../src-tauri/src/db.rs?raw";
import { beforeEach, describe, expect, it, vi } from "vitest";

// Backend simulado: guarda los ajustes en memoria y registra el orden de escritura.
const store: Record<string, string> = {};
const writes: [string, string][] = [];
vi.mock("./api", () => ({
  api: {
    settings: async () => ({ ...store }),
    setSetting: async (k: string, v: string) => {
      if (v === "Ctrl+Alt+Z") throw new Error("Ese atajo ya lo usa otra aplicación");
      writes.push([k, v]);
      store[k] = v;
    },
  },
}));
const toasts: { text: string; action?: { label: string; run: () => void } }[] = [];
vi.mock("./toast.svelte", () => ({
  showToast: (text: string, action?: { label: string; run: () => void }) => toasts.push({ text, action }),
}));

const { ALL_RESETTABLE, applySettings, isModified, resetWithUndo, SETTING_DEFAULTS, snapshot } = await import("./defaults");

beforeEach(() => {
  for (const k of Object.keys(store)) delete store[k];
  writes.length = 0;
  toasts.length = 0;
});

describe("valores predeterminados", () => {
  it("coinciden con los de Rust (src-tauri/src/db.rs)", () => {
    const block = rs.slice(rs.indexOf("const DEFAULTS"), rs.indexOf("];", rs.indexOf("const DEFAULTS")));
    const rust = Object.fromEntries([...block.matchAll(/\("([^"]+)",\s*"([^"]*)"\)/g)].map((m) => [m[1], m[2]]));
    for (const [k, v] of Object.entries(SETTING_DEFAULTS)) {
      if (k === "appearance") continue; // solo existe en la interfaz
      expect(rust[k], `ajuste «${k}»`).toBe(v);
    }
  });
  it("detecta si un ajuste está cambiado", () => {
    expect(isModified("theme", "dark")).toBe(false);
    expect(isModified("theme", undefined)).toBe(false);
    expect(isModified("theme", "light")).toBe(true);
    expect(isModified("appearance", "")).toBe(false);
  });
  it("«Restablecer todo» incluye apariencia, sonido, generales y atajos", () => {
    for (const k of ["appearance", "theme", "sound", "volume", "close_to_tray", "keys", "shortcut", "shortcut_toggle"]) {
      expect(ALL_RESETTABLE).toContain(k);
    }
  });
});

describe("restablecer y deshacer", () => {
  it("libera los atajos globales antes de reasignarlos", async () => {
    await applySettings({ shortcut: "Ctrl+Alt+N", shortcut_toggle: "Ctrl+Alt+A", theme: "dark" });
    expect(writes.slice(0, 2)).toEqual([["shortcut", ""], ["shortcut_toggle", ""]]);
    expect(store).toMatchObject({ shortcut: "Ctrl+Alt+N", shortcut_toggle: "Ctrl+Alt+A", theme: "dark" });
  });
  it("devuelve los errores sin detener el resto", async () => {
    const errors = await applySettings({ shortcut: "Ctrl+Alt+Z", theme: "light" });
    expect(errors).toHaveLength(1);
    expect(store.theme).toBe("light");
  });
  it("«Deshacer» recupera los valores anteriores", async () => {
    Object.assign(store, { theme: "light", appearance: '{"accent":"#d13438"}' });
    await resetWithUndo("la apariencia", { theme: "dark", appearance: "" });
    expect(store).toMatchObject({ theme: "dark", appearance: "" });
    expect(toasts[0].text).toBe("Se restableció la apariencia");
    toasts[0].action!.run();
    await vi.waitFor(() => expect(store).toMatchObject({ theme: "light", appearance: '{"accent":"#d13438"}' }));
  });
  it("la copia usa el predeterminado si el ajuste nunca se guardó", async () => {
    expect(await snapshot(["volume", "appearance"])).toEqual({ volume: "0.7", appearance: "" });
  });
});
