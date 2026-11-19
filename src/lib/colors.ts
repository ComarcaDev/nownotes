// Utilidades de color sin dependencias (para el acento y los tonos personalizados).

export type RGB = [number, number, number];

/** «#0067c0» o «#06c» → [0, 103, 192]; null si no es un color válido. */
export function hexToRgb(hex: string): RGB | null {
  let h = hex.trim().replace(/^#/, "");
  if (/^[0-9a-f]{3}$/i.test(h)) h = [...h].map((c) => c + c).join("");
  if (!/^[0-9a-f]{6}$/i.test(h)) return null;
  const n = parseInt(h, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function rgbToHex([r, g, b]: RGB): string {
  return "#" + [r, g, b].map((v) => Math.round(Math.max(0, Math.min(255, v))).toString(16).padStart(2, "0")).join("");
}

/** Mezcla `a` con `b` (t = 0 → a, t = 1 → b). */
export function mix(a: RGB, b: RGB, t: number): RGB {
  return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
}

/** Luminancia relativa (WCAG 2.x). */
export function luminance([r, g, b]: RGB): number {
  const f = (v: number) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}

export function contrast(a: RGB, b: RGB): number {
  const [l1, l2] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (l1 + 0.05) / (l2 + 0.05);
}

/** Texto sobre un fondo del color dado: el de mayor contraste entre blanco y negro. */
export function textOn(bg: RGB): "#ffffff" | "#000000" {
  return contrast(bg, [255, 255, 255]) >= contrast(bg, [0, 0, 0]) ? "#ffffff" : "#000000";
}

/** Variables CSS del acento derivadas de un solo color. */
export function accentVars(hex: string, dark: boolean): Record<string, string> | null {
  const rgb = hexToRgb(hex);
  if (!rgb) return null;
  // En oscuro, el hover se aclara; en claro, se oscurece (como el acento de Windows).
  const hover = mix(rgb, dark ? [0, 0, 0] : [255, 255, 255], 0.1);
  return {
    "--accent": rgbToHex(rgb),
    "--accent-hover": rgbToHex(hover),
    "--accent-text": textOn(rgb),
    "--accent-soft": `rgba(${rgb.join(", ")}, 0.14)`,
  };
}
