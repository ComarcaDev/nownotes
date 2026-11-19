<script lang="ts">
  // Barra para crear tareas o notas: clic en «+»/«Añadir» o Enter.
  import { onMount, untrack } from "svelte";
  import { slide } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import ReminderPicker from "./ReminderPicker.svelte";
  import DetectedDate from "./DetectedDate.svelte";
  import { api, type Filter, type Item, type Kind } from "../api";
  import { at, startOfDay, toDateInput, fromInput, dayLabel } from "../dates";
  import { parseDate } from "../nlp";
  import { INBOX_ID } from "../items";
  import { safe } from "../safe";

  let {
    filter,
    oncreated,
    compact = false,
    input = $bindable(),
  }: { filter: Filter; oncreated: (it: Item) => void; compact?: boolean; input?: HTMLInputElement } = $props();

  // Tipo fijo según la vista; en Hoy, listas y etiquetas se puede elegir.
  const fixedKind = $derived<Kind | null>(
    filter.view === "notes" ? "note" : ["today", "list", "tag"].includes(filter.view) ? null : "task"
  );
  let chosenKind = $state<Kind>("task");
  // Preferencia recordada para «Hoy» (tarea por defecto).
  let todayKind: Kind = "task";
  const kind = $derived<Kind>(fixedKind ?? chosenKind);

  onMount(async () => {
    try {
      const s = await api.settings();
      todayKind = s.today_kind === "note" ? "note" : "task";
      if (filter.view === "today") chosenKind = todayKind;
    } catch {
      /* sin backend */
    }
  });

  function setKind(k: Kind) {
    chosenKind = k;
    if (filter.view === "today" && k !== todayKind) {
      todayKind = k;
      api.setSetting("today_kind", k).catch(() => {});
    }
    input?.focus();
  }

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

  // Al cambiar de vista: el tipo vuelve al preferido (Hoy) o a tarea.
  $effect(() => {
    const v = filter.view;
    untrack(() => (chosenKind = v === "today" ? todayKind : "task"));
  });

  function reset() {
    text = "";
    dueAt = defaultDue();
    remindAt = null;
    showReminder = false;
    nlpOff = false;
  }

  // Fechas en lenguaje natural: «comprar pan mañana 18:00».
  let nlpOff = $state(false);
  const parsed = $derived(nlpOff || !text.trim() ? null : parseDate(text));

  const expanded = $derived(focused || text.trim() !== "" || remindAt != null);
  const canSubmit = $derived(text.trim() !== "");

  const placeholder = $derived.by(() => {
    if (compact) return kind === "note" ? "Nueva nota…" : "Nueva tarea…";
    if (kind === "note") return "Escribe una nota nueva…";
    if (filter.view === "today") return "Añadir una tarea para hoy…";
    if (filter.view === "upcoming") return "Añadir una tarea para mañana…";
    return "Añadir una tarea…";
  });

  async function create() {
    const t = text.trim();
    if (!t || busy) return;
    busy = true;
    const p = parsed;
    try {
      const it = await safe(
        () =>
          api.saveItem({
            kind,
            title: p && p.title ? p.title : t,
            body: "",
            listId: filter.view === "list" ? (filter.listId ?? INBOX_ID) : INBOX_ID,
            dueAt: kind === "task" ? (p ? p.dueAt : dueAt) : null,
            remindAt: remindAt ?? p?.remindAt ?? null,
            recur: "none",
            pinned: false,
            tags: filter.view === "tag" && filter.tag ? [filter.tag] : [],
          }),
        kind === "note" ? "No se pudo crear la nota" : "No se pudo crear la tarea"
      );
      if (!it) return; // se conserva el texto para reintentar
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

<div class="composer" class:expanded class:compact role="group" onfocusin={() => (focused = true)} onfocusout={onFocusOut}>
  <div class="line">
    <button
      class="plus"
      class:note={kind === "note"}
      title="Crear"
      aria-label={kind === "note" ? "Crear nota" : "Crear tarea"}
      onclick={() => (canSubmit ? create() : input?.focus())}
    >
      <Icon name={kind === "note" ? "note" : "plus"} size={15} />
    </button>
    <input
      type="text"
      bind:this={input}
      bind:value={text}
      {placeholder}
      aria-label={kind === "note" ? "Nueva nota" : "Nueva tarea"}
      onkeydown={(e) => {
        if (e.key === "Enter") { e.preventDefault(); create(); }
        if (e.key === "Escape") { reset(); e.currentTarget.blur(); }
        if (e.key === "Tab" && !e.shiftKey && fixedKind == null) {
          e.preventDefault();
          setKind(chosenKind === "task" ? "note" : "task");
        }
      }}
    />
    {#if fixedKind == null}
      <div class="segmented kind" class:icons={compact} role="radiogroup" aria-label="Tipo">
        <button
          role="radio"
          aria-checked={chosenKind === "task"}
          class:active={chosenKind === "task"}
          title="Crear tarea  (Tab para cambiar)"
          aria-label="Tarea"
          onclick={() => setKind("task")}
        >
          {#if compact}<Icon name="tasks" size={14} />{:else}Tarea{/if}
        </button>
        <button
          role="radio"
          aria-checked={chosenKind === "note"}
          class:active={chosenKind === "note"}
          title="Crear nota  (Tab para cambiar)"
          aria-label="Nota"
          onclick={() => setKind("note")}
        >
          {#if compact}<Icon name="note" size={14} />{:else}Nota{/if}
        </button>
      </div>
    {/if}
    <button class="add" class:ready={canSubmit} disabled={!canSubmit} onclick={create} title="Añadir (Enter)">
      {#if !compact}Añadir {/if}<kbd>↵</kbd>
    </button>
  </div>

  {#if expanded}
    <div class="extras" transition:slide={{ duration: 140 }}>
      {#if parsed}
        <DetectedDate {parsed} ondismiss={() => { nlpOff = true; input?.focus(); }} />
        <span class="sep"></span>
      {/if}
      {#if kind === "task" && !parsed}
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
          <input bind:this={dateEl} type="date" tabindex="-1" aria-hidden="true" value={toDateInput(dueAt)} onchange={(e) => (dueAt = fromInput(e.currentTarget.value))} />
        </span>
        <span class="sep"></span>
      {/if}
      {#if parsed}
        <span class="faint hint">Fecha detectada en el texto</span>
      {:else if showReminder || remindAt != null}
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
  .compact { margin: 0 12px 10px; }
  .compact .add { padding: 5px 6px; }
  .compact .extras { padding-left: 12px; }
  .kind.icons button { display: grid; place-items: center; padding: 3px 7px; }
  .sep { width: 1px; height: 16px; background: var(--border-strong); margin: 0 4px; }
  .hint { font-size: 11.5px; }
  .date { position: relative; display: inline-flex; }
  .date input { position: absolute; left: 0; bottom: 0; width: 1px; height: 1px; opacity: 0; pointer-events: none; padding: 0; border: 0; }
</style>
