<script lang="ts">
  let {
    onclick,
    variant = 'ghost',
    disabled = false,
    type = 'button',
    children
  }: {
    onclick?: () => void;
    variant?: 'primary' | 'ghost' | 'danger';
    disabled?: boolean;
    type?: 'button' | 'submit';
    children?: import('svelte').Snippet;
  } = $props();
</script>

<button {type} class="btn {variant}" {disabled} onclick={onclick}>
  {@render children?.()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 7px 14px;
    border-radius: var(--r-sm);
    border: 1px solid var(--border);
    background: var(--surface-1);
    color: var(--text-hi);
    cursor: pointer;
    font-weight: 600;
    transition: background var(--dur-fast) var(--ease);
  }
  .btn:hover:not(:disabled) {
    background: var(--surface-2);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .primary:hover:not(:disabled) {
    filter: brightness(1.08);
    background: var(--accent);
  }
  .danger {
    background: #c62828;
    border-color: #c62828;
    color: #fff;
  }
  .danger:hover:not(:disabled) {
    filter: brightness(1.08);
    background: #c62828;
  }
</style>
