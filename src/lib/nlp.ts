// Fechas en lenguaje natural (español e inglés), sin dependencias.
// «comprar pan mañana 18:00» → título «comprar pan», fecha mañana, aviso a las 18:00.

export interface Parsed {
  /** Texto sin la expresión de fecha. */
  title: string;
  /** Inicio (00:00) del día detectado. */
  dueAt: number;
  /** Fecha y hora del aviso, si se indicó una hora. */
  remindAt: number | null;
  /** Fragmentos del texto original que se han interpretado. */
  matches: string[];
}

const WEEKDAYS: Record<string, number> = {
  domingo: 0, lunes: 1, martes: 2, miercoles: 3, jueves: 4, viernes: 5, sabado: 6,
  sunday: 0, monday: 1, tuesday: 2, wednesday: 3, thursday: 4, friday: 5, saturday: 6,
};
const MONTHS: Record<string, number> = {
  ene: 0, feb: 1, mar: 2, abr: 3, may: 4, jun: 5, jul: 6, ago: 7, sep: 8, set: 8, oct: 9, nov: 10, dic: 11,
  jan: 0, apr: 3, aug: 7, dec: 11,
};
const UNIT_MS: Record<string, number> = { min: 60_000, h: 3_600_000, d: 86_400_000, w: 604_800_000 };

/** Minúsculas y sin tildes, manteniendo la misma longitud que el original (para cortar el título). */
function fold(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/(\p{L})\p{M}+/gu, "$1"); // quita tildes conservando la ñ como «n»
}

function startOfDay(d: Date): Date {
  const x = new Date(d);
  x.setHours(0, 0, 0, 0);
  return x;
}

function addDays(d: Date, n: number): Date {
  const x = new Date(d);
  x.setDate(x.getDate() + n);
  return x;
}

type Span = { start: number; end: number };

