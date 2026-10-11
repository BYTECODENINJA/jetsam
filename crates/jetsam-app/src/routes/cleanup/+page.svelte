<script lang="ts">
  import { onMount } from 'svelte';
  import {
    session,
    loadInventory,
    loadUsage,
    loadIgnored,
    loadCleanup
  } from '$lib/session.svelte';
  import { api, type Candidate, type RemovalPlan, type HelperOutcome } from '$lib/api';
  import { formatBytes, formatDays } from '$lib/format';
  import Badge from '$lib/components/Badge.svelte';
  import Button from '$lib/components/Button.svelte';
  import ScoreBar from '$lib/components/ScoreBar.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import Banner from '$lib/components/Banner.svelte';

  let selected = $state<Set<string>>(new Set());
  let minScore = $state(0);
  let showProtected = $state(false);
  let confirmOpen = $state(false);
  let plan = $state<RemovalPlan | null>(null);
  let outcome = $state<HelperOutcome | null>(null);
  let busy = $state(false);

  const cleanup = $derived(session.cleanup);

  onMount(async () => {
    await Promise.all([loadInventory(), loadUsage(), loadIgnored(), loadCleanup()]);
    const focus = new URLSearchParams(window.location.search).get('focus');
    if (focus) selected = new Set([focus]);
  });

  const candidates = $derived(
    (cleanup?.candidates ?? []).filter((c) => c.score.total >= minScore)
  );

  const selectedCandidates = $derived(
    (cleanup?.candidates ?? []).filter((c) => selected.has(c.name))
  );

  const selectedFreed = $derived(
    selectedCandidates.reduce((sum, c) => sum + Math.max(0, c.reclaimable_bytes), 0)
  );

  function toggle(name: string) {
    const next = new Set(selected);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    selected = next;
  }

  async function ignoreOne(name: string) {
    try {
      await api.setIgnored(name, true);
      await loadIgnored();
      const next = new Set(selected);
      next.delete(name);
      selected = next;
    } catch (e) {
      session.error = String(e);
    }
  }

  function toggleAll() {
    selected = selected.size === candidates.length ? new Set() : new Set(candidates.map((c) => c.name));
  }

  async function openConfirm() {
    if (selected.size === 0) return;
    busy = true;
    outcome = null;
    try {
      plan = await api.dryRun([...selected]);
      confirmOpen = true;
    } catch (e) {
      session.error = String(e);
    } finally {
      busy = false;
    }
  }

  async function apply() {
    busy = true;
    try {
      outcome = await api.applyRemove([...selected]);
      if (outcome.ok) {
        selected = new Set();
        await loadInventory(true);
        await loadCleanup();
      }
      confirmOpen = false;
    } catch (e) {
      outcome = {
        action: 'remove',
        ok: false,
        exit_code: null,
        stdout: '',
        stderr: String(e)
      };
    } finally {
      busy = false;
    }
  }

  async function ignore(name: string) {
    await api.setIgnored(name, true);
    selected = new Set([...selected].filter((n) => n !== name));
    await loadIgnored();
    await loadCleanup();
  }

  const reasonTone = (c: Candidate) =>
    c.breaks.length > 0 ? 'risk' : !cleanup?.usage_known ? 'caution' : c.score.total >= 66 ? 'risk' : 'neutral';
</script>

