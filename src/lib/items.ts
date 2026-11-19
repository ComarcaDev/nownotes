// Utilidades comunes sobre tareas y notas.
import type { Counts, Item, ItemInput } from "./api";

/** La «Bandeja de entrada» es la lista 1 (no se puede borrar). */
export const INBOX_ID = 1;

export const UNTITLED = "Sin título";

/** Título para mostrar; `max` recorta los títulos largos (avisos, confirmaciones). */
export function displayTitle(it: Pick<Item, "title">, max?: number): string {
  const t = it.title || UNTITLED;
  return max && t.length > max ? `${t.slice(0, max)}…` : t;
}

/** Título entre comillas angulares para mensajes: «Comprar pan». */
export const quoted = (it: Pick<Item, "title">, max = 40) => `«${displayTitle(it, max)}»`;

/** «trabajo, #casa ,  ,ideas» → ["trabajo", "casa", "ideas"] */
export function parseTags(text: string): string[] {
  return text
    .split(",")
    .map((t) => t.trim().replace(/^#+/, "").trim())
    .filter(Boolean);
}

/** Datos para guardar un elemento existente, con los cambios indicados. */
export function toInput(it: Item, patch: Partial<ItemInput> = {}): ItemInput {
  return {
    id: it.id,
    kind: it.kind,
    title: it.title,
    body: it.body,
    listId: it.listId,
    dueAt: it.dueAt,
    remindAt: it.remindAt,
    recur: it.recur,
    pinned: it.pinned,
    tags: it.tags,
    ...patch,
  };
}

const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;

/** Desglose del contador de «Hoy»: «1 tarea · 2 notas». Vacío si no hay nada. */
export function todayBreakdown(c: Pick<Counts, "today" | "todayTasks">): string {
  const notes = c.today - c.todayTasks;
  return [
    c.todayTasks > 0 ? plural(c.todayTasks, "tarea", "tareas") : "",
    notes > 0 ? plural(notes, "nota", "notas") : "",
  ].filter(Boolean).join(" · ");
}
