<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api, type Counts, type Filter, type Item, type ItemInput, type List, type TagCount } from "../lib/api";
  import type { ItemActions, MenuAction } from "../lib/actions";
  import { showToast } from "../lib/toast.svelte";
  import { fmt } from "../lib/dates";
  import Sidebar from "../lib/components/Sidebar.svelte";
  import ItemList from "../lib/components/ItemList.svelte";
  import Editor from "../lib/components/Editor.svelte";
  import SettingsDialog from "../lib/components/Settings.svelte";
  import ContextMenu from "../lib/components/ContextMenu.svelte";
  import Toast from "../lib/components/Toast.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import WindowToolbar from "../lib/components/WindowToolbar.svelte";
  import TrashList from "../lib/components/TrashList.svelte";
  import { win, initWindowState, toggleTheme, toggleMode, toggleOnTop } from "../lib/window.svelte";

  let filter = $state<Filter>({ view: "today" });
  let searchText = $state("");
  let search = $state("");
  let items = $state<Item[]>([]);
  let lists = $state<List[]>([]);
  let tags = $state<TagCount[]>([]);
  let counts = $state<Counts>({ today: 0, upcoming: 0, tasks: 0, notes: 0, done: 0, trash: 0 });
  let selectedId = $state<number | null>(null);
  let selectedItem = $state<Item | null>(null);
  let showSettings = $state(false);
  let shortcut = $state("");
  let menu = $state<{ x: number; y: number; item: Item } | null>(null);
  let width = $state(1040);
  let searchEl = $state<HTMLInputElement>();
  let newEl = $state<HTMLInputElement>();
  let editor = $state<ReturnType<typeof Editor>>();

  const mini = $derived(win.mode === "mini");
  const compact = $derived(selectedItem ? width < 1100 : width < 860);
  const searching = $derived(search.trim() !== "");

  // Pestañas del modo mínimo.
  const MINI_TABS = [
    { view: "today", label: "Hoy", icon: "sun" },
    { view: "tasks", label: "Tareas", icon: "tasks" },
    { view: "notes", label: "Notas", icon: "note" },
  ] as const;

  // Al entrar en modo mínimo, solo se admiten las vistas de sus pestañas.
  $effect(() => {
    if (mini && !MINI_TABS.some((t) => t.view === filter.view)) {
      filter = { view: "today" };
      searchText = "";
    }
  });

  const TITLES: Record<string, string> = {
    today: "Hoy", upcoming: "Próximas", tasks: "Tareas", notes: "Notas", done: "Completadas", trash: "Papelera",
  };
  const title = $derived.by(() => {
    if (searching) return `Resultados para «${search.trim()}»`;
    if (filter.view === "list") return lists.find((l) => l.id === filter.listId)?.name ?? "Lista";
    if (filter.view === "tag") return `#${filter.tag}`;
    return TITLES[filter.view];
  });
  const subtitle = $derived.by(() => {
    if (searching) return "";
    if (filter.view === "today") return new Date().toLocaleDateString("es-ES", { weekday: "long", day: "numeric", month: "long" });
    if (filter.view === "tasks") return "Cosas por hacer";
    if (filter.view === "notes") return "Ideas y textos para guardar";
    return "";
  });

  // ---------- carga de datos ----------
  let seq = 0;
  async function reload() {
    const my = ++seq;
    const f: Filter = { ...$state.snapshot(filter), search: search.trim() || null };
    const [it, ls, tg, ct] = await Promise.all([api.queryItems(f), api.lists(), api.tags(), api.counts()]);
    if (my !== seq) return;
    items = it;
    lists = ls;
    tags = tg;
    counts = ct;
    if (selectedId != null) {
      const found = it.find((i) => i.id === selectedId) ?? (await api.getItem(selectedId));
      if (found) selectedItem = found;
      else closeEditor();
    }
  }

  // Varios cambios seguidos → una sola recarga.
  let pending = false;
  function scheduleReload() {
    if (pending) return;
    pending = true;
    requestAnimationFrame(() => { pending = false; reload(); });
  }

  $effect(() => {
    void filter.view; void filter.listId; void filter.tag; void search;
    reload();
  });

  // Búsqueda con espera de 150 ms.
  $effect(() => {
    const s = searchText;
    if (!s.trim()) { search = ""; return; }
    const t = setTimeout(() => (search = s), 150);
    return () => clearTimeout(t);
  });

  // ---------- búsqueda del modo mínimo ----------
  let miniSearch = $state(false);

  async function openMiniSearch() {
    miniSearch = true;
    closeEditor();
    await tick();
    searchEl?.focus();
  }

  function closeMiniSearch() {
    searchText = "";
    miniSearch = false;
  }

  // ---------- selección ----------
  function select(it: Item) {
    selectedId = it.id;
    selectedItem = it;
  }
  function closeEditor() {
    selectedId = null;
    selectedItem = null;
  }

  // ---------- acciones ----------
  const toInput = (it: Item, patch: Partial<ItemInput> = {}): ItemInput => ({
    id: it.id, kind: it.kind, title: it.title, body: it.body, listId: it.listId, dueAt: it.dueAt,
    remindAt: it.remindAt, recur: it.recur, pinned: it.pinned, tags: it.tags, ...patch,
  });
  const name = (it: Item) => `«${(it.title || "Sin título").slice(0, 40)}»`;

  const actions: ItemActions = {
    select,
    async toggle(it) {
      const updated = await api.toggleDone(it.id);
      if (it.done) return;
      if (it.recur !== "none" && !updated.done) {
        const next = updated.dueAt ?? updated.remindAt;
        showToast(next ? `Hecha. Siguiente: ${fmt(next, updated.remindAt != null)}` : "Hecha", {
          label: "Deshacer",
          run: () => api.saveItem(toInput(it)),
        });
      } else {
        showToast(`${name(it)} completada`, { label: "Deshacer", run: () => api.toggleDone(it.id) });
      }
    },
    async remove(it) {
      await api.deleteItem(it.id);
      if (selectedId === it.id) closeEditor();
      showToast(`${name(it)} movida a la papelera`, {
        label: "Deshacer",
        run: () => api.restoreItem(it.id),
      });
    },
    async pin(it) {
      await api.saveItem(toInput(it, { pinned: !it.pinned }));
    },
    async setReminder(it, at) {
      await api.saveItem(toInput(it, { remindAt: at }));
      if (at != null) showToast(`Te avisaré ${fmt(at).toLowerCase()}`);
    },
    menu(e, it) {
      e.preventDefault();
      e.stopPropagation();
      menu = { x: e.clientX, y: e.clientY, item: it };
    },
  };

  async function onMenu(a: MenuAction) {
    const it = menu?.item;
    if (!it) return;
    if (a === "open") select(it);
    else if (a === "toggle") actions.toggle(it);
    else if (a === "pin") actions.pin(it);
    else if (a === "remove") actions.remove(it);
    else if (a === "convert") {
      const kind = it.kind === "task" ? "note" : "task";
      await api.saveItem(toInput(it, { kind, dueAt: kind === "note" ? null : it.dueAt }));
      showToast(`Convertida en ${kind === "note" ? "nota" : "tarea"}`);
    } else if (typeof a === "object") {
      await api.saveItem(toInput(it, { listId: a.move }));
      showToast(`Movida a «${lists.find((l) => l.id === a.move)?.name}»`);
    }
  }

  // ---------- teclado ----------
  function isTyping(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    return t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable;
  }

  function onKey(e: KeyboardEvent) {
    if (showSettings || menu) return;
    const k = e.key.toLowerCase();
    if (e.ctrlKey && e.shiftKey && k === "l") { e.preventDefault(); toggleTheme(); return; }
    if (e.ctrlKey && !e.shiftKey && k === "m") { e.preventDefault(); toggleMode(); return; }
    if (e.ctrlKey && !e.shiftKey && k === "p") { e.preventDefault(); toggleOnTop(); return; }
    if (e.ctrlKey && k === "n") { e.preventDefault(); newEl?.focus(); return; }
    if (e.ctrlKey && k === "f") {
      e.preventDefault();
      if (mini && !miniSearch) openMiniSearch();
      else { searchEl?.focus(); searchEl?.select(); }
      return;
    }
    if (e.key === "Escape" && mini && miniSearch && document.activeElement === searchEl) {
      closeMiniSearch();
      return;
    }
    if (e.key === "Escape") {
      if (searchText && document.activeElement === searchEl) { searchText = ""; return; }
      if (isTyping(e)) (e.target as HTMLElement).blur();
      else closeEditor();
      return;
    }
    if (isTyping(e) || !selectedItem) return;
    if (e.key === " " && selectedItem.kind === "task") { e.preventDefault(); actions.toggle(selectedItem); }
    else if (e.key === "Delete") { e.preventDefault(); actions.remove(selectedItem); }
    else if (e.key === "Enter") { e.preventDefault(); editor?.focusTitle(); }
    else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const ordered = [...items.filter((i) => i.kind === "task"), ...items.filter((i) => i.kind === "note")];
      const i = ordered.findIndex((x) => x.id === selectedId);
      const next = ordered[i + (e.key === "ArrowDown" ? 1 : -1)];
      if (next) select(next);
    }
  }

  onMount(() => {
    const unItems = listen("items-changed", scheduleReload);
    const unSettings = listen<[string, string]>("settings-changed", (e) => {
      if (e.payload[0] === "shortcut") shortcut = e.payload[1];
    });
    api.settings().then((s) => (shortcut = s.shortcut));
    initWindowState();
    // Se muestra la ventana cuando ya está pintada (sin parpadeo blanco).
    (async () => {
      await reload();
      if (!(await api.startupMinimized())) {
        requestAnimationFrame(async () => {
          const win = getCurrentWindow();
          await win.show();
          await win.setFocus();
        });
      }
    })();
    return () => { unItems.then((f) => f()); unSettings.then((f) => f()); };
  });
