<script lang="ts">
  let {
    value,
    label = ''
  }: { value: number; label?: string } = $props();
  const tone = $derived(value >= 66 ? 'risk' : value >= 33 ? 'caution' : 'safe');
</script>

<div class="scorebar" role="img" aria-label="{label} score {value} of 100">
  <div class="track">
    <div class="fill {tone}" style="width: {Math.max(0, Math.min(100, value))}%"></div>
  </div>
  <span class="num tnum">{value}</span>
</div>

<style>
  .scorebar {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    min-width: 96px;
  }
  .track {
    flex: 1;
    height: 6px;
    border-radius: var(--r-full);
    background: var(--surface-2);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: var(--r-full);
  }
  .safe {
    background: #2e7d32;
  }
  .caution {
    background: #b26a00;
  }
  .risk {
    background: #c62828;
  }
  .num {
    font-size: var(--fs-sm);
    color: var(--text-lo);
    min-width: 22px;
    text-align: right;
  }
</style>
