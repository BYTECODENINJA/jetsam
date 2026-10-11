<script lang="ts">
  let {
    title,
    message = '',
    children
  }: {
    title: string;
    message?: string;
    children?: import('svelte').Snippet;
  } = $props();
</script>

<div class="empty">
  <h2>{title}</h2>
  {#if message}<p class="muted">{message}</p>{/if}
  {@render children?.()}
</div>

<style>
  .empty {
    text-align: center;
    padding: var(--sp-7) var(--sp-4);
    color: var(--text-lo);
  }
  .empty h2 {
    color: var(--text-hi);
    margin-bottom: var(--sp-2);
  }
</style>
