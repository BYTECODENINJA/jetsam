<script lang="ts">
  import { onMount } from 'svelte';
  import { api, repoLabel, type PackageDetail, type RemovalPlan } from '$lib/api';
  import { formatBytes, formatDate, formatDays, shortEpoch } from '$lib/format';
  import { usageFor, loadUsage, daysSince } from '$lib/session.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  let name = $state('');
  let detail = $state<PackageDetail | null>(null);
  let plan = $state<RemovalPlan | null>(null);
  let error = $state<string | null>(null);
  let fileQuery = $state('');
  let loading = $state(true);

  onMount(async () => {
    name = new URLSearchParams(window.location.search).get('name') ?? '';
    if (!name) {
      error = 'No package specified.';
      loading = false;
      return;
    }
    try {
      detail = await api.packageDetail(name);
      plan = await api.dryRun([name]);
      await loadUsage();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const files = $derived.by(() => {
    if (!detail) return [];
    const q = fileQuery.trim().toLowerCase();
    const list = q ? detail.files.filter((f) => f.toLowerCase().includes(q)) : detail.files;
    return list.slice(0, 400);
  });

  const classLabel: Record<string, string> = {
    desktop_app: 'Desktop app',
    cli_tool: 'CLI tool',
    library: 'Library',
    daemon: 'Daemon',
    kernel_or_driver: 'Kernel / driver',
    data: 'Data',
    unknown: 'Unknown'
  };
</script>

<div class="page">
  {#if loading}
    <EmptyState title="Loading…" />
  {:else if error}
    <EmptyState title="Package not found" message={error}>
      <a href="/packages">← Back to packages</a>
    </EmptyState>
  {:else if detail}
    <a class="back" href="/packages">← Packages</a>
    <div class="spread head">
      <div>
        <h1 class="mono">{detail.record.name}</h1>
        <div class="row wrap muted">
          <span class="mono">{detail.record.version}</span>
          <Badge
            tone={detail.record.repo === 'foreign' || typeof detail.record.repo === 'object'
              ? 'caution'
              : 'neutral'}
            text={repoLabel(detail.record.repo)}
          />
          <Badge text={classLabel[detail.record.class] ?? detail.record.class} />
          {#if detail.record.protected}
            <Badge tone="risk" text={`protected · ${detail.record.protected_reason}`} />
          {/if}
          {#if detail.record.reason === 'explicit'}<Badge text="explicit" />{:else}<Badge text="dependency" />{/if}
        </div>
        {#if detail.record.description}<p class="muted">{detail.record.description}</p>{/if}
      </div>
      <div class="row">
        <Button variant="danger" disabled={detail.record.protected} onclick={() => (window.location.href = `/cleanup?focus=${detail?.record.name}`)}>
          Remove…
        </Button>
      </div>
    </div>

    <div class="cols">
      <section class="card">
        <h2>Usage</h2>
        <ul class="stats">
          <li><span>Launches</span><span class="tnum">{usageFor(name)?.launches ?? '—'}</span></li>
          <li><span>Last used</span><span>{daysSince(usageFor(name)?.last_used_epoch) != null ? formatDays(daysSince(usageFor(name)?.last_used_epoch)) : 'no data'}</span></li>
          <li><span>Avg PSS</span><span class="tnum">{formatBytes(usageFor(name)?.avg_pss_bytes ?? null)}</span></li>
          <li><span>First seen</span><span>{shortEpoch(usageFor(name)?.first_seen_epoch ?? null)}</span></li>
        </ul>
      </section>

      <section class="card">
        <h2>Removing this frees</h2>
        {#if plan}
          <div class="hero tnum">{formatBytes(plan.freed_bytes)}</div>
          <p class="muted">
            {plan.remove.length} package(s): this one
            {#if plan.exclusive_orphans.length > 0}
              plus {plan.exclusive_orphans.length} exclusive orphan(s)
            {/if}.
          </p>
          {#if plan.exclusive_orphans.length > 0}
            <div class="chips">
              {#each plan.exclusive_orphans.slice(0, 12) as o (o)}<span class="chip mono">{o}</span>{/each}
              {#if plan.exclusive_orphans.length > 12}<span class="dim">+{plan.exclusive_orphans.length - 12} more</span>{/if}
            </div>
          {/if}
          {#if plan.breaks.length > 0}
            <p class="risk-text" role="alert">
              ⚠ {plan.breaks.length} retained package(s) depend on this — pacman will refuse.
            </p>
          {/if}
        {:else}
          <p class="muted">No removal plan available.</p>
        {/if}
      </section>
    </div>

    <div class="cols">
      <section class="card">
        <h2>Depends on ({detail.record.depends.length})</h2>
        <div class="chips">
          {#each detail.record.depends.slice(0, 60) as dep (dep)}<a class="chip mono" href="/package?name={dep.replace(/[<>=].*$/, '')}">{dep}</a>{/each}
          {#if detail.record.depends.length === 0}<span class="dim">none</span>{/if}
        </div>
        <h2 class="mt">Required by ({detail.required_by.length})</h2>
        <div class="chips">
          {#each detail.required_by.slice(0, 60) as dep (dep)}<a class="chip mono" href="/package?name={dep}">{dep}</a>{/each}
          {#if detail.required_by.length === 0}<span class="dim">nothing</span>{/if}
        </div>
      </section>

      <section class="card">
        <div class="spread">
          <h2>Files ({detail.record.file_count})</h2>
          <input class="search" type="search" placeholder="Filter files" bind:value={fileQuery} aria-label="Filter files" />
        </div>
        <ul class="files mono">
          {#each files as file (file)}<li>{file}</li>{/each}
        </ul>
        {#if detail.files.length > files.length}
          <p class="dim">Showing {files.length} of {detail.files.length}.</p>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .back {
    display: inline-block;
    margin-bottom: var(--sp-3);
    font-size: var(--fs-sm);
  }
  .head {
    align-items: flex-start;
    margin-bottom: var(--sp-4);
  }
  .head h1 {
    margin-bottom: var(--sp-2);
  }
  .wrap {
    flex-wrap: wrap;
  }
  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-4);
    margin-bottom: var(--sp-4);
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
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
    margin: var(--sp-2) 0;
  }
  .chip {
    border: 1px solid var(--border);
    border-radius: var(--r-full);
    padding: 1px 8px;
    font-size: var(--fs-meta);
    color: var(--text-lo);
    text-decoration: none;
  }
  .files {
    list-style: none;
    padding: 0;
    margin: 0;
    max-height: 320px;
    overflow: auto;
    font-size: var(--fs-sm);
  }
  .files li {
    padding: 1px 0;
    color: var(--text-lo);
    overflow-wrap: anywhere;
  }
  .mt {
    margin-top: var(--sp-4);
  }
  .risk-text {
    color: #c62828;
    font-size: var(--fs-sm);
  }
  .search {
    border: 1px solid var(--border);
    background: var(--surface-1);
    border-radius: var(--r-sm);
    padding: 5px 10px;
    font-size: var(--fs-sm);
  }
  @media (max-width: 820px) {
    .cols {
      grid-template-columns: 1fr;
    }
  }
</style>
