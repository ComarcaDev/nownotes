<script lang="ts">
  import { fly } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import QuickReminder from "./QuickReminder.svelte";
  import type { Item, List } from "../api";
  import type { ItemActions } from "../actions";
  import { fmt, isOverdue, RECUR_LABEL, startOfDay } from "../dates";

  let {
    item,
    list,
    showList,
    selected,
    actions,
    hideToday = false,
    draggable = false,
  }: {
    item: Item; list: List | undefined; showList: boolean; selected: boolean; actions: ItemActions;
    hideToday?: boolean; draggable?: boolean;
  } = $props();

  // En la vista «Hoy» no se repite la etiqueta «Hoy» en cada fila.
  const showDue = $derived(
    item.dueAt != null && !(hideToday && startOfDay(item.dueAt).getTime() === startOfDay().getTime())
  );

  let leaving = $state(false);

  const dueOverdue = $derived(!item.done && item.dueAt != null && item.dueAt < startOfDay().getTime());
  const remindOverdue = $derived(!item.done && item.recur === "none" && isOverdue(item.snoozedUntil ?? item.remindAt));

  function check(e: MouseEvent) {
    e.stopPropagation();
    if (item.done) { actions.toggle(item); return; }
    leaving = true; // animación y luego se completa
    setTimeout(() => actions.toggle(item), 280);
  }
</script>

<div
  class="row"
  class:selected
  class:done={item.done}
  class:leaving
  role="button"
  tabindex="-1"
  data-drag-id={item.id}
  onclick={() => actions.select(item)}
  oncontextmenu={(e) => actions.menu(e, item)}
  onkeydown={() => {}}
  in:fly={{ y: -6, duration: 160 }}
>
  {#if draggable}
    <span class="grip" data-drag-handle title="Arrastrar para ordenar" role="presentation" onclick={(e) => e.stopPropagation()}>
      <Icon name="grip" size={14} />
    </span>
  {/if}
  {#if item.kind === "note"}
    <span class="note-icon"><Icon name="note" size={17} /></span>
  {:else}
    <button class="check" class:checked={item.done || leaving} title={item.done ? "Marcar como pendiente" : "Completar"} onclick={check}>
      {#if item.done || leaving}<Icon name="done" size={12} />{/if}
    </button>
  {/if}

  <div class="main">
    <div class="title">{item.title || "Sin título"}</div>
    <div class="meta">
      {#if showDue && item.dueAt != null}
        <span class:overdue={dueOverdue}><Icon name="calendar" size={12} /> {fmt(item.dueAt, false)}</span>
      {/if}
      {#if item.remindAt != null}
        <span class:overdue={remindOverdue}><Icon name="bell" size={12} /> {fmt(item.snoozedUntil ?? item.remindAt)}</span>
      {/if}
      {#if item.recur !== "none"}
        <span><Icon name="repeat" size={12} /> {RECUR_LABEL[item.recur]}</span>
      {/if}
      {#if item.body.trim()}<span title="Tiene notas"><Icon name="note" size={12} /></span>{/if}
      {#if showList && list && list.id !== 1}
        <span><i class="dot" style:background={list.color}></i>{list.name}</span>
      {/if}
      {#each item.tags as t (t)}<span class="tag">#{t}</span>{/each}
    </div>
  </div>

  {#if !item.done}
    <div class="actions">
      <button class="icon-btn act" class:on={item.pinned} title={item.pinned ? "Desfijar" : "Fijar arriba"} onclick={(e) => { e.stopPropagation(); actions.pin(item); }}>
        <Icon name={item.pinned ? "pinned" : "pin"} size={15} />
      </button>
      <QuickReminder value={item.remindAt} onchange={(v) => actions.setReminder(item, v)} />
      <button class="icon-btn act" title="Mover a la papelera (Supr)" onclick={(e) => { e.stopPropagation(); actions.remove(item); }}>
        <Icon name="trash" size={15} />
      </button>
    </div>
  {:else}
    <div class="actions">
      <button class="icon-btn act" title="Mover a la papelera" onclick={(e) => { e.stopPropagation(); actions.remove(item); }}><Icon name="trash" size={15} /></button>
    </div>
  {/if}
  {#if item.pinned}<span class="pin"><Icon name="pinned" size={13} /></span>{/if}
</div>

<style>
  .row {
    position: relative; display: flex; align-items: center; gap: 12px; padding: 9px 10px 9px 12px; margin: 0 0 3px;
    border-radius: 6px; background: var(--card); border: 1px solid var(--border); cursor: default; min-height: 46px;
  }
  .row:hover { background: var(--control-hover); }
  .row.selected { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent) inset; }
  .row.leaving { animation: fade-out 0.28s ease-in forwards; pointer-events: none; }
  .check {
    flex: none; width: 20px; height: 20px; border-radius: 50%;
    border: 1.5px solid var(--text-2); background: transparent; display: grid; place-items: center;
    color: var(--accent-text); cursor: default; padding: 0; transition: background 0.12s, border-color 0.12s;
  }
  .check:hover { border-color: var(--accent); background: var(--accent-soft); }
  .check.checked { background: var(--accent); border-color: var(--accent); animation: pop 0.25s ease-out; }
  .grip {
    position: absolute; left: -2px; top: 50%; translate: 0 -50%; display: grid; place-items: center;
    width: 14px; height: 28px; color: var(--text-3); cursor: grab; opacity: 0; touch-action: none;
  }
  .row:hover .grip { opacity: 1; }
  .note-icon { flex: none; display: grid; place-items: center; width: 20px; color: var(--accent); }
  .main { flex: 1; min-width: 0; }
  .title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .done .title { text-decoration: line-through; color: var(--text-3); }
  .meta { display: flex; flex-wrap: wrap; gap: 3px 12px; font-size: 12px; color: var(--text-2); margin-top: 2px; }
  .meta:empty { display: none; }
  .meta span { display: inline-flex; align-items: center; gap: 4px; }
  .meta .overdue { color: var(--danger); }
  .dot { display: inline-block; width: 7px; height: 7px; border-radius: 50%; }
  .tag { color: var(--accent); }
  .actions { display: flex; gap: 0; opacity: 0; transition: opacity 0.1s; }
  .row:hover .actions, .row.selected .actions { opacity: 1; }
  .act { width: 28px; height: 28px; }
  .pin { position: absolute; top: 4px; right: 6px; color: var(--accent); }
  .row:hover .pin { display: none; }
</style>
