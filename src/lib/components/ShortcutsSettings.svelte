<script lang="ts">
  // Sección «Atajos» de Ajustes: cambiar, quitar y restablecer cada atajo.
  import {
    APP_ACTIONS, GLOBAL_ACTIONS, keys, comboFromEvent, display, validate,
    setAppKey, setGlobalKey, type AppAction, type GlobalAction,
  } from "../keys.svelte";
  import Icon from "./Icon.svelte";
  import ResetButton from "./ResetButton.svelte";
  import { resetWithUndo, SETTING_DEFAULTS } from "../defaults";

  type Row = { id: string; label: string; global: boolean; def: string; setting?: string; hint?: string };
  const rows: Row[] = [
    ...GLOBAL_ACTIONS.map((g) => ({ id: g.id, label: g.label, global: true, def: g.def, setting: g.setting, hint: "Funciona desde cualquier programa" })),
    ...APP_ACTIONS.map((a) => ({ id: a.id, label: a.label, global: false, def: a.def })),
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

  const anyChanged = $derived(rows.some((r) => current(r) !== r.def));

  function resetRow(r: Row) {
    messages[r.id] = {};
    const values: Record<string, string> = r.global
      ? { [r.setting!]: r.def }
      : { keys: JSON.stringify({ ...keys.app, [r.id]: r.def }) };
    resetWithUndo(`el atajo «${r.label}»`, values);
  }

  function resetAll() {
    messages = {};
    resetWithUndo("los atajos", {
      keys: SETTING_DEFAULTS.keys,
      shortcut: SETTING_DEFAULTS.shortcut,
      shortcut_toggle: SETTING_DEFAULTS.shortcut_toggle,
    });
  }
</script>

<section>
  <h3>Atajos de teclado</h3>
  {#each rows as r, i (r.id)}
    {#if i === GLOBAL_ACTIONS.length}<div class="sub">En la app</div>{:else if i === 0}<div class="sub">Globales</div>{/if}
    <div class="row">
      <div class="info">
        <div class="lbl">{r.label}<ResetButton show={current(r) !== r.def} label={`el atajo (${display(r.def)})`} onclick={() => resetRow(r)} /></div>
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
    {#if anyChanged}<button class="btn subtle" onclick={resetAll}><Icon name="restore" size={13} />Restablecer atajos</button>{/if}
  </div>
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
  .lbl { display: inline-flex; align-items: center; min-height: 20px; }
  .small { font-size: 12px; }
  .combo { min-width: 160px; justify-content: center; font-family: Consolas, monospace; font-size: 12.5px; }
  .combo.capturing { border-color: var(--accent); color: var(--accent); }
  .msg { font-size: 12px; margin-top: 2px; }
  .msg.error { color: var(--danger); }
  .msg.warn { color: var(--warn); }
  .msg.ok { color: var(--accent); }
</style>
