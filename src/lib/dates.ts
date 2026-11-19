import type { Recur } from "./api";

export const HOUR_MS = 3_600_000;
export const DAY_MS = 24 * HOUR_MS;

/** 00:00 (hora local) del día de `d`. */
export function startOfDay(d: Date | number = new Date()): Date {
  const x = new Date(d);
  x.setHours(0, 0, 0, 0);
  return x;
}

/** Misma hora, `n` días después (respeta los cambios de horario). */
export function addDays(d: Date, n: number): Date {
  const x = new Date(d);
  x.setDate(x.getDate() + n);
  return x;
}

/** Instante de hoy + `days` a la hora indicada. */
export function at(days: number, h: number, m = 0): number {
  const x = addDays(startOfDay(), days);
  x.setHours(h, m, 0, 0);
  return x.getTime();
}

export interface Preset { label: string; value: number }

export function reminderPresets(): Preset[] {
  const now = new Date();
  const in1h = new Date(now.getTime() + HOUR_MS);
  in1h.setSeconds(0, 0);
  in1h.setMinutes(Math.ceil(in1h.getMinutes() / 5) * 5);
  const out: Preset[] = [{ label: "En 1 h", value: in1h.getTime() }];
  const h = now.getHours();
  if (h < 17) out.push({ label: "Hoy 18:00", value: at(0, 18) });
  else if (h < 20) out.push({ label: "Hoy 21:00", value: at(0, 21) });
  out.push({ label: "Mañana 9:00", value: at(1, 9) });
  const dow = now.getDay(); // 0 = domingo
  const toMonday = ((8 - dow) % 7) || 7;
  out.push({ label: "Lunes 9:00", value: at(toMonday, 9) });
  return out;
}

const pad = (n: number) => String(n).padStart(2, "0");

export function toDateTimeInput(ms: number | null): string {
  if (ms == null) return "";
  const d = new Date(ms);
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function toDateInput(ms: number | null): string {
  return toDateTimeInput(ms).slice(0, 10);
}

export function fromInput(v: string): number | null {
  if (!v) return null;
  // "YYYY-MM-DD" se interpretaría como UTC: se fuerza hora local.
  const t = new Date(v.length === 10 ? `${v}T00:00` : v).getTime();
  return isNaN(t) ? null : t;
}

const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

export function dayLabel(ms: number): string {
  const d = new Date(ms);
  const today = startOfDay();
  const diff = Math.round((startOfDay(d).getTime() - today.getTime()) / DAY_MS);
  if (diff === 0) return "Hoy";
  if (diff === 1) return "Mañana";
  if (diff === -1) return "Ayer";
  if (diff > 1 && diff < 7) return cap(d.toLocaleDateString("es-ES", { weekday: "long" }));
  return d.toLocaleDateString("es-ES", {
    weekday: "short",
    day: "numeric",
    month: "short",
    year: d.getFullYear() !== today.getFullYear() ? "numeric" : undefined,
  });
}

export function timeLabel(ms: number): string {
  return new Date(ms).toLocaleTimeString("es-ES", { hour: "2-digit", minute: "2-digit" });
}

export function fmt(ms: number, withTime = true): string {
  return withTime ? `${dayLabel(ms)} ${timeLabel(ms)}` : dayLabel(ms);
}

/** ¿Ya pasó este instante? */
export function isPast(ms: number | null): boolean {
  return ms != null && ms < Date.now();
}

/** ¿Es de un día anterior a hoy? */
export function isBeforeToday(ms: number | null): boolean {
  return ms != null && ms < startOfDay().getTime();
}

export const RECUR_LABEL: Record<Recur, string> = {
  none: "No se repite",
  daily: "Cada día",
  weekdays: "Días laborables",
  weekly: "Cada semana",
  monthly: "Cada mes",
  yearly: "Cada año",
};
