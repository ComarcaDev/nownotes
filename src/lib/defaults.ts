// Valores predeterminados de los ajustes que se pueden restablecer, y «Deshacer».
// Deben coincidir con `DEFAULTS` de src-tauri/src/db.rs (lo comprueba defaults.test.ts).
import { api } from "./api";
import { showToast } from "./toast.svelte";

export const SETTING_DEFAULTS = {
  theme: "dark",
  sound: "chime",
  sound_path: "",
  volume: "0.7",
  native_toast: "1",
  close_to_tray: "1",
  update_check: "1",
  shortcut: "Ctrl+Alt+N",
  shortcut_toggle: "Ctrl+Alt+A",
  keys: "",
  /** No está en Rust: vacío = apariencia predeterminada (ver appearance.ts). */
  appearance: "",
} as const;

export type ResettableKey = keyof typeof SETTING_DEFAULTS;

/** Ajustes que «Restablecer todo» devuelve a su valor original. */
export const ALL_RESETTABLE = Object.keys(SETTING_DEFAULTS) as ResettableKey[];

/** Los atajos globales se liberan antes de reasignarlos, para que no choquen entre sí. */
const GLOBAL_KEYS = ["shortcut", "shortcut_toggle"];

/** ¿El valor guardado difiere del predeterminado? */
export function isModified(key: ResettableKey, value: string | undefined): boolean {
  return (value ?? SETTING_DEFAULTS[key]) !== SETTING_DEFAULTS[key];
}

/** Copia de los valores actuales de esos ajustes. */
export async function snapshot(keys: string[]): Promise<Record<string, string>> {
  const s = await api.settings();
  return Object.fromEntries(keys.map((k) => [k, s[k] ?? (SETTING_DEFAULTS as Record<string, string>)[k] ?? ""]));
}

/** Guarda varios ajustes. Devuelve los errores (p. ej. un atajo que ya usa otra app). */
export async function applySettings(values: Record<string, string>): Promise<string[]> {
  const errors: string[] = [];
  const entries = Object.entries(values);
  const globals = entries.filter(([k]) => GLOBAL_KEYS.includes(k));
  for (const [k] of globals) await api.setSetting(k, "").catch(() => {});
  for (const [k, v] of entries) {
    try {
      await api.setSetting(k, v);
    } catch (e) {
      errors.push(String(e));
    }
  }
  return errors;
}

/**
 * Restablece ajustes y ofrece «Deshacer» en un aviso.
 * `what` completa la frase «Se restableció …» (p. ej. «el color de acento»).
 */
export async function resetWithUndo(what: string, values: Record<string, string>): Promise<void> {
  const before = await snapshot(Object.keys(values));
  const errors = await applySettings(values);
  if (errors.length) {
    showToast(`No se pudo restablecer todo: ${errors.join(" ")}`, undefined, 8000);
    return;
  }
  showToast(
    `Se restableció ${what}`,
    {
      label: "Deshacer",
      run: () => {
        applySettings(before).then((errs) => {
          if (errs.length) showToast(`No se pudo deshacer del todo: ${errs.join(" ")}`, undefined, 8000);
        });
      },
    },
    UNDO_MS
  );
}

/** Tiempo para decidir si deshacer (con el ratón encima, el aviso no se cierra). */
export const UNDO_MS = 12_000;
