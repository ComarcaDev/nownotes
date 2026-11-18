<script lang="ts">
  import { onDestroy } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Icon from "./Icon.svelte";
  import ReminderPicker from "./ReminderPicker.svelte";
  import { api, type Item, type Kind, type List, type Recur } from "../api";
  import { fmt, fromInput, toDateInput, RECUR_LABEL } from "../dates";
  import { renderMarkdown } from "../markdown";

  let {
    item,
    lists,
    onclose,
    onremove,
  }: { item: Item; lists: List[]; onclose: () => void; onremove: (it: Item) => void } = $props();

  let titleEl = $state<HTMLTextAreaElement>();
  let menuOpen = $state(false);

  export function focusTitle() {
    titleEl?.focus();
    titleEl?.select();
  }

  type Draft = {
    kind: Kind; title: string; body: string; listId: number; dueAt: number | null;
    remindAt: number | null; recur: Recur; pinned: boolean; tagsText: string;
  };
  const load = (it: Item): Draft => ({
    kind: it.kind, title: it.title, body: it.body, listId: it.listId, dueAt: it.dueAt,
    remindAt: it.remindAt, recur: it.recur, pinned: it.pinned, tagsText: it.tags.join(", "),
  });

  // svelte-ignore state_referenced_locally
  let d = $state<Draft>(load(item));
  // svelte-ignore state_referenced_locally
  let lastSeen = item.updatedAt;
  let version = 0;
  let savedVersion = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  // svelte-ignore state_referenced_locally
  let preview = $state(item.kind === "note" && item.body.trim() !== "");

  // Si el elemento cambia desde fuera (aviso pospuesto, marcado en la lista…) y no hay cambios sin guardar, se recarga.
  $effect(() => {
    const u = item.updatedAt;
    if (u !== lastSeen && version === savedVersion) {
      lastSeen = u;
      d = load(item);
    }
  });

  function changed() {
    version++;
    clearTimeout(timer);
    timer = setTimeout(save, 400);
  }

  async function save() {
    clearTimeout(timer);
    if (version === savedVersion) return;
    const v = version;
    const saved = await api.saveItem({
      id: item.id,
      kind: d.kind,
      title: d.title,
      body: d.body,
      listId: d.listId,
      dueAt: d.dueAt,
      remindAt: d.remindAt,
      recur: d.recur,
      pinned: d.pinned,
      tags: d.tagsText.split(",").map((t) => t.trim().replace(/^#/, "")).filter(Boolean),
    });
    lastSeen = saved.updatedAt;
    savedVersion = v;
  }

  onDestroy(() => { save(); });

  function set<K extends keyof Draft>(key: K, value: Draft[K]) {
    d[key] = value;
    changed();
  }

  async function remove() {
    menuOpen = false;
    await save(); // se guarda lo último para que «Deshacer» lo recupere completo
    onremove({ ...item, ...d, tags: d.tagsText.split(",").map((t) => t.trim()).filter(Boolean) });
  }

  function convert() {
    menuOpen = false;
    if (d.kind === "task") {
      d.kind = "note";
      d.dueAt = null; // las notas no tienen fecha límite
      preview = d.body.trim() !== "";
    } else {
      d.kind = "task";
    }
    changed();
  }

  function clickLinks(e: MouseEvent) {
    const a = (e.target as HTMLElement).closest("a");
    if (a) {
      e.preventDefault();
      openUrl(a.href);
    }
  }

  function autosize(node: HTMLTextAreaElement) {
    const fit = () => { node.style.height = "auto"; node.style.height = node.scrollHeight + "px"; };
    fit();
    node.addEventListener("input", fit);
    return { destroy: () => node.removeEventListener("input", fit) };
  }
</script>

<svelte:window onmousedown={() => (menuOpen = false)} />

<section class="editor">
  <header>
    <span class="kind-badge" class:note={d.kind === "note"}>
      <Icon name={d.kind === "note" ? "note" : "tasks"} size={13} />{d.kind === "note" ? "Nota" : "Tarea"}
    </span>
    <span class="spacer"></span>
    <button class="icon-btn" class:on={d.pinned} title={d.pinned ? "Desfijar" : "Fijar arriba"} onclick={() => set("pinned", !d.pinned)}>
      <Icon name={d.pinned ? "pinned" : "pin"} />
    </button>
    <div class="more">
      <button class="icon-btn" title="Más opciones" onclick={(e) => { e.stopPropagation(); menuOpen = !menuOpen; }}><Icon name="more" /></button>
      {#if menuOpen}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="dropdown" onmousedown={(e) => e.stopPropagation()}>
          <button class="mi" onclick={convert}>
            <Icon name={d.kind === "task" ? "note" : "tasks"} size={14} />Convertir en {d.kind === "task" ? "nota" : "tarea"}
          </button>
          <button class="mi danger" onclick={remove}><Icon name="trash" size={14} />Eliminar</button>
        </div>
      {/if}
    </div>
    <button class="icon-btn" title="Cerrar (Esc)" onclick={onclose}><Icon name="x" /></button>
  </header>

  <div class="body">
    <textarea
      bind:this={titleEl}
      class="title"
      rows="1"
      placeholder="Título"
      value={d.title}
      use:autosize
      oninput={(e) => set("title", e.currentTarget.value.replace(/\n/g, " "))}
      onkeydown={(e) => { if (e.key === "Enter") e.preventDefault(); }}
    ></textarea>

    <div class="field">
      <span class="label">Lista</span>
      <select value={d.listId} onchange={(e) => set("listId", Number(e.currentTarget.value))}>
        {#each lists as l (l.id)}<option value={l.id}>{l.name}</option>{/each}
      </select>
    </div>

    {#if d.kind === "task"}
      <div class="field">
        <span class="label">Fecha</span>
        <div class="row">
          <input type="date" value={toDateInput(d.dueAt)} onchange={(e) => set("dueAt", fromInput(e.currentTarget.value))} />
          {#if d.dueAt != null}
            <button class="icon-btn" title="Quitar fecha" onclick={() => set("dueAt", null)}><Icon name="x" size={13} /></button>
          {/if}
        </div>
      </div>
    {/if}

    <div class="field">
      <span class="label">Recordatorio</span>
      <div>
        <ReminderPicker value={d.remindAt} onchange={(v) => set("remindAt", v)} />
      </div>
    </div>

    {#if d.remindAt != null || d.dueAt != null}
      <div class="field">
        <span class="label">Repetir</span>
        <select value={d.recur} onchange={(e) => set("recur", e.currentTarget.value as Recur)}>
          {#each Object.entries(RECUR_LABEL) as [k, label] (k)}<option value={k}>{label}</option>{/each}
        </select>
      </div>
    {/if}

    <div class="field">
      <span class="label">Etiquetas</span>
      <input type="text" placeholder="trabajo, casa…" value={d.tagsText} oninput={(e) => set("tagsText", e.currentTarget.value)} />
    </div>

    <div class="notes-head">
      <span class="label">{d.kind === "note" ? "Contenido" : "Notas"}</span>
      <button class="icon-btn" title={preview ? "Editar" : "Vista previa"} onclick={() => (preview = !preview)}>
        <Icon name={preview ? "edit" : "eye"} />
      </button>
    </div>
    {#if preview}
      <div class="markdown preview" role="presentation" onclick={clickLinks} ondblclick={() => (preview = false)}>
        {#if d.body.trim()}{@html renderMarkdown(d.body)}{:else}<span class="faint">Sin contenido. Doble clic para escribir.</span>{/if}
      </div>
    {:else}
      <textarea
        class="content"
        placeholder={"Escribe aquí… Admite **negrita**, *cursiva*, listas con - y casillas - [ ]"}
        value={d.body}
        oninput={(e) => set("body", e.currentTarget.value)}
      ></textarea>
    {/if}
  </div>

  <footer class="faint">
    Creado {fmt(item.createdAt)} · Modificado {fmt(item.updatedAt)}
  </footer>
</section>

<style>
  .editor { display: flex; flex-direction: column; min-height: 0; background: var(--layer); border-left: 1px solid var(--border); }
  header { display: flex; align-items: center; gap: 2px; padding: 10px 12px; }
  .spacer { flex: 1; }
  .kind-badge {
    display: inline-flex; align-items: center; gap: 6px; padding: 3px 10px 3px 8px; border-radius: 999px;
    font-size: 12px; font-weight: 600; background: var(--accent-soft); color: var(--accent);
  }
  .more { position: relative; }
  .dropdown {
    position: absolute; right: 0; top: 34px; z-index: 5; width: 210px; padding: 4px;
    display: flex; flex-direction: column; background: var(--solid); border-radius: var(--radius); box-shadow: var(--shadow);
  }
  .mi {
    display: flex; align-items: center; gap: 10px; border: none; background: transparent;
    padding: 7px 10px; border-radius: var(--radius-sm); text-align: left; cursor: default;
  }
  .mi:hover { background: var(--hover); }
  .mi.danger { color: var(--danger); }
  .body { flex: 1; overflow-y: auto; padding: 4px 18px 18px; display: flex; flex-direction: column; gap: 12px; }
  .title {
    font-family: var(--font-display); font-size: 20px; font-weight: 600; border: none; background: transparent;
    padding: 4px 0; overflow: hidden;
  }
  .title:focus { border: none; background: transparent; padding-bottom: 4px; }
  .field { display: grid; grid-template-columns: 96px 1fr; align-items: start; gap: 8px; }
  .field > .label { padding-top: 6px; }
  .label { font-size: 12px; color: var(--text-2); }
  .row { display: flex; align-items: center; gap: 4px; }
  .notes-head { display: flex; align-items: center; justify-content: space-between; margin-top: 4px; }
  .content { flex: 1; min-height: 180px; line-height: 1.5; }
  .preview {
    flex: 1; min-height: 120px; padding: 8px 10px; border-radius: var(--radius-sm);
    background: var(--control); border: 1px solid var(--border);
  }
  footer { font-size: 11.5px; padding: 8px 18px 12px; }
</style>
