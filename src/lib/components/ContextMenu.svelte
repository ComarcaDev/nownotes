<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { Item, List } from "../api";
  import type { MenuAction } from "../actions";
  import { INBOX_ID } from "../items";

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

  // Foco en el primer elemento para poder usarlo con el teclado.
  $effect(() => {
    void moving;
    el?.querySelector<HTMLButtonElement>("button")?.focus();
  });

  function act(a: MenuAction) {
    onaction(a);
    onclose();
  }
</script>

<svelte:window onmousedown={onclose} onkeydown={(e) => e.key === "Escape" && onclose()} onblur={onclose} onresize={onclose} />

<div
  class="popover menu"
  role="menu"
  tabindex="-1"
  aria-label="Acciones"
  bind:this={el}
  style:left="{pos.x}px"
  style:top="{pos.y}px"
  onmousedown={(e) => e.stopPropagation()}
  oncontextmenu={(e) => e.preventDefault()}
>
  {#if moving}
    <button class="menu-item" role="menuitem" onclick={() => (moving = false)}><span class="back">‹</span>Mover a…</button>
    <div class="sep" role="separator"></div>
    <div class="scroll">
      {#each lists as l (l.id)}
        <button class="menu-item" role="menuitem" class:current={l.id === item.listId} onclick={() => act({ move: l.id })}>
          {#if l.id === INBOX_ID}<Icon name="inbox" size={14} />{:else}<i class="dot" style:background={l.color}></i>{/if}
          {l.name}
        </button>
      {/each}
    </div>
  {:else}
    <button class="menu-item" role="menuitem" onclick={() => act("open")}><Icon name="edit" size={14} />Abrir</button>
    {#if item.kind === "task"}
      <button class="menu-item" role="menuitem" onclick={() => act("toggle")}>
        <Icon name="done" size={14} />{item.done ? "Marcar como pendiente" : "Completar"}
      </button>
    {/if}
    <button class="menu-item" role="menuitem" onclick={() => act("pin")}>
      <Icon name={item.pinned ? "pin" : "pinned"} size={14} />{item.pinned ? "Desfijar" : "Fijar arriba"}
    </button>
    <button class="menu-item" role="menuitem" onclick={() => (moving = true)}>
      <Icon name="list" size={14} />Mover a lista<span class="grow"></span>›
    </button>
    <button class="menu-item" role="menuitem" onclick={() => act("convert")}>
      <Icon name={item.kind === "task" ? "note" : "done"} size={14} />Convertir en {item.kind === "task" ? "nota" : "tarea"}
    </button>
    <div class="sep" role="separator"></div>
    <button class="menu-item danger" role="menuitem" onclick={() => act("remove")}>
      <Icon name="trash" size={14} />Mover a la papelera<span class="grow"></span><kbd>Supr</kbd>
    </button>
  {/if}
</div>

<style>
  .menu { width: 230px; display: flex; flex-direction: column; outline: none; }
  .sep { height: 1px; background: var(--border); margin: 4px 0; }
  .scroll { max-height: 240px; overflow-y: auto; display: flex; flex-direction: column; }
  .dot { width: 9px; height: 9px; border-radius: 50%; margin: 0 2px; }
  .back { width: 14px; text-align: center; font-size: 16px; line-height: 1; }
</style>
