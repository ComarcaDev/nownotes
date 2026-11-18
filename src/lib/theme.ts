import { listen } from "@tauri-apps/api/event";
import { api } from "./api";

export function applyTheme(theme: string) {
  const root = document.documentElement;
  if (theme === "dark" || theme === "light") root.dataset.theme = theme;
  else delete root.dataset.theme;
}

/** Aplica el tema guardado y lo mantiene sincronizado entre ventanas. */
export async function initTheme() {
  try {
    const s = await api.settings();
    applyTheme(s.theme);
  } catch {
    /* sin backend (p. ej. vista previa en navegador) */
  }
  await listen<[string, string]>("settings-changed", (e) => {
    const [key, value] = e.payload;
    if (key === "theme") applyTheme(value);
  });
}
