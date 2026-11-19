// Atajos de teclado configurables: globales (registrados en Windows) y de la app.
import { listen } from "@tauri-apps/api/event";
import { api } from "./api";

export type AppAction = "new" | "search" | "mode" | "pin" | "theme";
export type GlobalAction = "quick" | "toggle";

export const APP_ACTIONS: { id: AppAction; label: string; def: string }[] = [
  { id: "new", label: "Escribir una tarea o nota nueva", def: "Ctrl+N" },
  { id: "search", label: "Buscar", def: "Ctrl+F" },
  { id: "mode", label: "Modo mínimo / completo", def: "Ctrl+M" },
  { id: "pin", label: "Anclar la ventana encima", def: "Ctrl+P" },
  { id: "theme", label: "Cambiar tema claro / oscuro", def: "Ctrl+Shift+L" },
];

export const GLOBAL_ACTIONS: { id: GlobalAction; setting: string; label: string; def: string }[] = [
  { id: "quick", setting: "shortcut", label: "Captura rápida", def: "Ctrl+Alt+N" },
  { id: "toggle", setting: "shortcut_toggle", label: "Mostrar u ocultar NowNotes", def: "Ctrl+Alt+A" },
];

/** Combinaciones que Windows o casi todas las apps ya usan. */
const RESERVED = new Set([
  "Ctrl+C", "Ctrl+V", "Ctrl+X", "Ctrl+Z", "Ctrl+Y", "Ctrl+A", "Ctrl+S",
  "Alt+F4", "Alt+Tab", "Ctrl+Alt+Delete", "Ctrl+Shift+Escape",
]);

const appDefaults = () => Object.fromEntries(APP_ACTIONS.map((a) => [a.id, a.def])) as Record<AppAction, string>;

export const keys = $state({
  app: appDefaults(),
  global: { quick: "Ctrl+Alt+N", toggle: "Ctrl+Alt+A" } as Record<GlobalAction, string>,
});

function parseAppKeys(json: string | undefined): Record<AppAction, string> {
  const out = appDefaults();
  try {
    const saved = JSON.parse(json || "{}") as Partial<Record<AppAction, string>>;
    for (const a of APP_ACTIONS) if (typeof saved[a.id] === "string") out[a.id] = saved[a.id]!;
  } catch {
    /* ajuste dañado: se usan los predeterminados */
  }
  return out;
}

let started = false;
export async function initKeys() {
  if (started) return;
  started = true;
  const s = await api.settings();
  keys.app = parseAppKeys(s.keys);
  keys.global = { quick: s.shortcut ?? "", toggle: s.shortcut_toggle ?? "" };
  await listen<[string, string]>("settings-changed", (e) => {
    const [k, v] = e.payload;
    if (k === "keys") keys.app = parseAppKeys(v);
    else if (k === "shortcut") keys.global.quick = v;
    else if (k === "shortcut_toggle") keys.global.toggle = v;
  });
}

/** Combinación normalizada a partir de una pulsación ("Ctrl+Shift+L"), o null si solo hay modificadores. */
export function comboFromEvent(e: KeyboardEvent): string | null {
  if (["Control", "Alt", "Shift", "Meta", "AltGraph"].includes(e.key)) return null;
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");
  let k = e.code;
  if (k.startsWith("Key")) k = k.slice(3);
  else if (k.startsWith("Digit")) k = k.slice(5);
  else if (k.startsWith("Numpad")) k = "Num" + k.slice(6);
  // Teclados en pantalla, escritorio remoto o pulsaciones simuladas pueden no traer `code`.
  if (!k || k === "Unidentified") {
    const key = e.key;
    if (!key || key === "Unidentified" || key === "Dead") return null;
    k = key.length === 1 ? key.toUpperCase() : key === " " ? "Space" : key;
  }
  parts.push(k);
  return parts.join("+");
}

const norm = (c: string) => c.trim().toLowerCase();

/** ¿La pulsación corresponde al atajo de esta acción? */
export function matches(e: KeyboardEvent, action: AppAction): boolean {
  const c = comboFromEvent(e);
  return !!c && !!keys.app[action] && norm(c) === norm(keys.app[action]);
}

/** Texto para mostrar ("Ctrl+Mayús+L"). */
export function display(combo: string): string {
  return combo ? combo.replace("Shift", "Mayús").replace("Super", "Win") : "Sin atajo";
}

function owner(combo: string, except: string): string | null {
  const c = norm(combo);
  for (const a of APP_ACTIONS) if (a.id !== except && norm(keys.app[a.id]) === c) return a.label;
  for (const g of GLOBAL_ACTIONS) if (g.id !== except && norm(keys.global[g.id]) === c) return g.label;
  return null;
}

/** Comprueba una combinación: error si no vale, aviso si es del sistema. */
export function validate(combo: string, action: string, isGlobal: boolean): { error?: string; warning?: string } {
  if (!combo) return {};
  const mods = combo.split("+").slice(0, -1);
  if (mods.length === 0) return { error: "Usa al menos una tecla modificadora (Ctrl, Alt, Mayús o Win)." };
  if (isGlobal && mods.length === 1 && mods[0] === "Shift") return { error: "Un atajo global necesita Ctrl, Alt o Win." };
  const used = owner(combo, action);
  if (used) return { error: `${display(combo)} ya se usa para «${used}».` };
  if (RESERVED.has(combo) || mods.includes("Super")) {
    return { warning: `${display(combo)} suele usarlo Windows u otras aplicaciones; puede no funcionar bien.` };
  }
  return {};
}

export async function setAppKey(action: AppAction, combo: string): Promise<void> {
  const next = { ...keys.app, [action]: combo };
  await api.setSetting("keys", JSON.stringify(next));
  keys.app = next;
}

export async function setGlobalKey(action: GlobalAction, combo: string): Promise<void> {
  const g = GLOBAL_ACTIONS.find((x) => x.id === action)!;
  await api.setSetting(g.setting, combo); // Rust comprueba si Windows u otra app ya lo usa
  keys.global[action] = combo;
}

export async function resetAllKeys(): Promise<string[]> {
  const errors: string[] = [];
  await api.setSetting("keys", JSON.stringify(appDefaults()));
  keys.app = appDefaults();
  // Primero se liberan los globales para que no choquen entre sí al reasignarlos.
  for (const g of GLOBAL_ACTIONS) await setGlobalKey(g.id, "").catch(() => {});
  for (const g of GLOBAL_ACTIONS) {
    try {
      await setGlobalKey(g.id, g.def);
    } catch (e) {
      errors.push(String(e));
    }
  }
  return errors;
}
