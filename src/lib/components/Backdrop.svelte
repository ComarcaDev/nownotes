<script lang="ts">
  // Imagen de fondo de la ventana principal, con velo y desenfoque.
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { api } from "../api";
  import { parseAppearance, PREVIEW_EVENT, type Appearance } from "../appearance";

  let appearance = $state<Appearance>(parseAppearance(undefined));
  let url = $state("");
  let loadedRev = -1;

  /** Carga (o recarga) la imagen guardada en la carpeta de datos. */
  async function load(rev: number) {
    loadedRev = rev;
    let next = "";
    try {
      const path = await api.backgroundPath();
      if (path) next = URL.createObjectURL(new Blob([await api.readFileBytes(path)]));
    } catch (e) {
      console.warn("No se pudo cargar la imagen de fondo", e);
    }
    if (url) URL.revokeObjectURL(url);
    url = next;
  }

  function update(a: Appearance) {
    appearance = a;
    if (a.bg === "image" && a.imageRev !== loadedRev) load(a.imageRev);
  }

  // Vista previa en vivo desde Ajustes, antes de guardar (ver theme.ts).
  const onPreview = (e: Event) => update((e as CustomEvent<Appearance>).detail);

  let unlisten: (() => void) | undefined;
  onMount(async () => {
    window.addEventListener(PREVIEW_EVENT, onPreview);
    try {
      update(parseAppearance((await api.settings()).appearance));
    } catch {
      /* sin backend */
    }
    unlisten = await listen<[string, string]>("settings-changed", (e) => {
      if (e.payload[0] === "appearance") update(parseAppearance(e.payload[1]));
    });
  });
  onDestroy(() => {
    window.removeEventListener(PREVIEW_EVENT, onPreview);
    unlisten?.();
    if (url) URL.revokeObjectURL(url);
  });
</script>

{#if appearance.bg === "image" && url}
  <div class="backdrop" aria-hidden="true">
    <div class="img" style:background-image="url({url})"></div>
    <div class="veil"></div>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; z-index: -1; overflow: hidden; pointer-events: none; }
  .img {
    position: absolute;
    /* Sobra un margen para que el desenfoque no deje bordes claros. */
    inset: calc(var(--bg-blur, 0px) * -2);
    background-size: cover;
    background-position: var(--bg-pos, center);
    background-repeat: no-repeat;
    filter: blur(var(--bg-blur, 0px));
  }
  .veil { position: absolute; inset: 0; background: var(--solid-0); opacity: var(--veil, 0.4); }
</style>
