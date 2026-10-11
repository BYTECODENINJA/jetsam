<script lang="ts">
  import { onMount } from 'svelte';
  import { session } from '$lib/session.svelte';
  import { api, type PendingUpdate, type HelperOutcome } from '$lib/api';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Badge from '$lib/components/Badge.svelte';

  let updates = $state<PendingUpdate[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let confirmOpen = $state(false);
  let busy = $state(false);
  let outcome = $state<HelperOutcome | null>(null);
  let query = $state('');

  async function check() {
    loading = true;
    error = null;
    try {
      updates = await api.checkUpdates();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(check);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return q ? updates.filter((u) => u.name.toLowerCase().includes(q)) : updates;
  });

  async function runUpgrade() {
    busy = true;
    outcome = null;
    try {
      outcome = await api.runUpgrade();
      confirmOpen = false;
      if (outcome.ok) await check();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="page">
  <div class="spread">
    <h1>Updates</h1>
    <Button variant="ghost" onclick={check} disabled={loading}>{loading ? 'Checking…' : 'Recheck'}</Button>
  </div>

  <Banner tone="info" title="Full-system upgrades only">
    Jetsam never installs partial or selective updates — that is the safe path Arch
    supports. This runs a plain <span class="mono">pacman -Syu</span> after you confirm.
  </Banner>

  {#if error}
    <Banner tone="risk" title="Update check failed">{error}</Banner>
  {/if}

  {#if outcome}
    <Banner tone={outcome.ok ? 'safe' : 'risk'} title={outcome.ok ? 'Upgrade finished' : 'Upgrade failed'}>
      {#if outcome.stderr}<span class="mono">{outcome.stderr}</span>{:else}Exit code {outcome.exit_code}.{/if}
    </Banner>
  {/if}

  {#if loading}
    <EmptyState title="Checking for updates…" message="Querying the configured mirrors." />
  {:else if updates.length === 0}
    <EmptyState title="System is up to date" message="No pending package updates were found." />
  {:else}
    <div class="card hero-card">
      <div>
        <span class="muted">Pending</span>
        <div class="hero tnum">{updates.length}</div>
        <span class="muted">package{updates.length === 1 ? '' : 's'} can be updated</span>
      </div>
      <Button variant="primary" onclick={() => (confirmOpen = true)}>Update system</Button>
    </div>

    <div class="spread filter">
      <input class="search" type="search" placeholder="Filter packages" bind:value={query} aria-label="Filter updates" />
      <Badge text={`${filtered.length} shown`} />
    </div>

    <div class="list card">
      {#each filtered as u (u.name)}
        <div class="row-item">
          <span class="mono name">{u.name}</span>
          <span class="vers mono">
            <span class="dim">{u.installed}</span> → <span>{u.available}</span>
          </span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<Modal title="Update system" open={confirmOpen} onclose={() => (confirmOpen = false)}>
  <p>
    This will update <strong class="tnum">{updates.length}</strong> package(s) with
    <span class="mono">pacman -Syu</span>.
  </p>
  <p class="muted">You will be asked for your password. Do not power off during the upgrade.</p>
  <div class="row end">
    <Button variant="ghost" onclick={() => (confirmOpen = false)}>Cancel</Button>
    <Button variant="primary" disabled={busy} onclick={runUpgrade}>
      {busy ? 'Upgrading…' : 'Upgrade now'}
    </Button>
  </div>
</Modal>

<style>
  .hero-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-4);
    margin-bottom: var(--sp-4);
  }
  .filter {
    margin-bottom: var(--sp-3);
  }
  .list {
    padding: 0;
  }
  .row-item {
    display: grid;
    grid-template-columns: 1fr 200px;
    gap: var(--sp-3);
    padding: var(--sp-2) var(--sp-4);
    border-bottom: 1px solid var(--border);
    font-size: var(--fs-sm);
  }
  .name {
    font-weight: 600;
    overflow-wrap: anywhere;
  }
  .vers {
    text-align: right;
  }
  .search {
    border: 1px solid var(--border);
    background: var(--surface-1);
    border-radius: var(--r-sm);
    padding: 6px 12px;
    min-width: 260px;
  }
  .end {
    justify-content: flex-end;
    margin-top: var(--sp-4);
  }
</style>
