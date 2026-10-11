<script lang="ts">
  let {
    checked,
    onchange,
    label = ''
  }: { checked: boolean; onchange: (next: boolean) => void; label?: string } = $props();
</script>

<button
  type="button"
  class="switch"
  class:on={checked}
  role="switch"
  aria-checked={checked}
  aria-label={label}
  onclick={() => onchange(!checked)}
>
  <span class="knob"></span>
</button>

<style>
  .switch {
    width: 40px;
    height: 22px;
    border-radius: var(--r-full);
    border: 1px solid var(--border);
    background: var(--surface-2);
    padding: 2px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    transition: background var(--dur) var(--ease);
  }
  .switch.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .knob {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    transition: transform var(--dur) var(--ease);
  }
  .switch.on .knob {
    transform: translateX(18px);
  }
  .switch:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