export function parseDate(text: string, now: Date = new Date()): Parsed | null {
  const src = fold(text);
  if (src.length !== text.length) return null; // por seguridad: no deberían diferir
  const spans: Span[] = [];
  const take = (re: RegExp): RegExpExecArray | null => {
    re.lastIndex = 0;
    const m = re.exec(src);
    if (!m) return null;
    const start = m.index + (m[0].length - m[0].trimStart().length);
    spans.push({ start, end: m.index + m[0].length });
    return m;
  };

  let day: Date | null = null;
  let hours: number | null = null;
  let minutes = 0;
  let exact: Date | null = null; // «en 2 horas»

  // --- plazos relativos: «en 2 horas», «in 30 minutes»
  const rel = take(/(?:^|\s)(?:en|in)\s+(\d+|un|una|an|a)\s+(minutos?|minutes?|mins?|horas?|hours?|h|dias?|days?|semanas?|weeks?)(?=\s|$|[.,;!?])/);
  if (rel) {
    const n = /^\d+$/.test(rel[1]) ? parseInt(rel[1], 10) : 1;
    const u = rel[2];
    const unit = u.startsWith("min") ? "min" : u.startsWith("h") ? "h" : u.startsWith("d") ? "d" : "w";
    exact = new Date(now.getTime() + n * UNIT_MS[unit]);
    if (unit === "d" || unit === "w") { day = startOfDay(exact); exact = null; }
  }

  // --- momentos del día (antes que «mañana» como día: «por la mañana» es una hora)
  const tod = take(/(?:^|\s)(por la manana|de la manana|in the morning|a mediodia|al mediodia|at noon|esta tarde|this afternoon|por la tarde|in the afternoon|esta noche|tonight|por la noche)(?=\s|$|[.,;!?])/);
  if (tod) {
    const t = tod[1];
    if (t.includes("manana") || t.includes("morning")) hours = 9;
    else if (t.includes("dia") || t.includes("noon")) hours = 12;
    else if (t.includes("tarde") || t.includes("afternoon")) hours = 18;
    else hours = 21;
  }

  // --- días relativos
  if (!day) {
    const d = take(/(?:^|\s)(pasado manana|day after tomorrow|manana|tomorrow|hoy|today)(?=\s|$|[.,;!?])/);
    if (d) {
      const w = d[1];
      const offset = w.startsWith("pasado") || w.startsWith("day after") ? 2 : w === "manana" || w === "tomorrow" ? 1 : 0;
      day = addDays(startOfDay(now), offset);
    }
  }

  // --- días de la semana: «el lunes», «next friday», «on monday»
  if (!day) {
    const w = take(/(?:^|\s)(?:(?:el|este|el proximo|next|on|this)\s+)?(lunes|martes|miercoles|jueves|viernes|sabado|domingo|monday|tuesday|wednesday|thursday|friday|saturday|sunday)(?=\s|$|[.,;!?])/);
    if (w) {
      const target = WEEKDAYS[w[1]];
      let diff = (target - now.getDay() + 7) % 7;
      if (diff === 0) diff = 7; // «el lunes» dicho un lunes = el de la semana que viene
      day = addDays(startOfDay(now), diff);
    }
  }

  // --- fechas: «25/12», «25-12-2027», «25 de diciembre», «dec 25»
  if (!day) {
    const num = take(/(?:^|\s)(?:el\s+|on\s+)?(\d{1,2})[/-](\d{1,2})(?:[/-](\d{2,4}))?(?=\s|$|[.,;!?])/);
    const named = num ? null : take(/(?:^|\s)(?:el\s+|on\s+)?(\d{1,2})\s+(?:de\s+)?([a-z]{3})[a-z]*\.?(?=\s|$|[.,;!?])/);
    const namedEn = num || named ? null : take(/(?:^|\s)(?:on\s+)?([a-z]{3})[a-z]*\.?\s+(\d{1,2})(?:st|nd|rd|th)?(?=\s|$|[.,;!?])/);
    let d: number | null = null;
    let m: number | null = null;
    let y: number | null = null;
    if (num) { d = +num[1]; m = +num[2] - 1; y = num[3] ? +num[3] : null; }
    else if (named && named[2] in MONTHS) { d = +named[1]; m = MONTHS[named[2]]; }
    else if (namedEn && namedEn[1] in MONTHS) { d = +namedEn[2]; m = MONTHS[namedEn[1]]; }
    else if (named || namedEn) spans.pop(); // no era un mes: se descarta
    if (d !== null && m !== null && d >= 1 && d <= 31 && m >= 0 && m <= 11) {
      if (y !== null && y < 100) y += 2000;
      let candidate = new Date(y ?? now.getFullYear(), m, d);
      if (y === null && candidate < startOfDay(now)) candidate = new Date(now.getFullYear() + 1, m, d);
      if (candidate.getDate() === d) day = candidate; // descarta 31/02, etc.
    } else if (num) spans.pop();
  }

  // --- horas: «a las 9», «at 3pm», «18:30», «9am»
  if (hours === null && !exact) {
    const t =
      take(/(?:^|\s)(?:a\s+las?|at|@)\s*(\d{1,2})(?::(\d{2}))?\s*(am|pm|h)?(?=\s|$|[.,;!?])/) ??
      take(/(?:^|\s)(\d{1,2}):(\d{2})\s*(am|pm|h)?(?=\s|$|[.,;!?])/) ??
      take(/(?:^|\s)(\d{1,2})\s*(am|pm)(?=\s|$|[.,;!?])/);
    if (t) {
      let h = parseInt(t[1], 10);
      const hasMinutes = t[2] !== undefined && /^\d+$/.test(t[2]);
      const suffix = hasMinutes ? t[3] : t[2] && !/^\d+$/.test(t[2]) ? t[2] : t[3];
      const min = hasMinutes ? parseInt(t[2], 10) : 0;
      if (suffix === "pm" && h < 12) h += 12;
      if (suffix === "am" && h === 12) h = 0;
      // «a las 3» sin am/pm: lo normal es por la tarde
      if (!suffix && !hasMinutes && h >= 1 && h <= 7) h += 12;
      if (h <= 23 && min <= 59) { hours = h; minutes = min; }
      else spans.pop();
    }
  }

  if (!day && hours === null && !exact) return null;

  let remindAt: number | null = null;
  let dueDay: Date;
  if (exact) {
    remindAt = exact.getTime();
    dueDay = startOfDay(exact);
  } else if (hours !== null) {
    const base = day ?? startOfDay(now);
    const at = new Date(base);
    at.setHours(hours, minutes, 0, 0);
    // Solo una hora y ya pasó hoy → mañana.
    if (!day && at.getTime() <= now.getTime()) at.setDate(at.getDate() + 1);
    remindAt = at.getTime();
    dueDay = startOfDay(at);
  } else {
    dueDay = day!;
  }

  // Título: se quitan los fragmentos detectados y las preposiciones que queden colgando.
  const ordered = [...spans].sort((a, b) => b.start - a.start);
  let title = text;
  for (const s of ordered) title = title.slice(0, s.start) + " " + title.slice(s.end);
  title = title
    .replace(/\s+/g, " ")
    .replace(/\s+(el|la|para|a|en|de|on|at|by|for|this|next)\s*$/i, "")
    .replace(/^\s*(el|para|on|at)\s+/i, "")
    .replace(/\s+([.,;!?])/g, "$1")
    .trim();

  return {
    title,
    dueAt: dueDay.getTime(),
    remindAt,
    matches: spans.sort((a, b) => a.start - b.start).map((s) => text.slice(s.start, s.end)),
  };
}
