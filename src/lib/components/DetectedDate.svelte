<script lang="ts">
  // Chip con la fecha detectada en el texto («mañana 18:00») y botón para descartarla.
  import Icon from "./Icon.svelte";
  import type { Parsed } from "../nlp";
  import { dayLabel, fmt } from "../dates";

  let { parsed, ondismiss }: { parsed: Parsed; ondismiss: () => void } = $props();

  const label = $derived(parsed.remindAt ? fmt(parsed.remindAt) : dayLabel(parsed.dueAt));
</script>

<span class="detected" title={`Detectado: «${parsed.matches.join(" ")}»`}>
  <Icon name={parsed.remindAt ? "bell" : "calendar"} size={12} />
  {label}
  <button class="undo" title="No usar esta fecha" aria-label="Descartar fecha detectada" onclick={ondismiss}>
    <Icon name="x" size={11} />
  </button>
</span>

<style>
  .detected {
    display: inline-flex; align-items: center; gap: 5px; padding: 2px 4px 2px 9px; border-radius: 999px;
    font-size: 12px; background: var(--accent); color: var(--accent-text);
  }
  .undo {
    display: grid; place-items: center; width: 18px; height: 18px; border: none; border-radius: 50%;
    background: transparent; color: inherit; cursor: default; padding: 0;
  }
  .undo:hover { background: color-mix(in srgb, var(--accent-text) 20%, transparent); }
</style>
