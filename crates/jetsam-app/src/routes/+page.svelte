<script lang="ts">
  import { onMount } from 'svelte';
  import { session, loadInventory, loadUsage } from '$lib/session.svelte';
  import { api, type CleanupView, type PendingUpdate } from '$lib/api';
  import { formatBytes, formatDays } from '$lib/format';
  import ScoreBar from '$lib/components/ScoreBar.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import Banner from '$lib/components/Banner.svelte';

  let cleanup = $state<CleanupView | null>(null);
  let updates = $state<PendingUpdate[]>([]);

  onMount(async () => {
    await Promise.all([
      loadInventory(),
      loadUsage(),
      api.cleanupSuggestions().then((c) => (cleanup = c)).catch(() => {}),
      api.checkUpdates().then((u) => (updates = u)).catch(() => {})
    ]);
  });

  const reclaimable = $derived(
    cleanup
      ? cleanup.candidates
          .filter((c) => c.score.total >= 50)
          .reduce((sum, c) => sum + Math.max(0, c.reclaimable_bytes), 0)
      : 0
  );

  const topUnused = $derived(
    cleanup
      ? [...cleanup.candidates]
          .filter((c) => c.days_since_last_use != null)
          .sort((a, b) => (b.days_since_last_use ?? 0) - (a.days_since_last_use ?? 0))
          .slice(0, 5)
      : []
  );

  const dayNumber = $derived(
    session.status?.daemon ? Math.max(1, session.status.daemon.days_of_history) : 0
  );

  const usageKnown = $derived(cleanup?.usage_known ?? false);
</script>

<div class="page">
  <div class="spread">
    <h1>Dashboard</h1>
    <button class="refresh" type="button" onclick={() => loadInventory(true)} disabled={session.loading}>
      {session.loading ? 'Scanning…' : 'Rescan'}
    </button>
  </div>

  {#if session.error}
    <Banner tone="risk" title="Something went wrong">{session.error}</Banner>
  {/if}

  {#if !usageKnown}
    <Banner tone="info" title="Usage data: accumulating">
      Cleanup scores currently use package metadata only (size, explicitness, risk).
      Staleness will appear as the daemon observes your usage.{#if dayNumber > 0}
        Day {dayNumber} of ~14.{/if}
      <a href="/onboarding">Read what we record →</a>
    </Banner>
  {/if}

  <section class="card hero-card">
    <span class="muted">Reclaimable</span>
    <div class="hero tnum">{formatBytes(reclaimable)}</div>
    <div class="muted">
      {session.summary
        ? `${formatBytes(session.summary.orphan_bytes)} orphans · ${formatBytes(
            session.summary.foreign_bytes
          )} foreign · ${formatBytes(session.summary.total_bytes)} installed`
        : 'loading…'}
    </div>
    <a class="cta" href="/cleanup">Review cleanup →</a>
  </section>

  <div class="cols">
    <section class="card">
      <div class="spread">
        <h2>Updates</h2>
        <a href="/updates">Open</a>
      </div>
      {#if updates.length === 0}
        <p class="muted">Up to date.</p>
      {:else}
        <p class="big tnum">{updates.length}</p>
        <p class="muted">packages pending a full system upgrade.</p>
      {/if}
    </section>

    <section class="card">
      <h2>Disk breakdown</h2>
      {#if session.summary}
        <ul class="stats">
          <li><span>Installed packages</span><span class="tnum">{session.summary.packages}</span></li>
          <li><span>Orphans</span><span class="tnum">{session.summary.orphans}</span></li>
          <li><span>Protected</span><span class="tnum">{session.summary.protected}</span></li>
          <li><span>Last sync</span><span>{session.lastSync ? new Date(session.lastSync).toLocaleTimeString() : '—'}</span></li>
        </ul>
      {:else}
        <p class="muted">Loading inventory…</p>
      {/if}
    </section>
  </div>

  <section class="card">
    <div class="spread">
      <h2>Most dormant</h2>
      <a href="/cleanup">All suggestions</a>
    </div>
    {#if topUnused.length === 0}
      <EmptyState
        title="No usage history yet"
        message="Once the daemon has observed your usage, dormant packages will appear here."
      />
    {:else}
      <table>
        <thead>
          <tr>
            <th>Package</th>
            <th>Last used</th>
            <th>Reclaimable</th>
            <th>Score</th>
          </tr>
        </thead>
        <tbody>
          {#each topUnused as c (c.name)}
            <tr>
              <td class="mono"><a href="/package?name={c.name}">{c.name}</a></td>
              <td>{formatDays(c.days_since_last_use)}</td>
              <td class="tnum">{formatBytes(c.reclaimable_bytes)}</td>
              <td><ScoreBar value={c.score.total} label={c.name} /></td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>
</div>

<style>
  .hero-card {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    margin-bottom: var(--sp-4);
  }
  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-4);
    margin-bottom: var(--sp-4);
  }
  .cta {
    margin-top: var(--sp-2);
    font-weight: 600;
  }
  .big {
    font-size: 32px;
    font-weight: 700;
    margin: var(--sp-2) 0 0;
  }
  .stats {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .stats li {
    display: flex;
    justify-content: space-between;
    color: var(--text-lo);
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
    text-align: left;
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border);
    font-size: var(--fs-sm);
  }
  th {
    color: var(--text-dim);
    font-weight: 600;
  }
  .refresh {
    border: 1px solid var(--border);
    background: var(--surface-1);
    border-radius: var(--r-sm);
    padding: 6px 12px;
    cursor: pointer;
  }
  @media (max-width: 720px) {
    .cols {
      grid-template-columns: 1fr;
    }
  }
</style>
