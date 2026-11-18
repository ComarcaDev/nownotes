import type { Item } from "./api";

export type MenuAction = "toggle" | "pin" | "convert" | "remove" | "open" | { move: number };

/** Acciones sobre elementos que la App implementa y pasa a listas, filas y tarjetas. */
export interface ItemActions {
  select: (it: Item) => void;
  toggle: (it: Item) => void;
  remove: (it: Item) => void;
  pin: (it: Item) => void;
  setReminder: (it: Item, at: number | null) => void;
  menu: (e: MouseEvent, it: Item) => void;
}
