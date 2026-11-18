<script lang="ts">
  // Barra para crear tareas o notas: clic en «+»/«Añadir» o Enter.
  import { slide } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import ReminderPicker from "./ReminderPicker.svelte";
  import { api, type Filter, type Item, type Kind } from "../api";
  import { at, startOfDay, toDateInput, fromInput, dayLabel } from "../dates";

  let {
    filter,
    oncreated,
    input = $bindable(),
  }: { filter: Filter; oncreated: (it: Item) => void; input?: HTMLInputElement } = $props();

  const fixedKind = $derived<Kind | null>(
    filter.view === "notes" ? "note" : filter.view === "list" || filter.view === "tag" ? null : "task"
  );
  let chosenKind = $state<Kind>("task");
  const kind = $derived<Kind>(fixedKind ?? chosenKind);

  const defaultDue = () => (filter.view === "today" ? startOfDay().getTime() : filter.view === "upcoming" ? at(1, 0) : null);

  let text = $state("");
  let dueAt = $state<number | null>(null);
  let remindAt = $state<number | null>(null);
  let showReminder = $state(false);
  let focused = $state(false);
  let dateEl = $state<HTMLInputElement>();
  let busy = false;

  // Al cambiar de vista se restablecen los valores por defecto.
  $effect(() => {
    void filter.view; void filter.listId; void filter.tag;
    reset();
  });

  function reset() {
    text = "";
    dueAt = defaultDue();
    remindAt = null;
    showReminder = false;
  }

  const expanded = $derived(focused || text.trim() !== "" || remindAt != null);
  const canSubmit = $derived(text.trim() !== "");

  const placeholder = $derived.by(() => {
    if (kind === "note") return "Escribe una nota nueva…";
    if (filter.view === "today") return "Añadir una tarea para hoy…";
    if (filter.view === "upcoming") return "Añadir una tarea para mañana…";
    return "Añadir una tarea…";
  });

  async function create() {
    const t = text.trim();
    if (!t || busy) return;
    busy = true;
    try {
      const it = await api.saveItem({
        kind,
        title: t,
        body: "",
        listId: filter.view === "list" ? (filter.listId ?? 1) : 1,
        dueAt: kind === "task" ? dueAt : null,
        remindAt,
        recur: "none",
        pinned: false,
        tags: filter.view === "tag" && filter.tag ? [filter.tag] : [],
      });
      reset();
      input?.focus();
      oncreated(it);
    } finally {
      busy = false;
    }
  }

  function onFocusOut(e: FocusEvent) {
    const next = e.relatedTarget as Node | null;
    if (!next || !(e.currentTarget as HTMLElement).contains(next)) focused = false;
  }

  const today = () => startOfDay().getTime();
  const tomorrow = () => at(1, 0);
</script>

<div class="composer" class:expanded role="group" onfocusin={() => (focused = true)} onfocusout={onFocusOut}>
  <div class="line">
    <button class="plus" class:note={kind === "note"} title="Crear" onclick={() => (canSubmit ? create() : input?.focus())}>
      <Icon name={kind === "note" ? "note" : "plus"} size={15} />
    </button>
    <input
      type="text"
      bind:this={input}
      bind:value={text}
      {placeholder}
      onkeydown={(e) => {
        if (e.key === "Enter") { e.preventDefault(); create(); }
        if (e.key === "Escape") { reset(); e.currentTarget.blur(); }
      }}
    />
    {#if fixedKind == null}
      <div class="segmented kind">
        <button class:active={chosenKind === "task"} onclick={() => (chosenKind = "task")}>Tarea</button>
        <button class:active={chosenKind === "note"} onclick={() => (chosenKind = "note")}>Nota</button>
      </div>
    {/if}
    <button class="add" class:ready={canSubmit} disabled={!canSubmit} onclick={create} title="Añadir (Enter)">
      Añadir <kbd>↵</kbd>
    </button>
  </div>

  {#if expanded}
    <div class="extras" transition:slide={{ duration: 140 }}>
      {#if kind === "task"}
        <button class="chip" class:active={dueAt === today()} onclick={() => (dueAt = dueAt === today() ? null : today())}>
          <Icon name="sun" size={12} />Hoy
        </button>
        <button class="chip" class:active={dueAt === tomorrow()} onclick={() => (dueAt = dueAt === tomorrow() ? null : tomorrow())}>
          <Icon name="calendar" size={12} />Mañana
        </button>
        <span class="date">
          <button class="chip" class:active={dueAt != null && dueAt !== today() && dueAt !== tomorrow()} onclick={() => dateEl?.showPicker()}>
            <Icon name="calendar" size={12} />
            {dueAt != null && dueAt !== today() && dueAt !== tomorrow() ? dayLabel(dueAt) : "Fecha…"}
          </button>
          <input bind:this={dateEl} type="date" tabindex="-1" value={toDateInput(dueAt)} onchange={(e) => (dueAt = fromInput(e.currentTarget.value))} />
        </span>
        <span class="sep"></span>
      {/if}
      {#if showReminder || remindAt != null}
        <Icon name="bell" size={13} />
        <ReminderPicker value={remindAt} onchange={(v) => { remindAt = v; if (v == null) showReminder = false; }} compact />
      {:else}
        <button class="chip" onclick={() => (showReminder = true)}><Icon name="bell" size={12} />Recordatorio</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .composer {
    margin: 0 24px 14px; background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
    transition: box-shadow 0.15s, border-color 0.15s;
  }
  .composer.expanded { border-color: var(--border-strong); box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08); }
  .line { display: flex; align-items: center; gap: 8px; padding: 6px 6px 6px 8px; }
  .plus {
    flex: none; display: grid; place-items: center; width: 26px; height: 26px; border-radius: 50%;
    border: 1.5px solid var(--accent); background: transparent; color: var(--accent); cursor: default; padding: 0;
    transition: background 0.12s, color 0.12s;
  }
  .plus:hover { background: var(--accent); color: var(--accent-text); }
  .plus.note { border-radius: 6px; }
  input[type="text"] { flex: 1; border: none; background: transparent; padding: 7px 2px; font-size: 14.5px; }
  input[type="text"]:focus { border: none; padding-bottom: 7px; background: transparent; }
  .kind { flex: none; }
  .kind button { padding: 2px 10px; font-size: 12.5px; }
  .add {
    flex: none; display: inline-flex; align-items: center; gap: 6px; padding: 5px 8px 5px 12px;
    border-radius: 6px; border: 1px solid var(--border); background: var(--control); color: var(--text-3);
    cursor: default; transition: background 0.12s, color 0.12s;
  }
  .add.ready { background: var(--accent); color: var(--accent-text); border-color: transparent; }
  .add.ready:hover { background: var(--accent-hover); }
  .add.ready kbd { border-color: color-mix(in srgb, var(--accent-text) 40%, transparent); background: transparent; }
  .extras {
    display: flex; flex-wrap: wrap; align-items: center; gap: 6px; padding: 2px 12px 10px 42px; color: var(--text-2);
  }
  .sep { width: 1px; height: 16px; background: var(--border-strong); margin: 0 4px; }
  .date { position: relative; display: inline-flex; }
  .date input { position: absolute; left: 0; bottom: 0; width: 1px; height: 1px; opacity: 0; pointer-events: none; padding: 0; border: 0; }
</style>
