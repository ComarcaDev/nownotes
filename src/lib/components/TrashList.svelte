<script lang="ts">
  import { fly } from "svelte/transition";
  import { ask } from "@tauri-apps/plugin-dialog";
  import Icon from "./Icon.svelte";
  import { api, type Item } from "../api";
  import { showToast } from "../toast.svelte";

  let { items }: { items: Item[] } = $props();

  const TRASH_DAYS = 10;
  const DAY = 86_400_000;

  function daysLeft(it: Item): number {
    const left = (it.deletedAt ?? Date.now()) + TRASH_DAYS * DAY - Date.now();
    return Math.max(0, Math.ceil(left / DAY));
  }

  function leftLabel(it: Item): string {
    const d = daysLeft(it);
    if (d <= 0) return "Se elimina hoy";
    return d === 1 ? "Se elimina mañana" : `Se elimina en ${d} días`;
  }

  async function restore(it: Item) {
    await api.restoreItem(it.id);
    showToast(`«${(it.title || "Sin título").slice(0, 40)}» restaurada`);
  }

  async function destroy(it: Item) {
    const ok = await ask(`«${it.title || "Sin título"}» se eliminará definitivamente. No se puede deshacer.`, {
      title: "¿Eliminar para siempre?",
      kind: "warning",
      okLabel: "Eliminar",
      cancelLabel: "Cancelar",
    });
    if (ok) await api.deleteForever(it.id);
  }

  async function emptyAll() {
    const ok = await ask(`Se eliminarán definitivamente ${items.length} elementos. No se puede deshacer.`, {
      title: "¿Vaciar la papelera?",
      kind: "warning",
      okLabel: "Vaciar",
      cancelLabel: "Cancelar",
    });
    if (ok) await api.emptyTrash();
  }
</script>

<div class="wrap">
  <div class="info">
    <Icon name="clock" size={14} />
    <span>Los elementos se borran definitivamente a los {TRASH_DAYS} días.</span>
    <span class="grow"></span>
    {#if items.length}
      <button class="btn subtle danger" onclick={emptyAll}><Icon name="trash" size={14} />Vaciar papelera</button>
    {/if}
  </div>

  <div class="list">
    {#each items as it (it.id)}
      <div class="row" in:fly={{ y: -6, duration: 160 }}>
        <span class="kind"><Icon name={it.kind === "note" ? "note" : "tasks"} size={16} /></span>
        <div class="main">
          <div class="title">{it.title || "Sin título"}</div>
          <div class="meta" class:soon={daysLeft(it) <= 1}>{leftLabel(it)}</div>
        </div>
        <button class="btn" onclick={() => restore(it)}><Icon name="restore" size={14} />Restaurar</button>
        <button class="icon-btn" title="Eliminar para siempre" onclick={() => destroy(it)}><Icon name="trash" size={15} /></button>
      </div>
    {/each}

    {#if items.length === 0}
      <div class="empty">
        <div class="empty-icon"><Icon name="trash" size={32} /></div>
        <div class="empty-title">La papelera está vacía</div>
        <div class="faint">Lo que elimines se guardará aquí {TRASH_DAYS} días por si necesitas recuperarlo.</div>
      </div>
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
  .grow { flex: 1; }
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
  .empty { display: flex; flex-direction: column; align-items: center; text-align: center; gap: 6px; padding: 56px 24px; }
  .empty-icon {
    display: grid; place-items: center; width: 72px; height: 72px; border-radius: 50%;
    background: var(--hover); color: var(--text-2); margin-bottom: 8px;
  }
  .empty-title { font-family: var(--font-display); font-size: 16px; font-weight: 600; }
</style>
