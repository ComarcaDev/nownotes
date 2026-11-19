<script lang="ts">
  import { ask } from "@tauri-apps/plugin-dialog";
  import Icon from "./Icon.svelte";
  import ItemRow from "./ItemRow.svelte";
  import NoteCard from "./NoteCard.svelte";
  import Composer from "./Composer.svelte";
  import { api, type Filter, type Item, type List } from "../api";
  import type { ItemActions } from "../actions";
  import { dayLabel, startOfDay } from "../dates";
  import { reorderable } from "../reorder";

  let {
    items,
    filter,
    lists,
    selectedId,
    searching,
    shortcut,
    actions,
    oncreated,
    compact = false,
    newInput = $bindable(),
  }: {
    items: Item[];
    filter: Filter;
    lists: List[];
    selectedId: number | null;
    searching: boolean;
    shortcut: string;
    actions: ItemActions;
    oncreated: (it: Item) => void;
    compact?: boolean;
    newInput?: HTMLInputElement;
  } = $props();

  type Section = { key: string; label: string; tone?: "danger"; tasks: Item[]; notes: Item[] };

  const listById = $derived(new Map(lists.map((l) => [l.id, l])));
  const showList = $derived(filter.view !== "list");
  const canAdd = $derived(filter.view !== "done" && !searching);
  // Orden manual solo donde no manda la fecha.
  const canReorder = $derived(!searching && ["tasks", "notes", "list"].includes(filter.view));

  function groupByDay(src: Item[], when: (it: Item) => number): Section[] {
    const out: Section[] = [];
    for (const it of src) {
      const label = dayLabel(when(it));
      const last = out[out.length - 1];
      if (last && last.label === label) last.tasks.push(it);
      else out.push({ key: label, label, tasks: [it], notes: [] });
    }
    return out;
  }

  const sections = $derived.by<Section[]>(() => {
    const tasks = items.filter((i) => i.kind === "task");
    const notes = items.filter((i) => i.kind === "note");
    if (searching) {
      return [
        { key: "t", label: tasks.length && notes.length ? "Tareas" : "", tasks, notes: [] },
        { key: "n", label: tasks.length && notes.length ? "Notas" : "", tasks: [], notes },
      ];
    }
    switch (filter.view) {
      case "today": {
        const sod = startOfDay().getTime();
        const overdue = tasks.filter((t) => startOfDay(t.dueAt ?? t.remindAt ?? 0).getTime() < sod);
        const rest = tasks.filter((t) => !overdue.includes(t));
        const out: Section[] = [];
        if (overdue.length) out.push({ key: "overdue", label: "Vencidas", tone: "danger", tasks: overdue, notes: [] });
        out.push({ key: "today", label: overdue.length || notes.length ? "Tareas" : "", tasks: rest, notes: [] });
        out.push({ key: "notes-today", label: "Notas de hoy", tasks: [], notes });
        return out;
      }
      case "upcoming":
        return groupByDay(tasks, (t) => t.dueAt ?? t.remindAt ?? 0);
      case "done":
        return groupByDay(items, (t) => t.doneAt ?? t.updatedAt);
      case "notes": {
        const pinned = notes.filter((n) => n.pinned);
        const others = notes.filter((n) => !n.pinned);
        return [
          { key: "pinned", label: pinned.length ? "Fijadas" : "", tasks: [], notes: pinned },
          { key: "others", label: pinned.length && others.length ? "Otras" : "", tasks: [], notes: others },
        ];
      }
      case "tasks":
        return [{ key: "tasks", label: "", tasks, notes: [] }];
      default: // list, tag
        return [
          { key: "t", label: notes.length && tasks.length ? "Tareas" : "", tasks, notes: [] },
          { key: "n", label: notes.length && tasks.length ? "Notas" : "", tasks: [], notes },
        ];
    }
  });

  const empty = $derived.by(() => {
    const sc = shortcut || "Ctrl+Alt+N";
    if (searching) return { icon: "search", title: "Sin resultados", hint: "Prueba con otras palabras." };
    switch (filter.view) {
      case "today": return { icon: "sun", title: "Nada pendiente para hoy", hint: `Escribe arriba o pulsa ${sc} desde cualquier programa.` };
      case "upcoming": return { icon: "calendar", title: "Nada programado", hint: "Las tareas con fecha o recordatorio futuros aparecerán aquí." };
      case "tasks": return { icon: "done", title: "Sin tareas pendientes", hint: "Las tareas son cosas por hacer: se marcan al terminarlas." };
      case "notes": return { icon: "note", title: "Aún no hay notas", hint: "Las notas guardan ideas, enlaces o textos largos con formato." };
      case "done": return { icon: "done", title: "Nada completado todavía", hint: "Al completar una tarea aparecerá aquí." };
      default: return { icon: "list", title: "Esta lista está vacía", hint: "Añade tareas o notas desde arriba." };
    }
  });

  async function clearDone() {
    const ok = await ask(`Se borrarán definitivamente ${items.length} elementos completados.`, {
      title: "¿Vaciar completadas?",
      kind: "warning",
      okLabel: "Vaciar",
      cancelLabel: "Cancelar",
    });
    if (ok) await api.clearDone();
  }
