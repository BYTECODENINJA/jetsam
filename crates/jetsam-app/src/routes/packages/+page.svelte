<script lang="ts">
  import { onMount } from 'svelte';
  import {
    session,
    loadInventory,
    loadUsage,
    loadIgnored,
    daysSince,
    usageFor,
    isIgnored
  } from '$lib/session.svelte';
  import { api, repoLabel, type PackageRecord } from '$lib/api';
  import { formatBytes, formatDate, formatDays } from '$lib/format';
  import Badge from '$lib/components/Badge.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  onMount(() => {
    void loadInventory();
    void loadUsage();
    void loadIgnored();
  });

  type SortKey =
    | 'name'
    | 'version'
    | 'repo'
    | 'installed_size'
    | 'install_date'
    | 'last_used'
    | 'launches'
    | 'reverse_dep_count';

  let query = $state('');
  let sortKey = $state<SortKey>('name');
  let sortAsc = $state(true);
  let scrollTop = $state(0);
  let viewport = $state(600);
  const rowHeight = 42;

  let selected = $state<Set<string>>(new Set());

  function rowValue(p: PackageRecord): string | number {
    switch (sortKey) {
      case 'last_used':
        return usageFor(p.name)?.last_used_epoch ?? -1;
      case 'launches':
        return usageFor(p.name)?.launches ?? -1;
      case 'installed_size':
        return p.installed_size;
      case 'install_date':
        return p.install_date ?? -1;
      case 'reverse_dep_count':
        return p.reverse_dep_count;
      case 'repo':
        return repoLabel(p.repo);
      case 'version':
        return p.version;
      default:
        return p.name;
    }
  }

  const rows = $derived.by(() => {
    const all = session.packages ?? [];
    const q = query.trim().toLowerCase();
    const filtered = q
      ? all.filter(
          (p) =>
            p.name.toLowerCase().includes(q) ||
            (p.description ?? '').toLowerCase().includes(q)
        )
      : all;
    return [...filtered].sort((a, b) => {
      const va = rowValue(a);
      const vb = rowValue(b);
      const cmp =
        typeof va === 'number' && typeof vb === 'number'
          ? va - vb
          : String(va).localeCompare(String(vb));
      return sortAsc ? cmp : -cmp;
    });
  });

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortAsc = !sortAsc;
    } else {
      sortKey = key;
      sortAsc = key === 'name';
    }
  }

  const total = $derived(rows.length);
  const start = $derived(Math.max(0, Math.floor(scrollTop / rowHeight) - 6));
  const visibleCount = $derived(Math.ceil(viewport / rowHeight) + 14);
  const visible = $derived(rows.slice(start, start + visibleCount));
  const padTop = $derived(start * rowHeight);
  const padBottom = $derived(Math.max(0, (total - start - visible.length) * rowHeight));

  function toggleSelect(name: string) {
    const next = new Set(selected);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    selected = next;
  }

  async function ignoreSelected(ignored: boolean) {
    for (const name of selected) {
      await api.setIgnored(name, ignored);
    }
    selected = new Set();
    await loadIgnored();
  }
</script>

