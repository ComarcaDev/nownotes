<script lang="ts">
  import { onMount } from "svelte";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { ask, open, save } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Icon from "./Icon.svelte";
  import ShortcutsSettings from "./ShortcutsSettings.svelte";
  import { api, type BackupInfo, type Info, type Settings } from "../api";
  import { SOUNDS, playSound } from "../sound";
  import { checkForUpdate, type UpdateInfo } from "../updates";

  let { onclose }: { onclose: () => void } = $props();

  let s = $state<Settings>({});
  let info = $state<Info | null>(null);
  let autostart = $state(false);
  let backups = $state<BackupInfo[]>([]);
  let message = $state("");
  let error = $state("");
  let checking = $state(false);
  let update = $state<UpdateInfo | null | undefined>(undefined);

  onMount(async () => {
    s = await api.settings();
    info = await api.info();
    backups = await api.listBackups();
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
    const path = await save({ defaultPath: `nownotes-${date}.json`, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (!path) return;
    try { await api.exportData(path); message = "Copia exportada."; } catch (e) { error = String(e); }
  }

  async function doImport() {
    const path = await open({ multiple: false, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (typeof path !== "string") return;
    try { const n = await api.importData(path); message = `Importados ${n} elementos.`; } catch (e) { error = String(e); }
  }

  function backupLabel(b: BackupInfo): string {
    const when = new Date(b.modified).toLocaleString("es-ES", { weekday: "short", day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
    const kind = b.name.includes("antes-de-restaurar") ? " · antes de restaurar" : "";
    return `${when}${kind}`;
  }

  async function restore(b: BackupInfo) {
    const ok = await ask(
      `Los datos actuales se sustituirán por los de la copia del ${backupLabel(b)}. Antes se guarda una copia del estado actual, por si quieres volver.`,
      { title: "¿Restaurar esta copia?", kind: "warning", okLabel: "Restaurar", cancelLabel: "Cancelar" }
    );
    if (!ok) return;
    try {
      await api.restoreBackup(b.name);
      location.reload(); // recarga la interfaz con los datos restaurados
    } catch (e) {
      error = String(e);
    }
  }

  async function checkNow() {
    checking = true;
    error = "";
    try {
      update = await checkForUpdate(info?.version ?? "0.0.0");
    } catch {
      error = "No se pudo comprobar: revisa la conexión a internet.";
    } finally {
      checking = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === "Escape") onclose(); }} />

<div class="backdrop" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) onclose(); }}>
  <div class="dialog" role="dialog" aria-label="Ajustes">
    <header>
      <h2>Ajustes</h2>
      <button class="icon-btn" title="Cerrar" onclick={onclose}><Icon name="x" /></button>
    </header>

    <div class="content">
      <ShortcutsSettings />

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
        <label class="row">
          <div><div>Buscar actualizaciones</div><div class="faint small">Una vez al día, consulta en GitHub si hay una versión nueva. No descarga nada.</div></div>
          <input type="checkbox" class="toggle" checked={s.update_check !== "0"} onchange={(e) => setKey("update_check", e.currentTarget.checked ? "1" : "0")} />
        </label>
        <div class="row">
          <div>
            <div>NowNotes {info?.version ?? ""}</div>
            {#if update === null}<div class="faint small">Tienes la última versión.</div>
            {:else if update}<div class="small accent">Versión {update.version} disponible.</div>{/if}
          </div>
          {#if update}
            <button class="btn primary" onclick={() => openUrl(update!.url)}>Ver novedades</button>
          {:else}
            <button class="btn" disabled={checking} onclick={checkNow}>{checking ? "Comprobando…" : "Buscar ahora"}</button>
          {/if}
        </div>
      </section>

      <section>
        <h3>Datos</h3>
        <div class="row">
          <div class="info">
            <div>{info?.portable ? "Modo portable" : "Instalada"}</div>
            <div class="faint small path" title={info?.dataDir}>{info?.dataDir}</div>
          </div>
        </div>
        <div class="row buttons">
          <button class="btn" onclick={doExport}>Exportar copia…</button>
          <button class="btn" onclick={doImport}>Importar…</button>
        </div>

        <div class="sub">
          <span>Copias de seguridad automáticas</span>
          <button class="btn subtle" onclick={() => api.openBackupsDir()}><Icon name="list" size={13} />Abrir carpeta</button>
        </div>
        {#if backups.length === 0}
          <div class="faint small empty">Se crea una copia al día; se conservan las últimas 7.</div>
        {:else}
          {#each backups as b (b.name)}
            <div class="row slim">
              <div class="small">{backupLabel(b)} <span class="faint">· {Math.max(1, Math.round(b.size / 1024))} KB</span></div>
              <button class="btn subtle" onclick={() => restore(b)}><Icon name="restore" size={13} />Restaurar</button>
            </div>
          {/each}
        {/if}
      </section>

      {#if error}<div class="msg error">{error}</div>{/if}
      {#if message}<div class="msg">{message}</div>{/if}
    </div>
  </div>
</div>

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.3); display: grid; place-items: center; z-index: 10; }
  .dialog {
    width: min(580px, calc(100vw - 24px)); max-height: calc(100vh - 32px); display: flex; flex-direction: column;
    background: var(--solid); border-radius: var(--radius); box-shadow: var(--shadow);
  }
  header { display: flex; align-items: center; justify-content: space-between; padding: 16px 16px 4px 24px; }
  h2 { margin: 0; font-family: var(--font-display); font-size: 20px; font-weight: 600; }
  .content { overflow-y: auto; padding: 0 24px 20px; }
  h3 { font-size: 13px; font-weight: 600; margin: 16px 0 6px; }
  .row {
    display: flex; align-items: center; justify-content: space-between; gap: 16px;
    background: var(--card); border: 1px solid var(--border); padding: 12px 14px; margin-bottom: 3px; border-radius: 6px;
  }
  .row.slim { padding: 6px 6px 6px 14px; }
  .row.buttons { justify-content: flex-start; gap: 8px; }
  .info { min-width: 0; }
  .sub { display: flex; align-items: center; justify-content: space-between; margin: 12px 2px 4px; font-size: 12px; color: var(--text-2); }
  .empty { padding: 4px 2px; }
  .ctl { display: flex; align-items: center; gap: 4px; }
  .small { font-size: 12px; }
  .accent { color: var(--accent); }
  .path { max-width: 400px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; user-select: text; }
  .toggle { appearance: none; width: 40px; height: 20px; border-radius: 10px; border: 1px solid var(--text-2); position: relative; flex: none; margin: 0; }
  .toggle::after { content: ""; position: absolute; width: 12px; height: 12px; border-radius: 50%; background: var(--text-2); top: 3px; left: 4px; transition: left 0.15s; }
  .toggle:checked { background: var(--accent); border-color: var(--accent); }
  .toggle:checked::after { left: 22px; background: var(--accent-text); }
  input[type="range"] { accent-color: var(--accent); width: 180px; }
  .msg { margin-top: 12px; padding: 8px 12px; border-radius: 6px; background: var(--accent-soft); }
  .msg.error { background: rgba(196, 43, 28, 0.12); color: var(--danger); }
</style>
