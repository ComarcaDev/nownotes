<script lang="ts">
  // Tema, modo de ventana y anclado: tres iconos discretos.
  import Icon from "./Icon.svelte";
  import { win, isDark, toggleTheme, toggleMode, toggleOnTop } from "../window.svelte";
  import { keys, display } from "../keys.svelte";

  const dark = $derived(win.ready && isDark());
  const k = (combo: string) => (combo ? `  (${display(combo)})` : "");
</script>

<div class="toolbar" role="toolbar" aria-label="Ventana">
  <button
    class="icon-btn tb"
    title={(dark ? "Cambiar a tema claro" : "Cambiar a tema oscuro") + k(keys.app.theme)}
    aria-label="Cambiar tema"
    onclick={toggleTheme}
  >
    <Icon name={dark ? "sun" : "moon"} size={16} />
  </button>
  <button
    class="icon-btn tb"
    title={(win.mode === "mini" ? "Modo completo" : "Modo mínimo") + k(keys.app.mode)}
    aria-label="Cambiar modo de ventana"
    onclick={toggleMode}
  >
    <Icon name={win.mode === "mini" ? "expand" : "minimize"} size={16} />
  </button>
  <button
    class="icon-btn tb"
    class:active={win.onTop}
    title={(win.onTop ? "Desanclar ventana" : "Anclar encima de todas las ventanas") + k(keys.app.pin)}
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
