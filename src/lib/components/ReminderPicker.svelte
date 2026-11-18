<script lang="ts">
  import Icon from "./Icon.svelte";
  import { reminderPresets, toDateTimeInput, fromInput, fmt } from "../dates";

  let {
    value,
    onchange,
    compact = false,
  }: { value: number | null; onchange: (v: number | null) => void; compact?: boolean } = $props();

  const presets = reminderPresets();
  let custom = $state(false);
</script>

<div class="picker" class:compact>
  {#each presets as p (p.label)}
    <button class="chip" class:active={value === p.value} onclick={() => onchange(value === p.value ? null : p.value)}>
      {#if !compact}<Icon name="bell" size={12} />{/if}{p.label}
    </button>
  {/each}
  {#if custom || (value != null && !presets.some((p) => p.value === value))}
    <input
      type="datetime-local"
      value={toDateTimeInput(value)}
      onchange={(e) => onchange(fromInput(e.currentTarget.value))}
    />
  {:else}
    <button class="chip" onclick={() => (custom = true)}><Icon name="calendar" size={12} />Elegir…</button>
  {/if}
  {#if value != null}
    <button class="icon-btn small" title="Quitar recordatorio" onclick={() => { custom = false; onchange(null); }}>
      <Icon name="x" size={13} />
    </button>
  {/if}
</div>
{#if value != null && !compact}
  <div class="when faint"><Icon name="bell" size={12} /> {fmt(value)}</div>
{/if}

<style>
  .picker { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
  .picker input { padding: 2px 8px; font-size: 12px; }
  .small { width: 24px; height: 24px; }
  .when { display: flex; align-items: center; gap: 4px; font-size: 12px; margin-top: 6px; }
</style>