<div class="page">
  <div class="spread">
    <h1>Packages</h1>
    <div class="row">
      <input
        class="search"
        type="search"
        placeholder="Search or filter packages"
        bind:value={query}
        aria-label="Search packages"
      />
      <Badge text={`${total} shown`} />
    </div>
  </div>

  {#if selected.size > 0}
    <div class="card bulk">
      <span class="grow">{selected.size} selected</span>
      <Button variant="ghost" onclick={() => ignoreSelected(true)}>Ignore</Button>
      <Button variant="ghost" onclick={() => ignoreSelected(false)}>Unignore</Button>
      <Button variant="ghost" onclick={() => (selected = new Set())}>Clear</Button>
    </div>
  {/if}

  {#if !session.packages}
    <EmptyState title="Loading inventory" message="Reading the package database…" />
  {:else if total === 0}
    <EmptyState title="No matches" message="Try a different search." />
  {:else}
    <div class="table card">
      <div class="tr head">
        <span class="c-sel"></span>
        <button class="th c-name" onclick={() => toggleSort('name')}>Package</button>
        <button class="th c-ver" onclick={() => toggleSort('version')}>Version</button>
        <button class="th c-repo" onclick={() => toggleSort('repo')}>Repo</button>
        <button class="th c-num" onclick={() => toggleSort('installed_size')}>Size</button>
        <button class="th c-num" onclick={() => toggleSort('install_date')}>Installed</button>
        <button class="th c-num" onclick={() => toggleSort('last_used')}>Last used</button>
        <button class="th c-num" onclick={() => toggleSort('launches')}>Launches</button>
        <button class="th c-num" onclick={() => toggleSort('reverse_dep_count')}>Revdeps</button>
      </div>
      <div
        class="viewport"
        bind:clientHeight={viewport}
        onscroll={(e) => (scrollTop = e.currentTarget.scrollTop)}
      >
        <div style="height: {padTop}px"></div>
        {#each visible as pkg (pkg.name)}
          <div class="tr" class:sel={selected.has(pkg.name)}>
            <span class="c-sel">
              <input
                type="checkbox"
                checked={selected.has(pkg.name)}
                onchange={() => toggleSelect(pkg.name)}
                aria-label="Select {pkg.name}"
              />
            </span>
            <span class="c-name">
              <a href="/package?name={pkg.name}">{pkg.name}</a>
              {#if pkg.protected}<Badge tone="caution" text="protected" />{/if}
              {#if isIgnored(pkg.name)}<Badge text="ignored" />{/if}
            </span>
            <span class="c-ver dim mono">{pkg.version}</span>
            <span class="c-repo">
              <Badge
                tone={pkg.repo === 'foreign' || typeof pkg.repo === 'object' ? 'caution' : 'neutral'}
                text={repoLabel(pkg.repo)}
              />
            </span>
            <span class="c-num tnum">{formatBytes(pkg.installed_size)}</span>
            <span class="c-num">{formatDate(pkg.install_date)}</span>
            <span class="c-num">{formatDays(daysSince(usageFor(pkg.name)?.last_used_epoch))}</span>
            <span class="c-num tnum">{usageFor(pkg.name)?.launches ?? '—'}</span>
            <span class="c-num tnum">{pkg.reverse_dep_count}</span>
          </div>
        {/each}
        <div style="height: {padBottom}px"></div>
      </div>
    </div>
  {/if}
</div>

<style>
  .search {
    border: 1px solid var(--border);
    background: var(--surface-1);
    border-radius: var(--r-sm);
    padding: 6px 12px;
    min-width: 260px;
  }
  .bulk {
    margin-bottom: var(--sp-3);
  }
  .table {
    padding: 0;
    max-height: 74vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .viewport {
    overflow-y: auto;
    flex: 1;
  }
  .tr {
    display: grid;
    grid-template-columns: 34px minmax(160px, 1.6fr) 120px 84px 84px 104px 110px 88px 76px;
    align-items: center;
    gap: var(--sp-2);
    padding: 0 var(--sp-3);
    height: 42px;
    border-bottom: 1px solid var(--border);
    font-size: var(--fs-sm);
  }
  .tr.sel {
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }
  .head {
    height: 40px;
    flex: 0 0 auto;
    background: var(--surface-1);
    color: var(--text-dim);
    font-weight: 600;
    position: sticky;
    top: 0;
    z-index: 2;
  }
  .th {
    border: none;
    background: transparent;
    color: var(--text-dim);
    font-weight: 600;
    font-size: var(--fs-sm);
    cursor: pointer;
    padding: 0;
    text-align: left;
    white-space: nowrap;
  }
  .th.c-num {
    text-align: right;
  }
  .c-sel {
    display: flex;
    justify-content: center;
  }
  .c-name {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-weight: 600;
  }
  .c-num {
    text-align: right;
  }
  .c-ver {
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
