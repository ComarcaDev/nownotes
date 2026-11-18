// Estado de la ventana principal compartido: tema, modo (completo/mínimo) y anclado.
import { listen } from "@tauri-apps/api/event";
import { api, type UiMode } from "./api";
import { applyTheme } from "./theme";
import { showToast } from "./toast.svelte";

export const win = $state({
  theme: "dark",
  mode: "full" as UiMode,
  onTop: false,
  ready: false,
});

export const isDark = () =>
  win.theme === "dark" || (win.theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches);

export async function initWindowState() {
  const s = await api.settings();
  win.theme = s.theme || "dark";
  win.mode = s.ui_mode === "mini" ? "mini" : "full";
  win.onTop = s.always_on_top === "1";
  win.ready = true;
  applyTheme(win.theme);
  await listen<[string, string]>("settings-changed", (e) => {
    const [key, value] = e.payload;
    if (key === "theme") win.theme = value;
    else if (key === "ui_mode") win.mode = value === "mini" ? "mini" : "full";
    else if (key === "always_on_top") win.onTop = value === "1";
  });
}

export async function toggleTheme() {
  const next = isDark() ? "light" : "dark";
  win.theme = next;
  applyTheme(next);
  await api.setSetting("theme", next);
}

export async function toggleOnTop() {
  const next = !win.onTop;
  win.onTop = next;
  try {
    await api.setSetting("always_on_top", next ? "1" : "0");
    showToast(next ? "Ventana anclada encima de las demás" : "Ventana desanclada", undefined, 2500);
  } catch {
    win.onTop = !next;
  }
}

export async function toggleMode() {
  const next: UiMode = win.mode === "mini" ? "full" : "mini";
  win.mode = next;
  await api.setUiMode(next);
}
