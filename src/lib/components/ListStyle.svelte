<script lang="ts">
  // Ventanita para elegir el color y el icono de una lista.
  import Icon from "./Icon.svelte";
  import ListMark from "./ListMark.svelte";
  import { api, type List } from "../api";
  import { LIST_COLORS, searchIcons } from "../listIcons";
  import { safe } from "../safe";

  let { list, x, y, onclose }: { list: List; x: number; y: number; onclose: () => void } = $props();

  // svelte-ignore state_referenced_locally (valores iniciales; luego se editan aquí)
  let color = $state(list.color);
  // svelte-ignore state_referenced_locally
  let icon = $state(list.icon ?? "");
  let query = $state("");
  let el = $state<HTMLDivElement>();
  let pos = $state({ x: 0, y: 0 });

  const icons = $derived(searchIcons(query));
  const isCustom = $derived(!LIST_COLORS.includes(color.toLowerCase()));

  $effect(() => {
    const w = el?.offsetWidth ?? 280;
    const h = el?.offsetHeight ?? 360;
    pos = {
      x: Math.max(8, Math.min(x, window.innerWidth - w - 8)),
      y: Math.max(8, Math.min(y, window.innerHeight - h - 8)),
    };
  });

  $effect(() => {
    el?.querySelector<HTMLInputElement>("input[type=search]")?.focus();
  });

  async function save(next: { color?: string; icon?: string }) {
    color = next.color ?? color;
    icon = next.icon ?? icon;
    await safe(() => api.saveList(list.id, list.name, color, icon), "No se pudo cambiar el estilo de la lista");
  }
</script>

<svelte:window onmousedown={onclose} onkeydown={(e) => e.key === "Escape" && onclose()} onresize={onclose} />

<div
  class="popover style"
  role="dialog"
  aria-label={`Color e icono de ${list.name}`}
  tabindex="-1"
  bind:this={el}
  style:left="{pos.x}px"
  style:top="{pos.y}px"
  onmousedown={(e) => e.stopPropagation()}
>
  <header>
    <ListMark list={{ id: list.id, color, icon }} size={18} />
    <span class="name">{list.name}</span>
  </header>

  <div class="label">Color</div>
  <div class="colors" role="radiogroup" aria-label="Color">
    {#each LIST_COLORS as c (c)}
      <button
        class="swatch"
        class:on={color.toLowerCase() === c}
        style:background={c}
        role="radio"
        aria-checked={color.toLowerCase() === c}
        aria-label={`Color ${c}`}
        title={c}
        onclick={() => save({ color: c })}
      ></button>
    {/each}
    <label class="swatch custom" class:on={isCustom} title="Color personalizado" style:background={isCustom ? color : undefined}>
      {#if !isCustom}<Icon name="plus" size={12} />{/if}
      <input type="color" aria-label="Color personalizado" value={color} onchange={(e) => save({ color: e.currentTarget.value })} />
    </label>
  </div>

  <div class="label">Icono</div>
  <div class="search">
    <Icon name="search" size={13} />
    <input type="search" placeholder="Buscar: cocina, estudio, viaje…" aria-label="Buscar icono" bind:value={query} />
  </div>
  <div class="icons" role="radiogroup" aria-label="Icono">
    {#if !query}
      <button class="ico" class:on={!icon} role="radio" aria-checked={!icon} title="Sin icono (punto de color)" aria-label="Sin icono" onclick={() => save({ icon: "" })}>
        <span class="dot" style:background={color}></span>
      </button>
    {/if}
    {#each icons as i (i.id)}
      <button
        class="ico"
        class:on={icon === i.id}
        style:color={color}
        role="radio"
        aria-checked={icon === i.id}
        title={i.label}
        aria-label={i.label}
        onclick={() => save({ icon: i.id })}
      >
        <Icon name={i.id} size={17} />
      </button>
    {:else}
      <div class="none faint small">Ningún icono coincide.</div>
    {/each}
  </div>
</div>

<style>
  .style { width: 272px; padding: 12px; display: flex; flex-direction: column; gap: 8px; }
  header { display: flex; align-items: center; gap: 10px; padding-bottom: 4px; }
  .name { font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .label { font-size: 12px; font-weight: 600; color: var(--text-2); margin-top: 2px; }
  .colors { display: grid; grid-template-columns: repeat(7, 1fr); gap: 6px; }
  .swatch {
    position: relative; width: 28px; height: 28px; border-radius: 50%; border: none; padding: 0;
    display: grid; place-items: center; color: var(--text-2); cursor: default;
    outline: 2px solid transparent; outline-offset: 4px;
  }
  /* La selección va con box-shadow: el contorno se reserva para el foco del teclado. */
  .swatch.on { box-shadow: 0 0 0 2px var(--solid), 0 0 0 4px var(--text); }
  .swatch:focus-visible { outline-color: var(--accent); }
  .custom { background: var(--control); border: 1px dashed var(--border-strong); overflow: hidden; }
  .custom input { position: absolute; inset: 0; opacity: 0; cursor: default; }
  .search {
    display: flex; align-items: center; gap: 6px; padding: 0 8px; color: var(--text-2);
    background: var(--control); border: 1px solid var(--border); border-radius: var(--radius-sm);
  }
  .search input { border: none; background: transparent; flex: 1; padding: 5px 2px; font-size: 12.5px; }
  .search input:focus { border: none; padding-bottom: 5px; }
  .icons { display: grid; grid-template-columns: repeat(7, 1fr); gap: 4px; max-height: 176px; overflow-y: auto; }
  .ico {
    height: 32px; border: none; border-radius: 6px; background: transparent; display: grid; place-items: center;
    cursor: default; outline: 2px solid transparent;
  }
  .ico:hover { background: var(--hover); }
  .ico.on { background: var(--selected); box-shadow: inset 0 0 0 1.5px var(--border-strong); }
  .ico:focus-visible { outline-color: var(--accent); }
  .dot { width: 10px; height: 10px; border-radius: 50%; }
  .none { grid-column: 1 / -1; padding: 8px 2px; }
</style>
