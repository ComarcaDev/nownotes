<script lang="ts">
  // Sección «Atajos» de Ajustes: cambiar, quitar y restablecer cada atajo.
  import {
    APP_ACTIONS, GLOBAL_ACTIONS, keys, comboFromEvent, display, validate,
    setAppKey, setGlobalKey, resetAllKeys, type AppAction, type GlobalAction,
  } from "../keys.svelte";

  type Row = { id: string; label: string; global: boolean; hint?: string };
  const rows: Row[] = [
    ...GLOBAL_ACTIONS.map((g) => ({ id: g.id, label: g.label, global: true, hint: "Funciona desde cualquier programa" })),
    ...APP_ACTIONS.map((a) => ({ id: a.id, label: a.label, global: false })),
  ];

  let capturing = $state<string | null>(null);
  let messages = $state<Record<string, { error?: string; warning?: string; ok?: string }>>({});

  const current = (r: Row) => (r.global ? keys.global[r.id as GlobalAction] : keys.app[r.id as AppAction]);

  async function apply(r: Row, combo: string) {
    const check = validate(combo, r.id, r.global);
    if (check.error) {
      messages[r.id] = { error: check.error };
      return;
    }
    try {
      if (r.global) await setGlobalKey(r.id as GlobalAction, combo);
      else await setAppKey(r.id as AppAction, combo);
      messages[r.id] = check.warning ? { warning: check.warning } : { ok: combo ? "Guardado" : "Atajo quitado" };
    } catch (e) {
      messages[r.id] = { error: String(e) };
    }
  }

  function onKey(e: KeyboardEvent, r: Row) {
    if (capturing !== r.id) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") { capturing = null; return; }
    if (e.key === "Backspace" || e.key === "Delete") { capturing = null; apply(r, ""); return; }
    const combo = comboFromEvent(e);
    if (!combo) return;
    capturing = null;
    apply(r, combo);
  }

  async function resetAll() {
    const errors = await resetAllKeys();
    messages = errors.length ? { _all: { error: errors.join(" ") } } : { _all: { ok: "Atajos restablecidos" } };
  }
</script>

<section>
  <h3>Atajos de teclado</h3>
  {#each rows as r, i (r.id)}
    {#if i === GLOBAL_ACTIONS.length}<div class="sub">En la app</div>{:else if i === 0}<div class="sub">Globales</div>{/if}
    <div class="row">
      <div class="info">
        <div>{r.label}</div>
        {#if messages[r.id]?.error}<div class="msg error">{messages[r.id].error}</div>
        {:else if messages[r.id]?.warning}<div class="msg warn">{messages[r.id].warning}</div>
        {:else if messages[r.id]?.ok}<div class="msg ok">{messages[r.id].ok}</div>
        {:else if r.hint}<div class="faint small">{r.hint}</div>{/if}
      </div>
      <button
        class="btn combo"
        class:capturing={capturing === r.id}
        title="Pulsa para cambiar · Supr para quitar · Esc para cancelar"
        onclick={() => { capturing = r.id; messages[r.id] = {}; }}
        onkeydown={(e) => onKey(e, r)}
        onblur={() => { if (capturing === r.id) capturing = null; }}
      >
        {capturing === r.id ? "Pulsa la combinación…" : display(current(r))}
      </button>
    </div>
  {/each}
  <div class="row foot">
    <span class="faint small">Pulsa un atajo para cambiarlo. Supr lo quita y Esc cancela.</span>
    <button class="btn subtle" onclick={resetAll}>Restablecer</button>
  </div>
  {#if messages._all?.error}<div class="msg error">{messages._all.error}</div>{/if}
  {#if messages._all?.ok}<div class="msg ok">{messages._all.ok}</div>{/if}
</section>

<style>
  h3 { font-size: 13px; font-weight: 600; margin: 16px 0 6px; }
  .sub { font-size: 11.5px; color: var(--text-3); margin: 8px 2px 4px; text-transform: uppercase; letter-spacing: 0.04em; }
  .row {
    display: flex; align-items: center; justify-content: space-between; gap: 16px;
    background: var(--card); border: 1px solid var(--border); padding: 10px 14px; margin-bottom: 3px; border-radius: 6px;
  }
  .row.foot { background: transparent; border: none; padding: 6px 2px; }
  .info { min-width: 0; }
  .small { font-size: 12px; }
  .combo { min-width: 160px; justify-content: center; font-family: Consolas, monospace; font-size: 12.5px; }
  .combo.capturing { border-color: var(--accent); color: var(--accent); }
  .msg { font-size: 12px; margin-top: 2px; }
  .msg.error { color: var(--danger); }
  .msg.warn { color: var(--warn); }
  .msg.ok { color: var(--accent); }
</style>
