<script lang="ts">
  import { fly } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import QuickReminder from "./QuickReminder.svelte";
  import type { Item, List } from "../api";
  import type { ItemActions } from "../actions";
  import { fmt, dayLabel } from "../dates";
  import { renderMarkdown } from "../markdown";

  let {
    item,
    list,
    showList,
    selected,
    actions,
  }: { item: Item; list: List | undefined; showList: boolean; selected: boolean; actions: ItemActions } = $props();

  // Vista previa: primeras líneas renderizadas, sin enlaces activos.
  const preview = $derived(
    renderMarkdown(item.body.split(/\r?\n/).slice(0, 8).join("\n")).replace(/<a [^>]*>(.*?)<\/a>/g, "$1")
  );
</script>

<div
  class="card"
  class:selected
  role="button"
  tabindex="-1"
  onclick={() => actions.select(item)}
  oncontextmenu={(e) => actions.menu(e, item)}
  onkeydown={() => {}}
  in:fly={{ y: -6, duration: 160 }}
>
  {#if showList && list && list.id !== 1}<i class="stripe" style:background={list.color}></i>{/if}
  <div class="head">
    <div class="title">{item.title || "Sin título"}</div>
    {#if item.pinned}<span class="pin"><Icon name="pinned" size={13} /></span>{/if}
  </div>
  {#if item.body.trim()}
    <div class="preview markdown">{@html preview}</div>
  {/if}
  <div class="foot">
    {#if item.remindAt != null}
      <span class="chip-s"><Icon name="bell" size={11} /> {fmt(item.snoozedUntil ?? item.remindAt)}</span>
    {/if}
    {#each item.tags as t (t)}<span class="tag">#{t}</span>{/each}
    <span class="grow"></span>
    <span class="date faint">{dayLabel(item.updatedAt)}</span>
  </div>
  <div class="actions">
    <button class="icon-btn act" class:on={item.pinned} title={item.pinned ? "Desfijar" : "Fijar"} onclick={(e) => { e.stopPropagation(); actions.pin(item); }}>
      <Icon name={item.pinned ? "pinned" : "pin"} size={14} />
    </button>
    <QuickReminder value={item.remindAt} onchange={(v) => actions.setReminder(item, v)} />
    <button class="icon-btn act" title="Mover a la papelera" onclick={(e) => { e.stopPropagation(); actions.remove(item); }}><Icon name="trash" size={14} /></button>
  </div>
</div>

<style>
  .card {
    position: relative; display: flex; flex-direction: column; gap: 6px; padding: 12px 14px 10px;
    min-height: 110px; max-height: 260px; overflow: hidden; cursor: default;
    background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
    transition: box-shadow 0.12s, border-color 0.12s;
  }
  .card:hover { box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1); }
  .card.selected { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent) inset; }
  .stripe { position: absolute; left: 0; top: 0; bottom: 0; width: 3px; }
  .head { display: flex; align-items: flex-start; gap: 6px; }
  .title { flex: 1; font-family: var(--font-display); font-weight: 600; font-size: 14.5px; line-height: 1.3; overflow-wrap: anywhere; }
  .pin { color: var(--accent); }
  .preview {
    flex: 1; font-size: 12.5px; color: var(--text-2); overflow: hidden; user-select: none;
    mask-image: linear-gradient(to bottom, black 70%, transparent);
  }
  .preview :global(p), .preview :global(ul) { margin-bottom: 4px; }
  .foot { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; font-size: 11.5px; }
  .chip-s { display: inline-flex; align-items: center; gap: 3px; padding: 1px 7px; border-radius: 999px; background: var(--accent-soft); color: var(--accent); }
  .tag { color: var(--accent); }
  .grow { flex: 1; }
  .actions {
    position: absolute; top: 6px; right: 6px; display: flex; opacity: 0; transition: opacity 0.1s;
    background: var(--card); border-radius: var(--radius-sm);
  }
  .card:hover .actions { opacity: 1; }
  .act { width: 26px; height: 26px; }
</style>