<div class="page">
  <div class="spread">
    <h1>Cleanup</h1>
    <button class="refresh" type="button" onclick={() => loadCleanup()} disabled={busy}>Recompute</button>
  </div>

  {#if cleanup && !cleanup.usage_known}
    <Banner tone="info" title="Usage still accumulating">
      These suggestions rank mostly by size, explicitness and risk — staleness is not
      yet known. Revisit after a couple of weeks of data.
    </Banner>
  {/if}

  {#if outcome}
    <Banner tone={outcome.ok ? 'safe' : 'risk'} title={outcome.ok ? 'Removal completed' : 'Removal failed'}>
      {#if outcome.stderr}<span class="mono">{outcome.stderr}</span>{:else}Exit code {outcome.exit_code}.{/if}
    </Banner>
  {/if}

  {#if !cleanup}
    <EmptyState title="Analyzing…" />
  {:else if candidates.length === 0}
    <EmptyState title="Nothing to suggest" message="No cleanup candidates above the current threshold." />
  {:else}
    <div class="toolbar card">
      <label class="row">
        <input type="checkbox" checked={selected.size === candidates.length && candidates.length > 0} indeterminate={selected.size > 0 && selected.size < candidates.length} onchange={toggleAll} />
        <span>{selected.size} selected · {formatBytes(selectedFreed)} reclaimable</span>
      </label>
      <label class="row dim">
        Min score
        <input type="range" min="0" max="90" step="5" bind:value={minScore} />
        <span class="tnum">{minScore}</span>
      </label>
      <Button variant="primary" disabled={selected.size === 0 || busy} onclick={openConfirm}>Review &amp; remove</Button>
    </div>

    <div class="list card">
      {#each candidates as c (c.name)}
        <label class="cand" class:sel={selected.has(c.name)}>
          <input type="checkbox" checked={selected.has(c.name)} onchange={() => toggle(c.name)} aria-label="Select {c.name}" />
          <div class="info">
            <div class="row wrap">
              <a class="mono name" href="/package?name={c.name}">{c.name}</a>
              {#if c.days_since_last_use == null}<Badge text="usage unknown" />{/if}
              {#if c.exclusive_orphans.length > 0}
                <Badge tone="neutral" text={`+${c.exclusive_orphans.length} orphan${c.exclusive_orphans.length === 1 ? '' : 's'}`} />
              {/if}
              {#if c.breaks.length > 0}<Badge tone="risk" text={`breaks ${c.breaks.length}`} />{/if}
            </div>
            <div class="dim meta">
              {formatBytes(c.reclaimable_bytes)} · last used {formatDays(c.days_since_last_use)} ·
              {c.reverse_dep_count} revdep{c.reverse_dep_count === 1 ? '' : 's'}
            </div>
          </div>
          <div class="score">
            <ScoreBar value={c.score.total} label={c.name} />
            <button class="ignore" type="button" onclick={(e) => { e.preventDefault(); ignoreOne(c.name); }}>ignore</button>
          </div>
        </label>
      {/each}
    </div>

    <details class="protected card" bind:open={showProtected}>
      <summary>Protected packages ({cleanup.protected.length})</summary>
      <p class="dim">Jetsam never suggests removing these.</p>
      <div class="chips">
        {#each cleanup.protected as p (p.name)}
          <span class="chip" title={p.reason}><span class="mono">{p.name}</span> · <span class="dim">{p.reason}</span></span>
        {/each}
      </div>
    </details>
  {/if}
</div>

<Modal title="Confirm removal" open={confirmOpen} onclose={() => (confirmOpen = false)}>
  {#if plan}
    <p>This will free <strong class="tnum">{formatBytes(plan.freed_bytes)}</strong>.</p>
    <p class="muted">{plan.remove.length} package(s) removed
      {#if plan.exclusive_orphans.length > 0}, including {plan.exclusive_orphans.length} exclusive orphan(s){/if}.</p>
    {#if plan.breaks.length > 0}
      <Banner tone="risk" title="Dependency conflict">
        {plan.breaks.length} retained package(s) depend on this set; pacman will refuse the transaction.
      </Banner>
    {/if}
    <ul class="removeList mono">
      {#each plan.remove.slice(0, 40) as n (n)}<li>{n}</li>{/each}
      {#if plan.remove.length > 40}<li class="dim">+{plan.remove.length - 40} more</li>{/if}
    </ul>
    <div class="row end">
      <Button variant="ghost" onclick={() => (confirmOpen = false)}>Cancel</Button>
      <Button variant="danger" disabled={busy} onclick={apply}>
        {busy ? 'Removing…' : 'Remove (asks for password)'}
      </Button>
    </div>
  {:else}
    <p>Computing plan…</p>
  {/if}
</Modal>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--sp-4);
    flex-wrap: wrap;
    margin-bottom: var(--sp-3);
    position: sticky;
    top: 56px;
    z-index: 6;
  }
  .toolbar > label:first-child {
    flex: 1;
  }
  .list {
    padding: 0;
  }
  .cand {
    display: grid;
    grid-template-columns: 32px 1fr 190px;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-3);
    border-bottom: 1px solid var(--border);
    cursor: pointer;
  }
  .cand.sel {
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }
  .name {
    font-weight: 600;
  }
  .info {
    min-width: 0;
  }
  .meta {
    font-size: var(--fs-sm);
    margin-top: 2px;
  }
  .score {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    justify-content: flex-end;
  }
  .ignore {
    border: none;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    font-size: var(--fs-meta);
    text-decoration: underline;
  }
  .protected {
    margin-top: var(--sp-4);
  }
  .protected summary {
    cursor: pointer;
    font-weight: 600;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
    margin-top: var(--sp-2);
  }
  .chip {
    border: 1px solid var(--border);
    border-radius: var(--r-full);
    padding: 1px 8px;
    font-size: var(--fs-meta);
    color: var(--text-lo);
    max-width: 100%;
  }
  .removeList {
    max-height: 200px;
    overflow: auto;
    font-size: var(--fs-sm);
    color: var(--text-lo);
    padding-left: var(--sp-4);
  }
  .end {
    justify-content: flex-end;
    margin-top: var(--sp-4);
  }
  .refresh {
    border: 1px solid var(--border);
    background: var(--surface-1);
    border-radius: var(--r-sm);
    padding: 6px 12px;
    cursor: pointer;
  }
</style>
