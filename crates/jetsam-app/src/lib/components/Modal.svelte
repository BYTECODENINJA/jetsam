<script lang="ts">
  let {
    open,
    title,
    onclose,
    children
  }: {
    open: boolean;
    title: string;
    onclose: () => void;
    children?: import('svelte').Snippet;
  } = $props();

  function onkeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') onclose();
  }
</script>

<svelte:window onkeydown={open ? onkeydown : undefined} />

{#if open}
  <div
    class="backdrop"
    role="presentation"
    onclick={onclose}
  >
    <div
      class="modal card"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <div class="spread">
        <h2>{title}</h2>
        <button class="x" type="button" aria-label="Close" onclick={onclose}>✕</button>
      </div>
      <div class="body">
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--sp-5);
    z-index: 100;
  }
  .modal {
    background: var(--surface-0);
    max-width: 640px;
    width: 100%;
    max-height: 85vh;
    overflow: auto;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.28);
  }
  .body {
    margin-top: var(--sp-3);
  }
  .x {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 16px;
    color: var(--text-lo);
  }
</style>
