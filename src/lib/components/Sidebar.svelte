<script lang="ts">
  import Icon from "./Icon.svelte";
  import { api, type Counts, type Filter, type List, type TagCount } from "../api";
  import { confirmDanger } from "../confirm";
  import { INBOX_ID } from "../items";
  import { keys, display } from "../keys.svelte";
  import { safe } from "../safe";

  let {
    lists,
    tags,
    counts,
    filter,
    compact,
    shortcut,
    search = $bindable(""),
    searchEl = $bindable(),
    onselect,
    onsettings,
  }: {
    lists: List[];
    tags: TagCount[];
    counts: Counts;
    filter: Filter;
    compact: boolean;
    shortcut: string;
    search?: string;
    searchEl?: HTMLInputElement;
    onselect: (f: Filter) => void;
    onsettings: () => void;
  } = $props();

  const COLORS = ["#0067c0", "#c239b3", "#038387", "#ca5010", "#8764b8", "#498205", "#d13438", "#986f0b"];

  const smart = [
    { view: "today", label: "Hoy", icon: "sun" },
    { view: "upcoming", label: "Próximas", icon: "calendar" },
    { view: "tasks", label: "Tareas", icon: "tasks" },
    { view: "notes", label: "Notas", icon: "note" },
    { view: "done", label: "Completadas", icon: "checkall" },
  ] as const;

  // «Completadas» no muestra contador: no es trabajo pendiente.
  const countFor = (v: (typeof smart)[number]["view"]) => (v === "done" ? 0 : counts[v]);

  let adding = $state(false);
  let newName = $state("");
  let editingId = $state<number | null>(null);
  let editName = $state("");

  async function addList() {
    const name = newName.trim();
    adding = false;
    newName = "";
    if (!name) return;
    const id = await safe(() => api.saveList(null, name, COLORS[lists.length % COLORS.length]), "No se pudo crear la lista");
    if (id != null) onselect({ view: "list", listId: id });
  }

  async function rename(l: List) {
    const name = editName.trim();
    editingId = null;
    if (name && name !== l.name) await safe(() => api.saveList(l.id, name, l.color), "No se pudo renombrar la lista");
  }

  async function cycleColor(l: List) {
    const next = COLORS[(COLORS.indexOf(l.color) + 1) % COLORS.length];
    await safe(() => api.saveList(l.id, l.name, next), "No se pudo cambiar el color");
  }

  async function remove(l: List) {
    const ok = await confirmDanger(
      "¿Eliminar la lista?",
      `Los elementos de «${l.name}» pasarán a la Bandeja de entrada.`,
      "Eliminar"
    );
    if (!ok) return;
    await safe(() => api.deleteList(l.id), "No se pudo eliminar la lista");
    if (filter.view === "list" && filter.listId === l.id) onselect({ view: "today" });
  }

  function focus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<aside class="sidebar" class:compact>
  {#if !compact}
    <div class="search">
      <Icon name="search" size={14} />
      <input type="search" placeholder="Buscar" aria-label="Buscar tareas y notas" bind:value={search} bind:this={searchEl} />
      {#if !search && keys.app.search}<kbd>{display(keys.app.search).replaceAll("+", " ")}</kbd>{/if}
    </div>
  {/if}

  <nav>
    {#each smart as s (s.view)}
      {@const n = countFor(s.view)}
      <button
        class="nav"
        class:active={filter.view === s.view && !search}
        title={compact ? s.label : undefined}
        aria-label={compact ? s.label : undefined}
        onclick={() => onselect({ view: s.view })}
      >
        <Icon name={s.icon} />
        {#if !compact}<span class="label">{s.label}</span>{/if}
        {#if n > 0}<span class="count" class:badge={compact}>{n}</span>{/if}
      </button>
    {/each}
  </nav>

  <div class="section">
    {#if !compact}<span>Listas</span>{/if}
    <button class="icon-btn mini" title="Nueva lista" aria-label="Nueva lista" onclick={() => (adding = true)}><Icon name="plus" size={14} /></button>
  </div>
  <nav class="scroll">
    {#each lists as l (l.id)}
      <div class="nav-row">
        {#if editingId === l.id && !compact}
          <input
            class="inline"
            aria-label="Nombre de la lista"
            bind:value={editName}
            use:focus
            onkeydown={(e) => { if (e.key === "Enter") rename(l); if (e.key === "Escape") editingId = null; }}
            onblur={() => rename(l)}
          />
        {:else}
          <button
            class="nav"
            class:active={filter.view === "list" && filter.listId === l.id && !search}
            title={compact ? l.name : l.id !== INBOX_ID ? "Doble clic para renombrar" : undefined}
            aria-label={compact ? l.name : undefined}
            onclick={() => onselect({ view: "list", listId: l.id })}
            ondblclick={() => { if (l.id !== INBOX_ID && !compact) { editingId = l.id; editName = l.name; } }}
          >
            {#if l.id === INBOX_ID}
              <Icon name="inbox" />
            {:else}
              <span
                class="dot"
                style:background={l.color}
                role="presentation"
                title="Cambiar color"
                onclick={(e) => { if (!compact) { e.stopPropagation(); cycleColor(l); } }}
              ></span>
            {/if}
            {#if !compact}<span class="label">{l.name}</span>{/if}
            {#if l.count > 0 && !compact}<span class="count">{l.count}</span>{/if}
          </button>
          {#if l.id !== INBOX_ID && !compact}
            <button class="icon-btn mini hover-only" title="Eliminar lista" aria-label={`Eliminar la lista ${l.name}`} onclick={() => remove(l)}>
              <Icon name="trash" size={13} />
            </button>
          {/if}
        {/if}
      </div>
    {/each}
    {#if adding}
      <input
        class="inline"
        placeholder="Nombre de la lista"
        aria-label="Nombre de la nueva lista"
        bind:value={newName}
        use:focus
        onkeydown={(e) => { if (e.key === "Enter") addList(); if (e.key === "Escape") { adding = false; newName = ""; } }}
        onblur={addList}
      />
    {/if}

    {#if tags.length && !compact}
      <div class="section"><span>Etiquetas</span></div>
      {#each tags as t (t.name)}
        <button class="nav" class:active={filter.view === "tag" && filter.tag === t.name && !search} onclick={() => onselect({ view: "tag", tag: t.name })}>
          <Icon name="tag" />
          <span class="label">{t.name}</span>
          {#if t.count > 0}<span class="count">{t.count}</span>{/if}
        </button>
      {/each}
    {/if}
  </nav>

  <div class="footer">
    {#if !compact && shortcut}
      <button class="quick" title="Abrir la captura rápida" onclick={() => api.openQuick()}>
        <Icon name="edit" size={14} />
        <span class="label">Captura rápida</span>
        <span class="keys">{#each display(shortcut).split("+") as k, i (i)}<kbd>{k}</kbd>{/each}</span>
      </button>
    {/if}
    <button
      class="nav"
      class:active={filter.view === "trash" && !search}
      title={compact ? "Papelera" : undefined}
      aria-label={compact ? "Papelera" : undefined}
      onclick={() => onselect({ view: "trash" })}
    >
      <Icon name="trash" />{#if !compact}<span class="label">Papelera</span>{/if}
      {#if counts.trash > 0}<span class="count" class:badge={compact}>{counts.trash}</span>{/if}
    </button>
    <button class="nav" title={compact ? "Ajustes" : undefined} aria-label={compact ? "Ajustes" : undefined} onclick={onsettings}>
      <Icon name="settings" />{#if !compact}<span class="label">Ajustes</span>{/if}
    </button>
  </div>
</aside>

<style>
  .sidebar { display: flex; flex-direction: column; padding: 10px 8px; gap: 2px; min-height: 0; min-width: 0; }
  .search {
    display: flex; align-items: center; gap: 6px; padding: 0 8px; margin: 2px 2px 12px;
    background: var(--control); border: 1px solid var(--border); border-bottom-color: var(--border-strong);
    border-radius: var(--radius-sm); color: var(--text-2);
  }
  .search:focus-within { border-bottom: 2px solid var(--accent); }
  .search input { border: none; background: transparent; flex: 1; padding: 6px 2px; }
  .search input:focus { border: none; padding-bottom: 6px; background: transparent; }
  .search kbd { font-size: 9.5px; opacity: 0.7; }
  nav { display: flex; flex-direction: column; gap: 2px; }
  .scroll { flex: 1; overflow-y: auto; overflow-x: hidden; min-height: 0; }
  .nav {
    position: relative; display: flex; align-items: center; gap: 12px; width: 100%;
    border: none; background: transparent; padding: 8px 10px; border-radius: 5px;
    text-align: left; color: var(--text); cursor: default;
  }
  .nav:hover { background: var(--hover); }
  .nav.active { background: var(--selected); }
  .nav.active::before {
    content: ""; position: absolute; left: 0; top: 50%; translate: 0 -50%;
    width: 3px; height: 16px; border-radius: 3px; background: var(--accent);
  }
  .label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .count { font-size: 12px; color: var(--text-2); }
  .count.badge {
    position: absolute; top: 2px; right: 4px; min-width: 15px; height: 15px; padding: 0 3px; border-radius: 8px;
    font-size: 9.5px; line-height: 15px; text-align: center; background: var(--accent); color: var(--accent-text);
  }
  .dot { width: 10px; height: 10px; border-radius: 50%; margin: 0 3px; flex: none; }
  .section {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 6px 4px 10px; font-size: 12px; font-weight: 600; color: var(--text-2);
  }
  .mini { width: 24px; height: 24px; }
  .nav-row { display: flex; align-items: center; }
  .nav-row .hover-only { visibility: hidden; flex: none; }
  .nav-row:hover .hover-only { visibility: visible; }
  .inline { margin: 2px 4px; }
  .footer { display: flex; flex-direction: column; gap: 2px; padding-top: 6px; border-top: 1px solid var(--border); margin-top: 4px; }
  .quick {
    display: flex; align-items: center; gap: 10px; border: none; background: transparent; color: var(--text-2);
    padding: 7px 10px; border-radius: 5px; font-size: 12.5px; text-align: left; cursor: default;
  }
  .quick:hover { background: var(--hover); color: var(--text); }
  .keys { display: inline-flex; gap: 2px; }
  .keys kbd { font-size: 9.5px; height: 16px; min-width: 16px; }

  .compact { padding: 10px 6px; align-items: stretch; }
  .compact .nav { justify-content: center; padding: 9px 0; gap: 0; }
  .compact .section { justify-content: center; padding: 12px 0 4px; }
</style>
