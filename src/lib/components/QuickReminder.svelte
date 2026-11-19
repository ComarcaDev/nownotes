<script lang="ts">
  // Botón de campana con un pequeño menú de recordatorios rápidos.
  import Icon from "./Icon.svelte";
  import { reminderPresets, toDateTimeInput, fromInput } from "../dates";

  let { value, onchange }: { value: number | null; onchange: (v: number | null) => void } = $props();

  let open = $state(false);
  let pos = $state({ x: 0, y: 0 });
  let presets = $state(reminderPresets());

  // Los listeners globales solo existen con el menú abierto (antes había dos por cada fila).
  $effect(() => {
    if (!open) return;
    const close = () => (open = false);
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.stopPropagation(); // Esc cierra solo este menú, no también el editor
      open = false;
    };
    window.addEventListener("mousedown", close);
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("mousedown", close);
      window.removeEventListener("keydown", onKey, true);
    };
  });

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

<button
  class="icon-btn act"
  class:on={value != null}
  title="Recordatorio"
  aria-label="Recordatorio"
  aria-haspopup="menu"
  aria-expanded={open}
  onclick={toggle}
>
  <Icon name="bell" size={15} />
</button>

{#if open}
  <div
    class="popover menu"
    role="menu"
    tabindex="-1"
    aria-label="Recordatorio rápido"
    style:left="{pos.x}px"
    style:top="{pos.y}px"
    onmousedown={(e) => e.stopPropagation()}
    onclick={(e) => e.stopPropagation()}
    onkeydown={() => {}}
  >
    {#each presets as p (p.label)}
      <button class="menu-item" role="menuitem" onclick={() => pick(p.value)}><Icon name="bell" size={13} />{p.label}</button>
    {/each}
    <input
      type="datetime-local"
      aria-label="Fecha y hora del recordatorio"
      value={toDateTimeInput(value)}
      onchange={(e) => pick(fromInput(e.currentTarget.value))}
    />
    {#if value != null}
      <button class="menu-item danger" role="menuitem" onclick={() => pick(null)}><Icon name="x" size={13} />Quitar recordatorio</button>
    {/if}
  </div>
{/if}

<style>
  .act { width: 28px; height: 28px; }
  .menu { width: 200px; display: flex; flex-direction: column; outline: none; }
  input { margin: 4px; font-size: 12px; padding: 4px 6px; }
</style>
