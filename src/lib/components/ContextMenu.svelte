<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { Item, List } from "../api";
  import type { MenuAction } from "../actions";

  let {
    x,
    y,
    item,
    lists,
    onaction,
    onclose,
  }: {
    x: number;
    y: number;
    item: Item;
    lists: List[];
    onaction: (a: MenuAction) => void;
    onclose: () => void;
  } = $props();

  let el = $state<HTMLDivElement>();
  let moving = $state(false);
  let pos = $state({ x: 0, y: 0 });

  // Ajuste para no salirse de la ventana.
  $effect(() => {
    void moving;
    const w = el?.offsetWidth ?? 220;
    const h = el?.offsetHeight ?? 240;
    pos = { x: Math.min(x, window.innerWidth - w - 8), y: Math.min(y, window.innerHeight - h - 8) };
  });

  function act(a: MenuAction) {
    onaction(a);
    onclose();
  }
</script>

<svelte:window onmousedown={onclose} onkeydown={(e) => e.key === "Escape" && onclose()} onblur={onclose} onresize={onclose} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="popover menu" bind:this={el} style:left="{pos.x}px" style:top="{pos.y}px" onmousedown={(e) => e.stopPropagation()} oncontextmenu={(e) => e.preventDefault()}>
  {#if moving}
    <button class="mi" onclick={() => (moving = false)}><span class="back">‹</span>Mover a…</button>
    <div class="sep"></div>
    <div class="scroll">
      {#each lists as l (l.id)}
        <button class="mi" class:current={l.id === item.listId} onclick={() => act({ move: l.id })}>
          {#if l.id === 1}<Icon name="inbox" size={14} />{:else}<i class="dot" style:background={l.color}></i>{/if}
          {l.name}
        </button>
      {/each}
    </div>
  {:else}
    <button class="mi" onclick={() => act("open")}><Icon name="edit" size={14} />Abrir</button>
    {#if item.kind === "task"}
      <button class="mi" onclick={() => act("toggle")}>
        <Icon name="done" size={14} />{item.done ? "Marcar como pendiente" : "Completar"}
      </button>
    {/if}
    <button class="mi" onclick={() => act("pin")}><Icon name={item.pinned ? "pin" : "pinned"} size={14} />{item.pinned ? "Desfijar" : "Fijar arriba"}</button>
    <button class="mi" onclick={() => (moving = true)}><Icon name="list" size={14} />Mover a lista<span class="grow"></span>›</button>
    <button class="mi" onclick={() => act("convert")}>
      <Icon name={item.kind === "task" ? "note" : "done"} size={14} />Convertir en {item.kind === "task" ? "nota" : "tarea"}
    </button>
    <div class="sep"></div>
    <button class="mi danger" onclick={() => act("remove")}><Icon name="trash" size={14} />Mover a la papelera<span class="grow"></span><kbd>Supr</kbd></button>
  {/if}
</div>

<style>
  .menu { width: 230px; display: flex; flex-direction: column; }
  .mi {
    display: flex; align-items: center; gap: 10px; border: none; background: transparent;
    padding: 7px 10px; border-radius: var(--radius-sm); text-align: left; cursor: default;
  }
  .mi:hover { background: var(--hover); }
  .mi.current { color: var(--accent); }
  .mi.danger { color: var(--danger); }
  .grow { flex: 1; }
  .sep { height: 1px; background: var(--border); margin: 4px 0; }
  .scroll { max-height: 240px; overflow-y: auto; display: flex; flex-direction: column; }
  .dot { width: 9px; height: 9px; border-radius: 50%; margin: 0 2px; }
  .back { width: 14px; text-align: center; font-size: 16px; line-height: 1; }
</style>