</script>

<div class="wrap" class:compact>
  {#if canAdd}
    <Composer {filter} {oncreated} {compact} bind:input={newInput} />
  {/if}

  <div class="list">
    {#if filter.view === "done" && items.length > 0 && !searching}
      <div class="toolbar">
        <button class="btn subtle danger" onclick={clearDone}><Icon name="trash" size={14} />Vaciar completadas</button>
      </div>
    {/if}

    {#each sections as s (s.key)}
      {#if s.tasks.length || s.notes.length}
        {#if s.label}
          <div class="section-title" class:danger={s.tone === "danger"}>
            {s.label}<span class="faint">{s.tasks.length + s.notes.length}</span>
          </div>
        {/if}
        {#if s.tasks.length}
          <div class="rows" data-axis="y" use:reorderable={{ enabled: canReorder, axis: "y", onreorder: actions.reorder }}>
            {#each s.tasks as it (it.id)}
              <ItemRow
                item={it}
                list={listById.get(it.listId)}
                {showList}
                selected={it.id === selectedId}
                {actions}
                draggable={canReorder}
                hideToday={filter.view === "today" && !searching}
              />
            {/each}
          </div>
        {/if}
        {#if s.notes.length && compact}
          <div class="rows" data-axis="y" use:reorderable={{ enabled: canReorder, axis: "y", onreorder: actions.reorder }}>
            {#each s.notes as it (it.id)}
              <ItemRow item={it} list={listById.get(it.listId)} {showList} selected={it.id === selectedId} {actions} draggable={canReorder} />
            {/each}
          </div>
        {:else if s.notes.length}
          <div class="grid" data-axis="grid" use:reorderable={{ enabled: canReorder, axis: "grid", onreorder: actions.reorder }}>
            {#each s.notes as it (it.id)}
              <NoteCard item={it} list={listById.get(it.listId)} {showList} selected={it.id === selectedId} {actions} draggable={canReorder} />
            {/each}
          </div>
        {/if}
      {/if}
    {/each}

    {#if items.length === 0}
      <div class="empty">
        <div class="empty-icon"><Icon name={empty.icon} size={34} /></div>
        <div class="empty-title">{empty.title}</div>
        <div class="faint">{empty.hint}</div>
      </div>
    {/if}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; min-height: 0; flex: 1; }
  .list { flex: 1; overflow-y: auto; padding: 0 24px 24px; }
  .toolbar { display: flex; justify-content: flex-end; margin-bottom: 4px; }
  .rows { display: flex; flex-direction: column; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 10px; margin-bottom: 6px; }
  .empty { display: flex; flex-direction: column; align-items: center; text-align: center; gap: 6px; padding: 56px 24px; }
  .empty-icon {
    display: grid; place-items: center; width: 72px; height: 72px; border-radius: 50%;
    background: var(--accent-soft); color: var(--accent); margin-bottom: 8px;
  }
  .empty-title { font-family: var(--font-display); font-size: 16px; font-weight: 600; }
  .compact .list { padding: 0 12px 12px; }
  .compact .empty { padding: 32px 16px; }
  .compact .empty-icon { width: 56px; height: 56px; }
</style>
