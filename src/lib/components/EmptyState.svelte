<script lang="ts">
  // Estado vacío: icono grande, título y una pista útil.
  import Icon from "./Icon.svelte";

  let {
    icon,
    title,
    hint,
    muted = false,
    compact = false,
  }: { icon: string; title: string; hint: string; muted?: boolean; compact?: boolean } = $props();
</script>

<div class="empty" class:compact role="status">
  <div class="icon" class:muted><Icon name={icon} size={compact ? 28 : 34} /></div>
  <div class="title">{title}</div>
  <div class="faint">{hint}</div>
</div>

<style>
  .empty { display: flex; flex-direction: column; align-items: center; text-align: center; gap: 6px; padding: 56px 24px; }
  .icon {
    display: grid; place-items: center; width: 72px; height: 72px; border-radius: 50%;
    background: var(--accent-soft); color: var(--accent); margin-bottom: 8px;
  }
  .icon.muted { background: var(--hover); color: var(--text-2); }
  .title { font-family: var(--font-display); font-size: 16px; font-weight: 600; }
  .compact { padding: 32px 16px; }
  .compact .icon { width: 56px; height: 56px; }
</style>
