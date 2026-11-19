<script lang="ts">
  // Sección «Apariencia» de Ajustes: tema, acento, tono y fondo, con vista previa en vivo.
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import Icon from "./Icon.svelte";
  import { api } from "../api";
  import {
    ACCENTS, DEFAULT_APPEARANCE, IMAGE_HINT, IMAGE_MIN_WIDTH, IMAGE_POSITIONS, TONES,
    type Appearance, type Background,
  } from "../appearance";
  import { contrast, hexToRgb } from "../colors";
  import { confirmDanger } from "../confirm";
  import { currentAppearance, isDarkNow, previewAppearance } from "../theme";

  let { onerror }: { onerror: (msg: string) => void } = $props();

  let a = $state<Appearance>({ ...currentAppearance() });
  let theme = $state("dark");
  let hasImage = $state(false);
  let imageNote = $state("");
  let busy = $state(false);

  onMount(() => {
    api.settings().then((s) => (theme = s.theme || "dark")).catch(() => {});
    api.backgroundPath().then((p) => (hasImage = !!p)).catch(() => {});
  });

  const THEMES = [
    { id: "dark", label: "Oscuro" },
    { id: "light", label: "Claro" },
    { id: "system", label: "Sistema" },
  ];
  const BACKGROUNDS: { id: Background; label: string }[] = [
    { id: "mica", label: "Mica" },
    { id: "solid", label: "Color" },
    { id: "image", label: "Imagen" },
  ];

  const customAccent = $derived(!!a.accent && !ACCENTS.includes(a.accent.toLowerCase()));
  // Aviso si el color de fondo elegido apenas contrasta con el texto.
  const lowContrast = $derived.by(() => {
    void theme;
    const bg = a.bg === "solid" && a.bgColor ? hexToRgb(a.bgColor) : null;
    const text = isDarkNow() ? ([255, 255, 255] as const) : ([27, 27, 27] as const);
    return !!bg && contrast(bg, [...text]) < 4.5;
  });

  /** Cambia la apariencia: se ve al momento y, si `persist`, se guarda (y llega a las demás ventanas). */
  async function set(patch: Partial<Appearance>, persist = true) {
    a = { ...a, ...patch };
    previewAppearance(a);
    if (!persist) return;
    try {
      await api.setSetting("appearance", JSON.stringify(a));
    } catch (e) {
      onerror(String(e));
    }
  }

  async function setTheme(id: string) {
    theme = id;
    try {
      await api.setSetting("theme", id);
    } catch (e) {
      onerror(String(e));
    }
  }

  /** Ancho real de la imagen guardada, para avisar si es pequeña. */
  async function imageWidth(path: string): Promise<number> {
    const url = URL.createObjectURL(new Blob([await api.readFileBytes(path)]));
    try {
      const img = new Image();
      img.src = url;
      await img.decode();
      return img.naturalWidth;
    } finally {
      URL.revokeObjectURL(url);
    }
  }

  /** Elige una imagen. Devuelve false si se canceló o falló. */
  async function pickImage(): Promise<boolean> {
    const path = await open({ multiple: false, filters: [{ name: "Imagen", extensions: ["jpg", "jpeg", "png", "webp"] }] });
    if (typeof path !== "string") return false;
    busy = true;
    imageNote = "";
    try {
      const saved = await api.setBackground(path);
      hasImage = true;
      const w = await imageWidth(saved).catch(() => 0);
      if (w && w < IMAGE_MIN_WIDTH) imageNote = `La imagen mide ${w} px de ancho: puede verse borrosa. ${IMAGE_HINT.split(".")[0]}.`;
      await set({ bg: "image", imageRev: a.imageRev + 1 });
      return true;
    } catch (e) {
      onerror(String(e));
      return false;
    } finally {
      busy = false;
    }
  }

  async function chooseBackground(id: Background) {
    if (id === "image" && !hasImage) {
      await pickImage(); // si se cancela, el fondo no cambia
      return;
    }
    await set({ bg: id });
  }

  async function removeImage() {
    try {
      await api.clearBackground();
      hasImage = false;
      imageNote = "";
      await set({ bg: "mica" });
    } catch (e) {
      onerror(String(e));
    }
  }

  async function reset() {
    const ok = await confirmDanger(
      "¿Restablecer la apariencia?",
      "Vuelve el tema oscuro con fondo Mica y el acento azul. La imagen de fondo se quita.",
      "Restablecer"
    );
    if (!ok) return;
    imageNote = "";
    await setTheme("dark");
    if (hasImage) await api.clearBackground().catch(() => {});
    hasImage = false;
    await set({ ...DEFAULT_APPEARANCE, imageRev: a.imageRev + 1 });
  }
