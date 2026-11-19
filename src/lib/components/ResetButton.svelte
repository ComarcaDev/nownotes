<script lang="ts">
  // ↺ pequeño junto a una opción: solo se muestra si la opción no tiene su valor predeterminado.
  import { fade } from "svelte/transition";
  import Icon from "./Icon.svelte";

  let { show, label, onclick }: { show: boolean; label: string; onclick: () => void } = $props();
</script>

{#if show}
  <button
    class="reset"
    title={`Restablecer ${label}`}
    aria-label={`Restablecer ${label}`}
    transition:fade={{ duration: 120 }}
    onclick={(e) => { e.preventDefault(); e.stopPropagation(); onclick(); }}
  >
    <Icon name="restore" size={12} />
  </button>
{/if}

<style>
  .reset {
    display: inline-grid; place-items: center; width: 20px; height: 20px; margin-left: 4px; vertical-align: middle;
    border: none; border-radius: 4px; background: transparent; color: var(--text-3); cursor: default; flex: none;
  }
  .reset:hover { background: var(--hover); color: var(--accent); }
</style>
