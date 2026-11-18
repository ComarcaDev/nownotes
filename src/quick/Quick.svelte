<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api, type Kind, type List } from "../lib/api";
  import { startOfDay } from "../lib/dates";
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
  }

  async function hide() {
    reset();
    await win.hide();
  }

  async function submit() {
    const t = text.trim();
    if (!t || saving) return;
    saving = true;
    const [first, ...rest] = t.split("\n");
    try {
      await api.saveItem({
        kind,
        title: first.trim(),
        body: rest.join("\n").trim(),
        listId,
        dueAt: kind === "task" && remindAt != null ? startOfDay(remindAt).getTime() : null,
        remindAt,
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
    {#key pickerKey}
      <ReminderPicker value={remindAt} onchange={(v) => (remindAt = v)} compact />
    {/key}
    <span class="grow"></span>
    <span class="hint faint">Enter guardar · Tab tarea/nota · Esc cerrar</span>
  </footer>
</div>

<style>
  .quick { display: flex; flex-direction: column; height: 100%; padding: 10px 14px 12px; gap: 8px; }
  header { display: flex; align-items: center; gap: 8px; }
  header select { padding: 3px 8px; font-size: 12.5px; max-width: 170px; }
  .grow { flex: 1; align-self: stretch; }
  textarea {
    flex: 1; font-size: 16px; line-height: 1.45; padding: 10px 12px;
    background: var(--card); border-radius: 6px;
  }
  footer { display: flex; align-items: center; gap: 8px; }
  .hint { font-size: 11px; white-space: nowrap; }
</style>
