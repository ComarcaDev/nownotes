<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { api, type Counts, type Filter, type Item, type List, type TagCount } from "../lib/api";
  import type { ItemActions, MenuAction } from "../lib/actions";
  import { showToast } from "../lib/toast.svelte";
  import { fmt } from "../lib/dates";
  import { quoted, toInput } from "../lib/items";
  import { toggleCheckLine } from "../lib/markdown";
  import { safe } from "../lib/safe";
  import { getInfo } from "../lib/info";
  import { win, initWindowState, toggleTheme, toggleMode, toggleOnTop } from "../lib/window.svelte";
  import { keys, initKeys, matches, display } from "../lib/keys.svelte";
  import { checkForUpdate, dueForCheck } from "../lib/updates";
  import Sidebar from "../lib/components/Sidebar.svelte";
  import ItemList from "../lib/components/ItemList.svelte";
  import Editor from "../lib/components/Editor.svelte";
  import Toast from "../lib/components/Toast.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import WindowToolbar from "../lib/components/WindowToolbar.svelte";

  // Carga diferida: solo se descargan al usarse (menos JS al arrancar).
  const loadSettings = () => import("../lib/components/Settings.svelte");
  const loadContextMenu = () => import("../lib/components/ContextMenu.svelte");
  const loadTrash = () => import("../lib/components/TrashList.svelte");

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
  const shortcut = $derived(keys.global.quick);
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
    const loaded = await safe(
      () => Promise.all([api.queryItems(f), api.lists(), api.tags(), api.counts()]),
      "No se pudieron cargar los datos"
    );
    if (!loaded || my !== seq) return;
    const [it, ls, tg, ct] = loaded;
    items = it;
    lists = ls;
    tags = tg;
    counts = ct;
    if (selectedId != null) {
      const found = it.find((i) => i.id === selectedId) ?? (await api.getItem(selectedId).catch(() => null));
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
  // Todas pasan por `safe`: si algo falla, se avisa con un toast en lugar de fallar en silencio.
  const actions: ItemActions = {
    select,
    async toggle(it) {
      const updated = await safe(() => api.toggleDone(it.id));
      if (!updated || it.done) return;
      if (it.recur !== "none" && !updated.done) {
        const next = updated.dueAt ?? updated.remindAt;
        showToast(next ? `Hecha. Siguiente: ${fmt(next, updated.remindAt != null)}` : "Hecha", {
          label: "Deshacer",
          run: () => safe(() => api.saveItem(toInput(it))),
        });
      } else {
        showToast(`${quoted(it)} completada`, { label: "Deshacer", run: () => safe(() => api.toggleDone(it.id)) });
      }
    },
    async remove(it) {
      const ok = await safe(() => api.deleteItem(it.id).then(() => true), "No se pudo mover a la papelera");
      if (!ok) return;
      if (selectedId === it.id) closeEditor();
      showToast(`${quoted(it)} movida a la papelera`, {
        label: "Deshacer",
        run: () => safe(() => api.restoreItem(it.id)),
      });
    },
    async pin(it) {
      await safe(() => api.saveItem(toInput(it, { pinned: !it.pinned })));
    },
    async setReminder(it, at) {
      const saved = await safe(() => api.saveItem(toInput(it, { remindAt: at })));
      if (saved && at != null) showToast(`Te avisaré ${fmt(at).toLowerCase()}`);
    },
    menu(e, it) {
      e.preventDefault();
      e.stopPropagation();
      menu = { x: e.clientX, y: e.clientY, item: it };
    },
    async toggleCheck(it, line) {
      await safe(() => api.saveItem(toInput(it, { body: toggleCheckLine(it.body, line) })));
    },
    async reorder(ids) {
      // Orden optimista: se aplica ya en pantalla y luego se guarda (si falla, la recarga lo deshace).
      const pos = new Map(ids.map((id, i) => [id, i]));
      const moved = items.filter((i) => pos.has(i.id)).sort((a, b) => pos.get(a.id)! - pos.get(b.id)!);
      let k = 0;
      items = items.map((i) => (pos.has(i.id) ? moved[k++] : i));
      const ok = await safe(() => api.reorderItems(ids).then(() => true), "No se pudo guardar el orden");
      if (!ok) reload();
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
      const saved = await safe(() => api.saveItem(toInput(it, { kind, dueAt: kind === "note" ? null : it.dueAt })));
      if (saved) showToast(`Convertida en ${kind === "note" ? "nota" : "tarea"}`);
    } else if (typeof a === "object") {
      const saved = await safe(() => api.saveItem(toInput(it, { listId: a.move })));
      if (saved) showToast(`Movida a «${lists.find((l) => l.id === a.move)?.name}»`);
    }
  }

  // ---------- actualizaciones ----------
  /** Aviso discreto de versión nueva (como mucho una vez al día, si está activado). */
  async function notifyUpdate() {
    try {
      const [s, info] = await Promise.all([api.settings(), getInfo()]);
      if (s.update_check === "0" || !dueForCheck()) return;
      const up = await checkForUpdate(info.version);
      if (up) {
        showToast(`NowNotes ${up.version} disponible`, { label: "Ver novedades", run: () => openUrl(up.url) }, 15000);
      }
    } catch {
      /* sin conexión: se intentará otro día */
    }
  }

  // ---------- teclado ----------
  function isTyping(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    return t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable;
  }

  function onKey(e: KeyboardEvent) {
    if (showSettings || menu) return;
    // Atajos configurables (Ajustes → Atajos)
    if (matches(e, "theme")) { e.preventDefault(); toggleTheme(); return; }
    if (matches(e, "mode")) { e.preventDefault(); toggleMode(); return; }
    if (matches(e, "pin")) { e.preventDefault(); toggleOnTop(); return; }
    if (matches(e, "new")) { e.preventDefault(); if (mini && miniSearch) closeMiniSearch(); newEl?.focus(); return; }
    if (matches(e, "search")) {
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
    initWindowState().catch((e) => console.error("Estado de la ventana", e));
    initKeys().catch((e) => console.error("Atajos", e));
    // Se muestra la ventana cuando ya está pintada (sin parpadeo blanco).
    (async () => {
      await reload();
      const minimized = await api.startupMinimized().catch(() => false);
      if (!minimized) {
        requestAnimationFrame(async () => {
          const appWindow = getCurrentWindow();
          await appWindow.show();
          await appWindow.setFocus();
        });
      }
      notifyUpdate();
    })();
    return () => { unItems.then((f) => f()); };
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
      <button class="icon-btn tb-search" title="Buscar  ({display(keys.app.search)})" aria-label="Buscar" onclick={openMiniSearch}>
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
      {#await loadTrash() then m}<m.default {items} />{/await}
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
  {@const m0 = menu}
  {#await loadContextMenu() then m}
    <m.default x={m0.x} y={m0.y} item={m0.item} {lists} onaction={onMenu} onclose={() => (menu = null)} />
  {/await}
{/if}

{#if showSettings}
  {#await loadSettings() then m}<m.default onclose={() => (showSettings = false)} />{/await}
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
