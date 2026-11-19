<script lang="ts">
  // Marca visual de una lista: bandeja, icono con el color de la lista, o punto de color.
  import Icon from "./Icon.svelte";
  import { INBOX_ID } from "../items";
  import { LIST_ICON_PATHS } from "../listIcons";

  let { list, size = 16 }: { list: { id: number; color: string; icon?: string }; size?: number } = $props();
</script>

{#if list.id === INBOX_ID}
  <Icon name="inbox" {size} />
{:else if list.icon && LIST_ICON_PATHS[list.icon]}
  <span class="icon" style:color={list.color}><Icon name={list.icon} {size} /></span>
{:else}
  <span class="dot" style:background={list.color} style:--d={`${Math.round(size * 0.62)}px`}></span>
{/if}

<style>
  .icon { display: inline-flex; flex: none; }
  .dot { width: var(--d); height: var(--d); border-radius: 50%; flex: none; display: inline-block; }
</style>