</script>

<section>
  <div class="head">
    <h3>Apariencia</h3>
    <button class="btn subtle small-btn" onclick={reset}><Icon name="restore" size={13} />Restablecer</button>
  </div>

  <div class="row">
    <div>Tema</div>
    <div class="segmented" role="radiogroup" aria-label="Tema">
      {#each THEMES as t (t.id)}
        <button role="radio" aria-checked={theme === t.id} class:active={theme === t.id} onclick={() => setTheme(t.id)}>{t.label}</button>
      {/each}
    </div>
  </div>

  <div class="row col">
    <div>Color de acento</div>
    <div class="swatches" role="radiogroup" aria-label="Color de acento">
      <button
        class="swatch auto"
        class:on={!a.accent}
        role="radio"
        aria-checked={!a.accent}
        title="El del tema"
        aria-label="Acento del tema"
        onclick={() => set({ accent: "" })}
      >A</button>
      {#each ACCENTS as c (c)}
        <button
          class="swatch"
          class:on={a.accent.toLowerCase() === c}
          style:background={c}
          role="radio"
          aria-checked={a.accent.toLowerCase() === c}
          aria-label={`Acento ${c}`}
          title={c}
          onclick={() => set({ accent: c })}
        ></button>
      {/each}
      <label class="swatch custom" class:on={customAccent} title="Color personalizado" style:background={customAccent ? a.accent : undefined}>
        {#if !customAccent}<Icon name="plus" size={12} />{/if}
        <input
          type="color"
          aria-label="Acento personalizado"
          value={a.accent || "#0067c0"}
          oninput={(e) => set({ accent: e.currentTarget.value }, false)}
          onchange={(e) => set({ accent: e.currentTarget.value })}
        />
      </label>
    </div>
  </div>

  <div class="row">
    <div><div>Tono</div><div class="faint small">Tiñe suavemente paneles y tarjetas.</div></div>
    <div class="segmented" role="radiogroup" aria-label="Tono de las superficies">
      {#each TONES as t (t.id)}
        <button role="radio" aria-checked={a.tone === t.id} class:active={a.tone === t.id} onclick={() => set({ tone: t.id })}>{t.label}</button>
      {/each}
    </div>
  </div>

  <div class="row">
    <div>Fondo</div>
    <div class="segmented" role="radiogroup" aria-label="Fondo">
      {#each BACKGROUNDS as b (b.id)}
        <button role="radio" aria-checked={a.bg === b.id} class:active={a.bg === b.id} disabled={busy} onclick={() => chooseBackground(b.id)}>{b.label}</button>
      {/each}
    </div>
  </div>

  {#if a.bg === "solid"}
    <div class="row sub-row">
      <div>
        <div>Color del fondo</div>
        {#if lowContrast}<div class="small warn">Poco contraste con el texto: prueba un color {isDarkNow() ? "más oscuro" : "más claro"}.</div>{/if}
      </div>
      <div class="ctl">
        {#if a.bgColor}<button class="btn subtle" onclick={() => set({ bgColor: "" })}>Del tema</button>{/if}
        <input
          type="color"
          class="color-input"
          aria-label="Color del fondo"
          value={a.bgColor || (isDarkNow() ? "#202020" : "#f3f3f3")}
          oninput={(e) => set({ bgColor: e.currentTarget.value }, false)}
          onchange={(e) => set({ bgColor: e.currentTarget.value })}
        />
      </div>
    </div>
  {/if}

  {#if a.bg === "image"}
    <div class="row sub-row col">
      <div class="img-head">
        <div class="faint small hint">{IMAGE_HINT}</div>
        <div class="ctl">
          <button class="btn" disabled={busy} onclick={pickImage}>{busy ? "Cargando…" : hasImage ? "Cambiar imagen…" : "Elegir imagen…"}</button>
          {#if hasImage}<button class="icon-btn" title="Quitar imagen" aria-label="Quitar imagen" onclick={removeImage}><Icon name="trash" size={14} /></button>{/if}
        </div>
      </div>
      {#if imageNote}<div class="small warn">{imageNote}</div>{/if}

      <div class="grid">
        <span>Enfoque</span>
        <div class="positions" role="radiogroup" aria-label="Parte de la imagen que se mantiene a la vista">
          {#each IMAGE_POSITIONS as p (p.id)}
            <button class="pos" class:on={a.imagePos === p.id} role="radio" aria-checked={a.imagePos === p.id} title={p.label} aria-label={p.label} onclick={() => set({ imagePos: p.id })}>
              <i></i>
            </button>
          {/each}
        </div>

        <label for="ap-veil">Atenuar</label>
        <div class="slider">
          <input id="ap-veil" type="range" min="0" max="85" step="5" value={a.veil}
            oninput={(e) => set({ veil: +e.currentTarget.value }, false)} onchange={(e) => set({ veil: +e.currentTarget.value })} />
          <span class="val">{a.veil}%</span>
        </div>

        <label for="ap-blur">Desenfoque</label>
        <div class="slider">
          <input id="ap-blur" type="range" min="0" max="20" step="1" value={a.blur}
            oninput={(e) => set({ blur: +e.currentTarget.value }, false)} onchange={(e) => set({ blur: +e.currentTarget.value })} />
          <span class="val">{a.blur}px</span>
        </div>

        <label for="ap-panels">Paneles</label>
        <div class="slider">
          <input id="ap-panels" type="range" min="30" max="100" step="5" value={a.panels}
            oninput={(e) => set({ panels: +e.currentTarget.value }, false)} onchange={(e) => set({ panels: +e.currentTarget.value })} />
          <span class="val">{a.panels}%</span>
        </div>
      </div>
    </div>
  {/if}
</section>

<style>
  .head { display: flex; align-items: center; justify-content: space-between; }
  h3 { font-size: 13px; font-weight: 600; margin: 16px 0 6px; }
  .small-btn { font-size: 12px; padding: 3px 8px; margin-top: 10px; }
  .row {
    display: flex; align-items: center; justify-content: space-between; gap: 16px;
    background: var(--card); border: 1px solid var(--border); padding: 12px 14px; margin-bottom: 3px; border-radius: 6px;
  }
  .row.col { flex-direction: column; align-items: stretch; gap: 10px; }
  .sub-row { margin-left: 14px; }
  .small { font-size: 12px; }
  .warn { color: var(--warn); }
  .ctl { display: flex; align-items: center; gap: 6px; flex: none; }
  .swatches { display: flex; flex-wrap: wrap; gap: 8px; }
  .swatch {
    position: relative; width: 26px; height: 26px; border-radius: 50%; border: none; padding: 0; cursor: default;
    display: grid; place-items: center; color: var(--text-2); font-size: 11px; font-weight: 600;
    outline: 2px solid transparent; outline-offset: 4px;
  }
  /* La selección va con box-shadow: el contorno se reserva para el foco del teclado. */
  .swatch.on { box-shadow: 0 0 0 2px var(--card), 0 0 0 4px var(--text); }
  .swatch:focus-visible { outline-color: var(--accent); }
  .swatch.auto { background: var(--control); border: 1px solid var(--border-strong); }
  .custom { background: var(--control); border: 1px dashed var(--border-strong); overflow: hidden; }
  .custom input { position: absolute; inset: 0; opacity: 0; cursor: default; }
  .color-input { width: 40px; height: 28px; padding: 0 2px; border-radius: 4px; }
  .img-head { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .hint { max-width: 300px; }
  .grid { display: grid; grid-template-columns: 90px 1fr; align-items: center; gap: 10px 12px; }
  .grid > span, .grid > label { font-size: 13px; color: var(--text-2); }
  .positions { display: grid; grid-template-columns: repeat(3, 22px); gap: 3px; }
  .pos {
    width: 22px; height: 16px; border-radius: 3px; border: 1px solid var(--border-strong); background: var(--control);
    display: grid; place-items: center; padding: 0; cursor: default;
  }
  .pos i { width: 5px; height: 5px; border-radius: 50%; background: var(--text-3); }
  .pos.on { background: var(--accent); border-color: transparent; }
  .pos.on i { background: var(--accent-text); }
  .slider { display: flex; align-items: center; gap: 10px; }
  .slider input { flex: 1; accent-color: var(--accent); }
  .val { width: 40px; text-align: right; font-size: 12px; color: var(--text-2); font-variant-numeric: tabular-nums; }
</style>
