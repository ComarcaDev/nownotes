<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api, type Kind, type List } from "../lib/api";
  import { startOfDay } from "../lib/dates";
  import { parseDate } from "../lib/nlp";
  import { INBOX_ID } from "../lib/items";
  import ReminderPicker from "../lib/components/ReminderPicker.svelte";
  import DetectedDate from "../lib/components/DetectedDate.svelte";
  import Icon from "../lib/components/Icon.svelte";

  const appWindow = getCurrentWindow();
  let text = $state("");
  let kind = $state<Kind>("task");
  let listId = $state(INBOX_ID);
  let lists = $state<List[]>([]);
  let remindAt = $state<number | null>(null);
  let pickerKey = $state(0);
  let saving = $state(false);
  let error = $state("");
  let el = $state<HTMLTextAreaElement>();

  // Fechas en lenguaje natural sobre la primera línea («llamar a Ana mañana 9:00»).
  let nlpOff = $state(false);
  const firstLine = $derived(text.split("\n")[0] ?? "");
  const parsed = $derived(nlpOff || !firstLine.trim() ? null : parseDate(firstLine));

  async function prepare() {
    try {
      lists = await api.lists();
    } catch {
      lists = [];
    }
    if (!lists.some((l) => l.id === listId)) listId = INBOX_ID;
    pickerKey++; // recalcula «En 1 h», etc.
    await tick();
    el?.focus();
  }

  function reset() {
    text = "";
    remindAt = null;
    kind = "task";
    nlpOff = false;
    error = "";
  }

  async function hide() {
    reset();
    await appWindow.hide();
  }

  async function submit() {
    const t = text.trim();
    if (!t || saving) return;
    saving = true;
    error = "";
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
    } catch (e) {
      error = `No se pudo guardar: ${String(e)}`; // se conserva el texto para reintentar
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
    const focusInput = () => el?.focus();
    window.addEventListener("focus", focusInput);
    return () => {
      un.then((f) => f());
      window.removeEventListener("focus", focusInput);
    };
  });
</script>

<div class="quick">
  <header data-tauri-drag-region>
    <div class="segmented" role="radiogroup" aria-label="Tipo">
      <button role="radio" aria-checked={kind === "task"} class:active={kind === "task"} onclick={() => (kind = "task")}>Tarea</button>
      <button role="radio" aria-checked={kind === "note"} class:active={kind === "note"} onclick={() => (kind = "note")}>Nota</button>
    </div>
    <span class="drag" data-tauri-drag-region></span>
    <select bind:value={listId} title="Lista" aria-label="Lista">
      {#each lists as l (l.id)}<option value={l.id}>{l.name}</option>{/each}
    </select>
    <button class="icon-btn" title="Cerrar (Esc)" aria-label="Cerrar" onclick={hide}><Icon name="x" /></button>
  </header>

  <textarea
    bind:this={el}
    bind:value={text}
    onkeydown={onKey}
    aria-label={kind === "task" ? "Nueva tarea" : "Nueva nota"}
    placeholder={kind === "task" ? "¿Qué tienes que hacer?" : "Escribe tu nota… (Mayús+Enter para nueva línea)"}
  ></textarea>

  <footer>
    {#if error}
      <span class="error" role="alert">{error}</span>
    {:else if parsed}
      <DetectedDate {parsed} ondismiss={() => { nlpOff = true; el?.focus(); }} />
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
  .drag { flex: 1; align-self: stretch; }
  textarea {
    flex: 1; font-size: 16px; line-height: 1.45; padding: 10px 12px;
    background: var(--card); border-radius: 6px;
  }
  footer { display: flex; align-items: center; gap: 8px; }
  .hint { font-size: 11px; white-space: nowrap; }
  .error { font-size: 12px; color: var(--danger); }
</style>
