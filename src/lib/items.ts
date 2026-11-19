// Utilidades comunes sobre tareas y notas.
import type { Item, ItemInput } from "./api";

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
