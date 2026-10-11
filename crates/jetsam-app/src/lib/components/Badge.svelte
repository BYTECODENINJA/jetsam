<script lang="ts">
  let { text, tone = 'neutral' }: { text: string; tone?: string } = $props();
</script>

<span class="badge {tone}">{text}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    padding: 1px 8px;
    border-radius: var(--r-full);
    font-size: var(--fs-meta);
    font-weight: 600;
    line-height: 18px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-lo);
    white-space: nowrap;
  }
  .safe {
    color: #2e7d32;
    border-color: #2e7d32;
    background: color-mix(in srgb, #2e7d32 14%, transparent);
  }
  .caution {
    color: #b26a00;
    border-color: #b26a00;
    background: color-mix(in srgb, #b26a00 14%, transparent);
  }
  .risk {
    color: #c62828;
    border-color: #c62828;
    background: color-mix(in srgb, #c62828 14%, transparent);
  }
  .info {
    color: #1565c0;
    border-color: #1565c0;
    background: color-mix(in srgb, #1565c0 14%, transparent);
  }
</style>
