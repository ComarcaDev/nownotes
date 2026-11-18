import { invoke } from "@tauri-apps/api/core";

export type Kind = "task" | "note";
export type Recur = "none" | "daily" | "weekdays" | "weekly" | "monthly" | "yearly";
export type View = "today" | "upcoming" | "tasks" | "notes" | "done" | "list" | "tag" | "trash";
export type UiMode = "full" | "mini";

export interface Item {
  id: number;
  kind: Kind;
  title: string;
  body: string;
  listId: number;
  done: boolean;
  doneAt: number | null;
  dueAt: number | null;
  remindAt: number | null;
  recur: Recur;
  snoozedUntil: number | null;
  pinned: boolean;
  createdAt: number;
  updatedAt: number;
  deletedAt: number | null;
  tags: string[];
}

export interface ItemInput {
  id?: number | null;
  kind: Kind;
  title: string;
  body: string;
  listId: number | null;
  dueAt: number | null;
  remindAt: number | null;
  recur: Recur;
  pinned: boolean;
  tags: string[];
}

export interface List { id: number; name: string; color: string; sort: number; count: number }
export interface TagCount { name: string; count: number }
export interface Counts { today: number; upcoming: number; tasks: number; notes: number; done: number; trash: number }
export interface Filter { view: View; listId?: number | null; tag?: string | null; search?: string | null }
export type Settings = Record<string, string>;
export interface Info { dataDir: string; portable: boolean; version: string }

export const api = {
  info: () => invoke<Info>("get_info"),
  queryItems: (filter: Filter) => invoke<Item[]>("query_items", { filter }),
  getItem: (id: number) => invoke<Item | null>("get_item", { id }),
  saveItem: (input: ItemInput) => invoke<Item>("save_item", { input }),
  deleteItem: (id: number) => invoke<void>("delete_item", { id }),
  toggleDone: (id: number) => invoke<Item>("toggle_done", { id }),
  lists: () => invoke<List[]>("list_lists"),
  saveList: (id: number | null, name: string, color: string) => invoke<number>("save_list", { id, name, color }),
  deleteList: (id: number) => invoke<void>("delete_list", { id }),
  tags: () => invoke<TagCount[]>("list_tags"),
  counts: () => invoke<Counts>("counts"),
  pendingAlerts: () => invoke<Item[]>("pending_alerts"),
  snoozeAlert: (id: number, until: number) => invoke<void>("snooze_alert", { id, until }),
  resolveAlert: (id: number, complete: boolean) => invoke<void>("resolve_alert", { id, complete }),
  settings: () => invoke<Settings>("get_settings"),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
  readFileBytes: (path: string) => invoke<ArrayBuffer>("read_file_bytes", { path }),
  exportData: (path: string) => invoke<void>("export_data", { path }),
  importData: (path: string) => invoke<number>("import_data", { path }),
  showMain: () => invoke<void>("show_main"),
  openQuick: () => invoke<void>("open_quick"),
  startupMinimized: () => invoke<boolean>("startup_minimized"),
  clearDone: () => invoke<number>("clear_done"),
  restoreItem: (id: number) => invoke<void>("restore_item", { id }),
  deleteForever: (id: number) => invoke<void>("delete_forever", { id }),
  emptyTrash: () => invoke<number>("empty_trash"),
  setUiMode: (mode: UiMode) => invoke<void>("set_ui_mode", { mode }),
};
