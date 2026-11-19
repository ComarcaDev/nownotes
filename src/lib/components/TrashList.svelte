<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import EmptyState from "./EmptyState.svelte";
  import { api, type Item } from "../api";
  import { DAY_MS } from "../dates";
  import { displayTitle, quoted } from "../items";
  import { confirmDanger } from "../confirm";
  import { getInfo } from "../info";
  import { safe } from "../safe";
  import { showToast } from "../toast.svelte";

  let { items }: { items: Item[] } = $props();

  // Lo decide el backend (db::TRASH_DAYS); 10 hasta que llega la respuesta.
  let trashDays = $state(10);
  onMount(() => {
    getInfo().then((i) => (trashDays = i.trashDays)).catch(() => {});
  });

  function daysLeft(it: Item): number {
    const left = (it.deletedAt ?? Date.now()) + trashDays * DAY_MS - Date.now();
    return Math.max(0, Math.ceil(left / DAY_MS));
  }

  function leftLabel(it: Item): string {
    const d = daysLeft(it);
    if (d <= 0) return "Se elimina hoy";
    return d === 1 ? "Se elimina mañana" : `Se elimina en ${d} días`;
  }

  async function restore(it: Item) {
    const ok = await safe(() => api.restoreItem(it.id).then(() => true), "No se pudo restaurar");
    if (ok) showToast(`${quoted(it)} restaurada`);
  }

  async function destroy(it: Item) {
    const ok = await confirmDanger(
      "¿Eliminar para siempre?",
      `${quoted(it, 80)} se eliminará definitivamente. No se puede deshacer.`,
      "Eliminar"
    );
    if (ok) await safe(() => api.deleteForever(it.id), "No se pudo eliminar");
  }

  async function emptyAll() {
    const ok = await confirmDanger(
      "¿Vaciar la papelera?",
      `Se eliminarán definitivamente ${items.length} elementos. No se puede deshacer.`,
      "Vaciar"
    );
    if (ok) await safe(() => api.emptyTrash(), "No se pudo vaciar la papelera");
  }
</script>

<div class="wrap">
  <div class="info">
    <Icon name="clock" size={14} />
    <span>Los elementos se borran definitivamente a los {trashDays} días.</span>
    <span class="grow"></span>
    {#if items.length}
      <button class="btn subtle danger" onclick={emptyAll}><Icon name="trash" size={14} />Vaciar papelera</button>
    {/if}
  </div>

  <div class="list">
    {#each items as it (it.id)}
      <div class="row" in:fly={{ y: -6, duration: 160 }}>
        <span class="kind" aria-label={it.kind === "note" ? "Nota" : "Tarea"}>
          <Icon name={it.kind === "note" ? "note" : "tasks"} size={16} />
        </span>
        <div class="main">
          <div class="title">{displayTitle(it)}</div>
          <div class="meta" class:soon={daysLeft(it) <= 1}>{leftLabel(it)}</div>
        </div>
        <button class="btn" onclick={() => restore(it)}><Icon name="restore" size={14} />Restaurar</button>
        <button class="icon-btn" title="Eliminar para siempre" aria-label="Eliminar para siempre" onclick={() => destroy(it)}>
          <Icon name="trash" size={15} />
        </button>
      </div>
    {/each}

    {#if items.length === 0}
      <EmptyState
        icon="trash"
        title="La papelera está vacía"
        hint={`Lo que elimines se guardará aquí ${trashDays} días por si necesitas recuperarlo.`}
        muted
      />
    {/if}
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; min-height: 0; flex: 1; }
  .info {
    display: flex; align-items: center; gap: 8px; margin: 0 24px 12px; padding: 6px 6px 6px 12px;
    font-size: 12.5px; color: var(--text-2); background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
    min-height: 40px;
  }
  .list { flex: 1; overflow-y: auto; padding: 0 24px 24px; }
  .row {
    display: flex; align-items: center; gap: 12px; padding: 8px 8px 8px 12px; margin-bottom: 3px;
    border-radius: 6px; background: var(--card); border: 1px solid var(--border);
  }
  .kind { color: var(--text-3); display: grid; }
  .main { flex: 1; min-width: 0; }
  .title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-2); }
  .meta { font-size: 12px; color: var(--text-3); margin-top: 1px; }
  .meta.soon { color: var(--danger); }
</style>
