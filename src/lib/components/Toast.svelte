<script lang="ts">
  import { fly } from "svelte/transition";
  import { toast, dismissToast, holdToast, releaseToast } from "../toast.svelte";
  import Icon from "./Icon.svelte";
</script>

{#if toast.current}
  {#key toast.current.id}
    <div class="toast" role="status" transition:fly={{ y: 16, duration: 160 }} onmouseenter={holdToast} onmouseleave={releaseToast}>
      <span class="text">{toast.current.text}</span>
      {#if toast.current.action}
        {@const action = toast.current.action}
        <button class="undo" onclick={() => { action.run(); dismissToast(); }}>{action.label}</button>
      {/if}
      <button class="icon-btn close" title="Cerrar" onclick={dismissToast}><Icon name="x" size={13} /></button>
    </div>
  {/key}
{/if}

<style>
  .toast {
    position: fixed; left: 50%; bottom: 20px; translate: -50% 0; z-index: 30;
    display: flex; align-items: center; gap: 12px; padding: 6px 6px 6px 16px;
    background: var(--solid); border-radius: var(--radius); box-shadow: var(--shadow);
    max-width: calc(100vw - 32px);
  }
  .text { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .undo {
    border: none; background: transparent; color: var(--accent); font-weight: 600;
    padding: 5px 8px; border-radius: var(--radius-sm); cursor: default;
  }
  .undo:hover { background: var(--hover); }
  .close { width: 26px; height: 26px; }
</style>