</script>

<svelte:window onkeydown={onKey} bind:innerWidth={width} />

{#if mini}
  <div class="mini">
    <header class="mini-head">
      {#if miniSearch}
        <div class="mini-search" in:fade={{ duration: 120 }}>
          <Icon name="search" size={14} />
          <input
            type="search"
            placeholder="Buscar tareas y notas…"
            aria-label="Buscar"
            bind:value={searchText}
            bind:this={searchEl}
          />
          <button class="icon-btn close" title="Cerrar búsqueda  (Esc)" aria-label="Cerrar búsqueda" onclick={closeMiniSearch}>
            <Icon name="x" size={14} />
          </button>
        </div>
      {:else}
      <div class="tabs" role="tablist" in:fade={{ duration: 120 }}>
        {#each MINI_TABS as t (t.view)}
          {@const n = counts[t.view]}
          <button
            role="tab"
            class="tab"
            class:active={filter.view === t.view}
            aria-selected={filter.view === t.view}
            title={t.label}
            onclick={() => { filter = { view: t.view }; closeEditor(); }}
          >
            <Icon name={t.icon} size={15} />
            <span>{t.label}</span>
            {#if n > 0}<span class="n">{n}</span>{/if}
          </button>
        {/each}
      </div>
      <button class="icon-btn tb-search" title="Buscar  (Ctrl+F)" aria-label="Buscar" onclick={openMiniSearch}>
        <Icon name="search" size={16} />
      </button>
      {/if}
      <WindowToolbar />
    </header>
    {#if selectedItem}
      {#key selectedItem.id}
        <Editor bind:this={editor} item={selectedItem} {lists} onclose={closeEditor} onremove={actions.remove} compact />
      {/key}
    {:else}
      <ItemList
        {items}
        {filter}
        {lists}
        {selectedId}
        {searching}
        {shortcut}
        {actions}
        compact
        oncreated={() => {}}
        bind:newInput={newEl}
      />
    {/if}
  </div>
{:else}
<div class="app" style:grid-template-columns="{compact ? 56 : 248}px 1fr {selectedItem ? '380px' : ''}">
  <Sidebar
    {lists}
    {tags}
    {counts}
    {filter}
    {compact}
    {shortcut}
    bind:search={searchText}
    bind:searchEl
    onselect={(f) => { filter = f; searchText = ""; closeEditor(); }}
    onsettings={() => (showSettings = true)}
  />

  <main class="content">
    <header>
      <div class="titles">
        <h1>{title}</h1>
        {#if subtitle}<div class="faint subtitle">{subtitle}</div>{/if}
      </div>
      <div class="head-right">
        {#if compact}
          <div class="search">
            <Icon name="search" size={14} />
            <input type="search" placeholder="Buscar" bind:value={searchText} bind:this={searchEl} />
          </div>
        {/if}
        <WindowToolbar />
      </div>
    </header>
    {#if filter.view === "trash" && !searching}
      <TrashList {items} />
    {:else}
    <ItemList
      {items}
      {filter}
      {lists}
      {selectedId}
      {searching}
      {shortcut}
      {actions}
      oncreated={(it) => { if (it.kind === "note") select(it); }}
      bind:newInput={newEl}
    />
    {/if}
  </main>

  {#if selectedItem}
    {#key selectedItem.id}
      <Editor bind:this={editor} item={selectedItem} {lists} onclose={closeEditor} onremove={actions.remove} />
    {/key}
  {/if}
</div>
{/if}

{#if menu}
  <ContextMenu x={menu.x} y={menu.y} item={menu.item} {lists} onaction={onMenu} onclose={() => (menu = null)} />
{/if}

{#if showSettings}
  <SettingsDialog onclose={() => (showSettings = false)} />
{/if}

<Toast />

<style>
  .app { display: grid; height: 100%; }
  .content {
    display: flex; flex-direction: column; min-width: 0; min-height: 0;
    background: var(--layer); border-top-left-radius: 8px; border: 1px solid var(--border); border-right: none; border-bottom: none;
  }
  header { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; padding: 22px 24px 16px; }
  .titles { min-width: 0; }
  h1 { margin: 0; font-family: var(--font-display); font-size: 26px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .subtitle { margin-top: 2px; }
  .subtitle::first-letter { text-transform: uppercase; }
  .head-right { display: flex; align-items: center; gap: 10px; flex: none; }

  /* ---------- modo mínimo ---------- */
  .mini { display: flex; flex-direction: column; height: 100%; background: var(--layer); }
  .mini-head { display: flex; align-items: center; gap: 6px; padding: 8px 8px 10px 10px; }
  .tabs { display: flex; gap: 2px; flex: 1; min-width: 0; background: var(--hover); border-radius: 8px; padding: 3px; }
  .tab {
    flex: 1; min-width: 0; display: inline-flex; align-items: center; justify-content: center; gap: 6px;
    border: none; background: transparent; color: var(--text-2); border-radius: 6px; padding: 5px 6px;
    font-size: 12.5px; cursor: default; white-space: nowrap;
  }
  .tab:hover { color: var(--text); }
  .tab.active { background: var(--card); color: var(--text); box-shadow: 0 1px 2px rgba(0, 0, 0, 0.15); }
  .tab .n { font-size: 11px; color: var(--accent); font-weight: 600; }
  .tab :global(svg) { flex: none; }
  /* Ventana estrecha: solo la pestaña activa muestra su nombre; las demás, icono y contador. */
  @media (max-width: 440px) {
    .tab:not(.active) { flex: 0 0 auto; padding: 5px 9px; }
    .tab:not(.active) span:not(.n) { display: none; }
  }
  .tb-search { width: 32px; height: 32px; border-radius: 6px; flex: none; }
  .mini-search {
    flex: 1; min-width: 0; display: flex; align-items: center; gap: 6px; padding: 0 2px 0 10px; height: 34px;
    background: var(--control); border: 1px solid var(--border); border-bottom: 2px solid var(--accent);
    border-radius: 8px; color: var(--text-2);
  }
  .mini-search input { flex: 1; min-width: 0; border: none; background: transparent; padding: 4px 2px; }
  .mini-search input:focus { border: none; padding-bottom: 4px; background: transparent; }
  .mini-search input::-webkit-search-cancel-button { display: none; }
  .mini-search .close { width: 26px; height: 26px; }

  .search {
    display: flex; align-items: center; gap: 6px; padding: 0 8px; width: 200px; flex: none;
    background: var(--control); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-2);
  }
  .search input { border: none; background: transparent; flex: 1; padding: 6px 2px; min-width: 0; }
  .search input:focus { border: none; padding-bottom: 6px; background: transparent; }
  .search:focus-within { border-bottom: 2px solid var(--accent); }
</style>
