<script lang="ts">
  import { onMount } from "svelte";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import Icon from "./Icon.svelte";
  import { api, type Info, type Settings } from "../api";
  import { SOUNDS, playSound } from "../sound";

  let { onclose }: { onclose: () => void } = $props();

  let s = $state<Settings>({});
  let info = $state<Info | null>(null);
  let autostart = $state(false);
  let capturing = $state(false);
  let message = $state("");
  let error = $state("");

  onMount(async () => {
    s = await api.settings();
    info = await api.info();
    try { autostart = await isEnabled(); } catch { autostart = false; }
  });

  async function setKey(key: string, value: string) {
    error = "";
    try {
      await api.setSetting(key, value);
      s[key] = value;
    } catch (e) {
      error = String(e);
    }
  }

  function capture(e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") { capturing = false; return; }
    if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return;
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Super");
    if (!parts.length) { error = "Usa al menos una tecla modificadora (Ctrl, Alt, Shift…)."; return; }
    let k = e.code;
    if (k.startsWith("Key")) k = k.slice(3);
    else if (k.startsWith("Digit")) k = k.slice(5);
    parts.push(k);
    capturing = false;
    setKey("shortcut", parts.join("+"));
  }

  async function toggleAutostart() {
    try {
      if (autostart) await disable(); else await enable();
      autostart = await isEnabled();
    } catch (e) {
      error = String(e);
    }
  }

  async function pickSound() {
    const path = await open({ multiple: false, filters: [{ name: "Audio", extensions: ["wav", "mp3", "ogg"] }] });
    if (typeof path === "string") {
      await setKey("sound_path", path);
      await setKey("sound", "custom");
    }
  }

  async function doExport() {
    const date = new Date().toISOString().slice(0, 10);
    const path = await save({ defaultPath: `apunta-${date}.json`, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (!path) return;
    try { await api.exportData(path); message = "Copia exportada."; } catch (e) { error = String(e); }
  }

  async function doImport() {
    const path = await open({ multiple: false, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (typeof path !== "string") return;
    try { const n = await api.importData(path); message = `Importados ${n} elementos.`; } catch (e) { error = String(e); }
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === "Escape" && !capturing) onclose(); }} />

<div class="backdrop" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) onclose(); }}>
  <div class="dialog" role="dialog" aria-label="Ajustes">
    <header>
      <h2>Ajustes</h2>
      <button class="icon-btn" title="Cerrar" onclick={onclose}><Icon name="x" /></button>
    </header>

    <div class="content">
      <section>
        <h3>Captura rápida</h3>
        <div class="row">
          <div><div>Atajo global</div><div class="faint small">Abre la ventana de captura desde cualquier programa.</div></div>
          <button class="btn shortcut" class:capturing onclick={() => (capturing = true)} onkeydown={(e) => capturing && capture(e)}>
            {capturing ? "Pulsa la combinación…" : s.shortcut || "Sin atajo"}
          </button>
        </div>
      </section>

      <section>
        <h3>Recordatorios</h3>
        <div class="row">
          <div>Sonido</div>
          <div class="ctl">
            <select value={s.sound} onchange={(e) => { const v = e.currentTarget.value; if (v === "custom" && !s.sound_path) pickSound(); else setKey("sound", v); }}>
              {#each SOUNDS as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
            </select>
            <button class="icon-btn" title="Probar" onclick={() => playSound(s.sound, parseFloat(s.volume), s.sound_path)}><Icon name="play" /></button>
          </div>
        </div>
        {#if s.sound === "custom"}
          <div class="row">
            <div class="faint small path" title={s.sound_path}>{s.sound_path || "Ningún archivo"}</div>
            <button class="btn" onclick={pickSound}>Elegir archivo…</button>
          </div>
        {/if}
        <div class="row">
          <div>Volumen</div>
          <input type="range" min="0" max="1" step="0.05" value={s.volume} onchange={(e) => setKey("volume", e.currentTarget.value)} />
        </div>
        <label class="row">
          <div><div>Notificación de Windows</div><div class="faint small">Además de la ventana de aviso, enviar una notificación al centro de actividades.</div></div>
          <input type="checkbox" class="toggle" checked={s.native_toast !== "0"} onchange={(e) => setKey("native_toast", e.currentTarget.checked ? "1" : "0")} />
        </label>
      </section>

      <section>
        <h3>General</h3>
        <label class="row">
          <div><div>Seguir en la bandeja al cerrar</div><div class="faint small">Necesario para recibir avisos con la ventana cerrada.</div></div>
          <input type="checkbox" class="toggle" checked={s.close_to_tray !== "0"} onchange={(e) => setKey("close_to_tray", e.currentTarget.checked ? "1" : "0")} />
        </label>
        <label class="row">
          <div><div>Iniciar con Windows</div><div class="faint small">Arranca minimizada en la bandeja.</div></div>
          <input type="checkbox" class="toggle" checked={autostart} onchange={toggleAutostart} />
        </label>
      </section>

      <section>
        <h3>Datos</h3>
        <div class="row">
          <div>
            <div>{info?.portable ? "Modo portable" : "Instalada"}</div>
            <div class="faint small path" title={info?.dataDir}>{info?.dataDir}</div>
          </div>
        </div>
        <div class="row buttons">
          <button class="btn" onclick={doExport}>Exportar copia…</button>
          <button class="btn" onclick={doImport}>Importar…</button>
        </div>
      </section>

      {#if error}<div class="msg error">{error}</div>{/if}
      {#if message}<div class="msg">{message}</div>{/if}
      <div class="faint small version">Apunta {info?.version ?? ""}</div>
    </div>
  </div>
</div>

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.3); display: grid; place-items: center; z-index: 10; }
  .dialog {
    width: min(560px, calc(100vw - 32px)); max-height: calc(100vh - 48px); display: flex; flex-direction: column;
    background: var(--solid); border-radius: var(--radius); box-shadow: var(--shadow);
  }
  header { display: flex; align-items: center; justify-content: space-between; padding: 16px 16px 4px 24px; }
  h2 { margin: 0; font-family: var(--font-display); font-size: 20px; font-weight: 600; }
  .content { overflow-y: auto; padding: 8px 24px 20px; }
  h3 { font-size: 13px; font-weight: 600; margin: 16px 0 6px; }
  .row {
    display: flex; align-items: center; justify-content: space-between; gap: 16px;
    background: var(--card); border: 1px solid var(--border); padding: 12px 14px; margin-bottom: 3px; border-radius: 6px;
  }
  .row.buttons { justify-content: flex-start; gap: 8px; }
  .ctl { display: flex; align-items: center; gap: 4px; }
  .small { font-size: 12px; }
  .path { max-width: 340px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; user-select: text; }
  .shortcut { min-width: 150px; justify-content: center; font-family: Consolas, monospace; }
  .shortcut.capturing { border-color: var(--accent); color: var(--accent); }
  .toggle { appearance: none; width: 40px; height: 20px; border-radius: 10px; border: 1px solid var(--text-2); position: relative; flex: none; margin: 0; }
  .toggle::after { content: ""; position: absolute; width: 12px; height: 12px; border-radius: 50%; background: var(--text-2); top: 3px; left: 4px; transition: left 0.15s; }
  .toggle:checked { background: var(--accent); border-color: var(--accent); }
  .toggle:checked::after { left: 22px; background: var(--accent-text); }
  input[type="range"] { accent-color: var(--accent); width: 180px; }
  .msg { margin-top: 12px; padding: 8px 12px; border-radius: 6px; background: var(--accent-soft); }
  .msg.error { background: rgba(196, 43, 28, 0.12); color: var(--danger); }
  .version { margin-top: 14px; text-align: right; }
</style>
