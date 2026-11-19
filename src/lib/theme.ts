import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import { applyAppearance, parseAppearance, PREVIEW_EVENT, type Appearance } from "./appearance";

let theme = "dark";
let appearance: Appearance = parseAppearance(undefined);
/** La imagen de fondo solo se muestra en la ventana principal; las emergentes usan Mica. */
let allowImage = false;

const systemDark = () => matchMedia("(prefers-color-scheme: dark)").matches;
const isDark = () => theme === "dark" || (theme !== "light" && systemDark());

function refresh() {
  const a = !allowImage && appearance.bg === "image" ? { ...appearance, bg: "mica" as const } : appearance;
  applyAppearance(a, isDark());
}

export function applyTheme(next: string) {
  theme = next;
  const root = document.documentElement;
  if (next === "dark" || next === "light") root.dataset.theme = next;
  else delete root.dataset.theme;
  refresh(); // el acento se ajusta según sea claro u oscuro
}

/** Vista previa inmediata (sin guardar), p. ej. mientras se mueve un control deslizante. */
export function previewAppearance(a: Appearance) {
  appearance = a;
  refresh();
  window.dispatchEvent(new CustomEvent(PREVIEW_EVENT, { detail: a }));
}

/** Apariencia actual de esta ventana. */
export const currentAppearance = () => appearance;
/** ¿Se ve en oscuro ahora mismo? */
export const isDarkNow = isDark;

/** Aplica el tema y la apariencia guardados y los mantiene sincronizados entre ventanas. */
export async function initTheme(opts: { backgroundImage?: boolean } = {}) {
  allowImage = !!opts.backgroundImage;
  try {
    const s = await api.settings();
    appearance = parseAppearance(s.appearance);
    applyTheme(s.theme);
  } catch {
    refresh(); /* sin backend (p. ej. vista previa en navegador) */
  }
  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", refresh);
  await listen<[string, string]>("settings-changed", (e) => {
    const [key, value] = e.payload;
    if (key === "theme") applyTheme(value);
    else if (key === "appearance") previewAppearance(parseAppearance(value));
  });
}
