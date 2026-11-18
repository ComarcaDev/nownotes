<script lang="ts">
  // Botón de campana con un pequeño menú de recordatorios rápidos.
  import Icon from "./Icon.svelte";
  import { reminderPresets, toDateTimeInput, fromInput } from "../dates";

  let { value, onchange }: { value: number | null; onchange: (v: number | null) => void } = $props();

  let open = $state(false);
  let pos = $state({ x: 0, y: 0 });
  let presets = $state(reminderPresets());

  function toggle(e: MouseEvent) {
    e.stopPropagation();
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    pos = { x: Math.min(r.right - 200, window.innerWidth - 216), y: Math.min(r.bottom + 4, window.innerHeight - 230) };
    presets = reminderPresets();
    open = !open;
  }

  function pick(v: number | null) {
    open = false;
    onchange(v);
  }
</script>

<svelte:window onmousedown={() => (open = false)} onkeydown={(e) => e.key === "Escape" && (open = false)} />

<button class="icon-btn act" class:on={value != null} title="Recordatorio" onclick={toggle}>
  <Icon name="bell" size={15} />
</button>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
  <div class="popover menu" style:left="{pos.x}px" style:top="{pos.y}px" onmousedown={(e) => e.stopPropagation()} onclick={(e) => e.stopPropagation()}>
    {#each presets as p (p.label)}
      <button class="mi" onclick={() => pick(p.value)}><Icon name="bell" size={13} />{p.label}</button>
    {/each}
    <input type="datetime-local" value={toDateTimeInput(value)} onchange={(e) => pick(fromInput(e.currentTarget.value))} />
    {#if value != null}
      <button class="mi danger" onclick={() => pick(null)}><Icon name="x" size={13} />Quitar recordatorio</button>
    {/if}
  </div>
{/if}

<style>
  .act { width: 28px; height: 28px; }
  .menu { width: 200px; display: flex; flex-direction: column; }
  .mi {
    display: flex; align-items: center; gap: 10px; border: none; background: transparent;
    padding: 7px 10px; border-radius: var(--radius-sm); text-align: left; cursor: default;
  }
  .mi:hover { background: var(--hover); }
  .mi.danger { color: var(--danger); }
  input { margin: 4px; font-size: 12px; padding: 4px 6px; }
</style>
