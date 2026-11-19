<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api, type Kind, type List } from "../lib/api";
  import { startOfDay, fmt, dayLabel } from "../lib/dates";
  import { parseDate } from "../lib/nlp";
  import ReminderPicker from "../lib/components/ReminderPicker.svelte";
  import Icon from "../lib/components/Icon.svelte";

  const win = getCurrentWindow();
  let text = $state("");
  let kind = $state<Kind>("task");
  let listId = $state(1);
  let lists = $state<List[]>([]);
  let remindAt = $state<number | null>(null);
  let pickerKey = $state(0);
  let saving = $state(false);
  let el = $state<HTMLTextAreaElement>();

  async function prepare() {
    lists = await api.lists();
    if (!lists.some((l) => l.id === listId)) listId = 1;
    pickerKey++; // recalcula "En 1 h", etc.
    await tick();
    el?.focus();
  }

  function reset() {
    text = "";
    remindAt = null;
    kind = "task";
    nlpOff = false;
  }

  // Fechas en lenguaje natural sobre la primera línea («llamar a Ana mañana 9:00»).
  let nlpOff = $state(false);
  const firstLine = $derived(text.split("\n")[0] ?? "");
  const parsed = $derived(nlpOff || !firstLine.trim() ? null : parseDate(firstLine));

  async function hide() {
    reset();
    await win.hide();
  }

  async function submit() {
    const t = text.trim();
    if (!t || saving) return;
    saving = true;
    const [first, ...rest] = t.split("\n");
    const p = parsed;
    const remind = remindAt ?? p?.remindAt ?? null;
    try {
      await api.saveItem({
        kind,
        title: p && p.title ? p.title : first.trim(),
        body: rest.join("\n").trim(),
        listId,
        dueAt: kind !== "task" ? null : p ? p.dueAt : remind != null ? startOfDay(remind).getTime() : null,
        remindAt: remind,
        recur: "none",
        pinned: false,
        tags: [],
      });
      await hide();
    } finally {
      saving = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); hide(); }
    else if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); submit(); }
    else if (e.key === "Tab") { e.preventDefault(); kind = kind === "task" ? "note" : "task"; }
  }

  onMount(() => {
    prepare();
    const un = listen("quick-open", prepare);
    window.addEventListener("focus", () => el?.focus());
    return () => { un.then((f) => f()); };
  });
</script>

<div class="quick">
  <header data-tauri-drag-region>
    <div class="segmented">
      <button class:active={kind === "task"} onclick={() => (kind = "task")}>Tarea</button>
      <button class:active={kind === "note"} onclick={() => (kind = "note")}>Nota</button>
    </div>
    <span class="grow" data-tauri-drag-region></span>
    <select bind:value={listId} title="Lista">
      {#each lists as l (l.id)}<option value={l.id}>{l.name}</option>{/each}
    </select>
    <button class="icon-btn" title="Cerrar (Esc)" onclick={hide}><Icon name="x" /></button>
  </header>

  <textarea
    bind:this={el}
    bind:value={text}
    onkeydown={onKey}
    placeholder={kind === "task" ? "¿Qué tienes que hacer?" : "Escribe tu nota… (Mayús+Enter para nueva línea)"}
  ></textarea>

  <footer>
    {#if parsed}
      <span class="detected" title={`Detectado: «${parsed.matches.join(" ")}»`}>
        <Icon name={parsed.remindAt ? "bell" : "calendar"} size={12} />
        {parsed.remindAt ? fmt(parsed.remindAt) : dayLabel(parsed.dueAt)}
        <button class="undo" title="No usar esta fecha" aria-label="Descartar fecha detectada" onclick={() => { nlpOff = true; el?.focus(); }}>
          <Icon name="x" size={11} />
        </button>
      </span>
    {:else}
      {#key pickerKey}
        <ReminderPicker value={remindAt} onchange={(v) => (remindAt = v)} compact />
      {/key}
    {/if}
    <span class="grow"></span>
    <span class="hint faint">Enter guardar · Tab tarea/nota · Esc cerrar</span>
  </footer>
</div>

<style>
  .quick { display: flex; flex-direction: column; height: 100%; padding: 10px 14px 12px; gap: 8px; }
  header { display: flex; align-items: center; gap: 8px; }
  header select { padding: 3px 8px; font-size: 12.5px; max-width: 170px; }
  .grow { flex: 1; align-self: stretch; }
  .detected {
    display: inline-flex; align-items: center; gap: 5px; padding: 2px 4px 2px 9px; border-radius: 999px;
    font-size: 12px; background: var(--accent); color: var(--accent-text);
  }
  .undo {
    display: grid; place-items: center; width: 18px; height: 18px; border: none; border-radius: 50%;
    background: transparent; color: inherit; cursor: default; padding: 0;
  }
  textarea {
    flex: 1; font-size: 16px; line-height: 1.45; padding: 10px 12px;
    background: var(--card); border-radius: 6px;
  }
  footer { display: flex; align-items: center; gap: 8px; }
  .hint { font-size: 11px; white-space: nowrap; }
</style>
