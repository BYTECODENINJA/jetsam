<script lang="ts">
  let {
    tone = 'info',
    title,
    children
  }: {
    tone?: 'info' | 'caution' | 'risk' | 'safe';
    title: string;
    children?: import('svelte').Snippet;
  } = $props();
</script>

<div class="banner {tone}" role="status">
  <strong>{title}</strong>
  <div class="detail">{@render children?.()}</div>
</div>

<style>
  .banner {
    border: 1px solid var(--border);
    border-left-width: 4px;
    border-radius: var(--r-sm);
    padding: var(--sp-3) var(--sp-4);
    margin-bottom: var(--sp-4);
    background: var(--surface-1);
  }
  .detail {
    margin-top: 2px;
    color: var(--text-lo);
    font-size: var(--fs-sm);
  }
  .info {
    border-left-color: #1565c0;
  }
  .caution {
    border-left-color: #b26a00;
  }
  .risk {
    border-left-color: #c62828;
  }
  .safe {
    border-left-color: #2e7d32;
  }
</style>
