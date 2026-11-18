<script lang="ts">
  // Tema, modo de ventana y anclado: tres iconos discretos.
  import Icon from "./Icon.svelte";
  import { win, isDark, toggleTheme, toggleMode, toggleOnTop } from "../window.svelte";

  const dark = $derived(win.ready && isDark());
</script>

<div class="toolbar" role="toolbar" aria-label="Ventana">
  <button
    class="icon-btn tb"
    title={dark ? "Cambiar a tema claro  (Ctrl+Mayús+L)" : "Cambiar a tema oscuro  (Ctrl+Mayús+L)"}
    aria-label="Cambiar tema"
    onclick={toggleTheme}
  >
    <Icon name={dark ? "sun" : "moon"} size={16} />
  </button>
  <button
    class="icon-btn tb"
    title={win.mode === "mini" ? "Modo completo  (Ctrl+M)" : "Modo mínimo  (Ctrl+M)"}
    aria-label="Cambiar modo de ventana"
    onclick={toggleMode}
  >
    <Icon name={win.mode === "mini" ? "expand" : "minimize"} size={16} />
  </button>
  <button
    class="icon-btn tb"
    class:active={win.onTop}
    title={win.onTop ? "Desanclar ventana  (Ctrl+P)" : "Anclar encima de todas las ventanas  (Ctrl+P)"}
    aria-label="Anclar ventana"
    aria-pressed={win.onTop}
    onclick={toggleOnTop}
  >
    <Icon name={win.onTop ? "anchored" : "anchor"} size={16} />
  </button>
</div>

<style>
  .toolbar { display: inline-flex; align-items: center; gap: 2px; flex: none; }
  .tb { width: 32px; height: 32px; border-radius: 6px; }
  .tb.active { color: var(--accent); background: var(--accent-soft); }
  .tb.active:hover { color: var(--accent); }
</style>
