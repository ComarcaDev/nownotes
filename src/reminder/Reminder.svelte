<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api, type Item } from "../lib/api";
  import { at, timeLabel, RECUR_LABEL } from "../lib/dates";
  import { displayTitle } from "../lib/items";
  import { playConfigured } from "../lib/sound";
  import Icon from "../lib/components/Icon.svelte";

  const MINUTE = 60_000;
  const appWindow = getCurrentWindow();
  let alerts = $state<Item[]>([]);
  let error = $state("");
  let known = new Set<number>();

  async function load() {
    try {
      alerts = await api.pendingAlerts();
    } catch (e) {
      error = `No se pudieron cargar los avisos: ${String(e)}`;
      return;
    }
    if (alerts.length === 0) {
      await appWindow.close();
      return;
    }
    // Suena solo cuando llega un aviso nuevo.
    const fresh = alerts.some((a) => !known.has(a.id));
    known = new Set(alerts.map((a) => a.id));
    if (fresh) playConfigured().catch(() => {});
  }

  /** Ejecuta la acción y recarga; si falla, lo muestra en la ventana. */
  async function run(action: () => Promise<unknown>) {
    error = "";
    try {
      await action();
    } catch (e) {
      error = `No se pudo completar la acción: ${String(e)}`;
    }
    await load();
  }

  const snooze = (it: Item, until: number) => run(() => api.snoozeAlert(it.id, until));
  const resolve = (it: Item, complete: boolean) => run(() => api.resolveAlert(it.id, complete));
  const dismissAll = () => run(async () => { for (const a of alerts) await api.resolveAlert(a.id, false); });

  onMount(() => {
    load();
    const un = listen("alerts-changed", load);
    return () => { un.then((f) => f()); };
  });
</script>

<div class="reminder">
  <header data-tauri-drag-region>
    <span class="bell"><Icon name="bell" size={15} /></span>
    <span data-tauri-drag-region class="head">Recordatorio{alerts.length > 1 ? `s (${alerts.length})` : ""}</span>
    <span class="drag" data-tauri-drag-region></span>
    <button class="icon-btn" title="Abrir NowNotes" aria-label="Abrir NowNotes" onclick={() => api.showMain()}><Icon name="all" size={15} /></button>
    <button class="icon-btn" title="Descartar todo" aria-label="Descartar todos los avisos" onclick={dismissAll}><Icon name="x" size={15} /></button>
  </header>

  <div class="cards">
    {#if error}<div class="error" role="alert">{error}</div>{/if}
    {#each alerts as it (it.id)}
      <div class="card">
        <div class="title">{displayTitle(it)}</div>
        {#if it.body.trim()}<div class="body faint">{it.body.trim().split("\n")[0]}</div>{/if}
        {#if it.recur !== "none" && it.remindAt}
          <div class="meta faint"><Icon name="repeat" size={11} /> {RECUR_LABEL[it.recur]} · próximo {timeLabel(it.remindAt)}</div>
        {/if}
        <div class="actions" role="group" aria-label="Posponer">
          <span class="faint small">Posponer</span>
          <button class="chip" onclick={() => snooze(it, Date.now() + 5 * MINUTE)}>5 min</button>
          <button class="chip" onclick={() => snooze(it, Date.now() + 60 * MINUTE)}>1 h</button>
          <button class="chip" onclick={() => snooze(it, at(1, 9))}>Mañana</button>
          <span class="grow"></span>
          {#if it.kind === "task"}
            <button class="btn primary small-btn" onclick={() => resolve(it, true)}><Icon name="done" size={13} />Hecho</button>
          {:else}
            <button class="btn small-btn" onclick={() => resolve(it, false)}>Vale</button>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .reminder { display: flex; flex-direction: column; height: 100%; }
  header { display: flex; align-items: center; gap: 8px; padding: 8px 8px 4px 14px; }
  .bell { color: var(--accent); display: grid; }
  .head { font-weight: 600; font-size: 13px; }
  .drag { flex: 1; align-self: stretch; }
  .cards { flex: 1; overflow-y: auto; padding: 4px 12px 12px; display: flex; flex-direction: column; gap: 6px; }
  .card { background: var(--card); border: 1px solid var(--border); border-radius: 6px; padding: 10px 12px; }
  .title { font-family: var(--font-display); font-size: 15px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .body { font-size: 12.5px; margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .meta { font-size: 11.5px; margin-top: 4px; display: flex; align-items: center; gap: 4px; }
  .actions { display: flex; align-items: center; gap: 5px; margin-top: 10px; }
  .small { font-size: 11.5px; margin-right: 2px; }
  .small-btn { padding: 4px 10px; font-size: 12.5px; }
  .error { font-size: 12px; color: var(--danger); padding: 4px 2px; }
</style>
