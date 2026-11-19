// Apariencia personalizada: acento, tono de superficies y fondo (Mica, color o imagen).
import { accentVars, hexToRgb } from "./colors";

export type Tone = "neutral" | "cool" | "warm" | "accent";
export type Background = "mica" | "solid" | "image";

export interface Appearance {
  /** Color de acento; vacío = el del tema. */
  accent: string;
  tone: Tone;
  bg: Background;
  /** Color del fondo sólido; vacío = el del tema. */
  bgColor: string;
  /** Punto de enfoque de la imagen (background-position). */
  imagePos: string;
  /** Velo sobre la imagen, 0–85 %: oscurece en tema oscuro y aclara en claro. */
  veil: number;
  /** Desenfoque de la imagen, 0–20 px. */
  blur: number;
  /** Opacidad de los paneles sobre la imagen, 30–100 %. */
  panels: number;
  /** Cambia al subir otra imagen, para que se recargue. */
  imageRev: number;
}

/** Evento de ventana para la vista previa en vivo (antes de guardar). */
export const PREVIEW_EVENT = "nownotes:appearance-preview";

export const DEFAULT_APPEARANCE: Appearance = {
  accent: "",
  tone: "neutral",
  bg: "mica",
  bgColor: "",
  imagePos: "center",
  veil: 40,
  blur: 0,
  panels: 70,
  imageRev: 0,
};

export const ACCENTS = [
  "#0067c0", "#4cc2ff", "#038387", "#00a383", "#498205", "#c19c00",
  "#ca5010", "#d13438", "#e3008c", "#8764b8",
];

export const TONES: { id: Tone; label: string }[] = [
  { id: "neutral", label: "Neutro" },
  { id: "cool", label: "Frío" },
  { id: "warm", label: "Cálido" },
  { id: "accent", label: "Del acento" },
];

export const IMAGE_POSITIONS: { id: string; label: string }[] = [
  { id: "left top", label: "Arriba a la izquierda" }, { id: "center top", label: "Arriba" }, { id: "right top", label: "Arriba a la derecha" },
  { id: "left center", label: "Izquierda" }, { id: "center", label: "Centro" }, { id: "right center", label: "Derecha" },
  { id: "left bottom", label: "Abajo a la izquierda" }, { id: "center bottom", label: "Abajo" }, { id: "right bottom", label: "Abajo a la derecha" },
];

/** Recomendación para la imagen de fondo. */
export const IMAGE_HINT =
  "Recomendado: 1920 × 1080 o mayor (16:9), JPG o WebP de menos de 5 MB. En modo mínimo se muestra la parte central.";
export const IMAGE_MIN_WIDTH = 1280;

const clamp = (n: unknown, min: number, max: number, def: number) =>
  typeof n === "number" && Number.isFinite(n) ? Math.max(min, Math.min(max, n)) : def;
const oneOf = <T extends string>(v: unknown, ok: readonly T[], def: T): T => (ok.includes(v as T) ? (v as T) : def);
const color = (v: unknown) => (typeof v === "string" && hexToRgb(v) ? v : "");

/** Lee el ajuste guardado; cualquier valor dañado vuelve a su valor por defecto. */
export function parseAppearance(json: string | undefined): Appearance {
  let raw: Record<string, unknown> = {};
  try {
    const v = JSON.parse(json || "{}");
    if (v && typeof v === "object") raw = v;
  } catch {
    /* ajuste dañado */
  }
  const d = DEFAULT_APPEARANCE;
  return {
    accent: color(raw.accent),
    tone: oneOf(raw.tone, ["neutral", "cool", "warm", "accent"], d.tone),
    bg: oneOf(raw.bg, ["mica", "solid", "image"], d.bg),
    bgColor: color(raw.bgColor),
    imagePos: oneOf(raw.imagePos, IMAGE_POSITIONS.map((p) => p.id), d.imagePos),
    veil: clamp(raw.veil, 0, 85, d.veil),
    blur: clamp(raw.blur, 0, 20, d.blur),
    panels: clamp(raw.panels, 30, 100, d.panels),
    imageRev: clamp(raw.imageRev, 0, Number.MAX_SAFE_INTEGER, 0),
  };
}

const TONE_COLORS: Record<Exclude<Tone, "neutral" | "accent">, string> = { cool: "#3b6ef5", warm: "#f0883e" };

/** Variables CSS que dependen de la apariencia (las demás vienen del tema claro/oscuro). */
export function appearanceVars(a: Appearance, dark: boolean): Record<string, string> {
  const vars: Record<string, string> = {};
  const accent = a.accent ? accentVars(a.accent, dark) : null;
  if (accent) Object.assign(vars, accent);
  const tint = a.tone === "neutral" ? "" : a.tone === "accent" ? a.accent || (dark ? "#4cc2ff" : "#0067c0") : TONE_COLORS[a.tone];
  vars["--tint"] = tint || "transparent";
  vars["--tint-amt"] = tint ? (dark ? "9%" : "6%") : "0%";
  vars["--panels"] = `${a.panels}%`;
  vars["--veil"] = String(a.veil / 100);
  vars["--bg-blur"] = `${a.blur}px`;
  vars["--bg-pos"] = a.imagePos;
  if (a.bgColor) vars["--bg-color"] = a.bgColor;
  return vars;
}

const MANAGED = ["--accent", "--accent-hover", "--accent-text", "--accent-soft", "--bg-color"];

/** Aplica la apariencia al documento (cualquier ventana). */
export function applyAppearance(a: Appearance, dark: boolean, root = document.documentElement) {
  for (const k of MANAGED) root.style.removeProperty(k);
  for (const [k, v] of Object.entries(appearanceVars(a, dark))) root.style.setProperty(k, v);
  root.dataset.bg = a.bg;
}
