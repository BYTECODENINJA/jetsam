<script lang="ts">
  import { onMount } from 'svelte';
  import {
    session,
    refreshStatus,
    loadUsage,
    loadIgnored,
    loadInventory
  } from '$lib/session.svelte';
  import { api } from '$lib/api';
  import { formatDuration, shortEpoch } from '$lib/format';
  import Button from '$lib/components/Button.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import Modal from '$lib/components/Modal.svelte';

  let busy = $state(false);
  let message = $state<string | null>(null);
  let wipeOpen = $state(false);

  onMount(() => {
    void refreshStatus();
  });

  const status = $derived(session.status);

  async function toggleOptIn(next: boolean) {
    busy = true;
    try {
      await api.setOptIn(next);
      message = next ? 'Usage recording enabled.' : 'Usage recording disabled.';
      await refreshStatus();
      await loadUsage();
    } catch (e) {
      message = String(e);
    } finally {
      busy = false;
    }
  }

  async function unignore(name: string) {
    try {
      await api.setIgnored(name, false);
      await refreshStatus();
    } catch (e) {
      message = String(e);
    }
  }

  async function wipe() {
    busy = true;
    try {
      await api.wipeData();
      wipeOpen = false;
      message = 'Recorded usage data erased.';
      await Promise.all([refreshStatus(), loadUsage()]);
    } catch (e) {
      message = String(e);
    } finally {
      busy = false;
    }
  }

  async function startDaemon() {
    try {
      await api.startDaemon();
      await refreshStatus();
    } catch (e) {
      message = String(e);
    }
  }
</script>

<div class="page">
  <h1>Settings</h1>

  {#if message}
    <Banner tone="info" title="Done">{message}</Banner>
  {/if}

  <section class="card">
    <div class="spread">
      <div>
        <h2>Usage recording</h2>
        <p class="muted">
          When on, the Jetsam user service records which known executables your own
          processes launch, their cumulative CPU time, and a coarse memory sample. It never
          reads command-line arguments, environment, keystrokes, window titles, or file
          contents, and nothing leaves this machine.
        </p>
        <a href="/onboarding">What exactly is recorded →</a>
      </div>
      <Switch checked={status?.daemon?.opted_in ?? false} label="Usage recording" onchange={toggleOptIn} />
    </div>
  </section>

  <section class="card">
    <h2>Daemon</h2>
    {#if status?.daemon}
      <ul class="stats">
        <li><span>State</span><Badge tone="safe" text="running" /></li>
        <li><span>Version</span><span class="mono">{status.daemon.daemon_version}</span></li>
        <li><span>Packages observed</span><span class="tnum">{status.daemon.packages_observed}</span></li>
        <li><span>History</span><span class="tnum">{status.daemon.days_of_history} day(s)</span></li>
        <li><span>Sampling every</span><span class="tnum">{status.daemon.sampling_interval_secs}s</span></li>
      </ul>
    {:else}
      <div class="spread">
        <p class="muted">The daemon is not reachable. Usage data will not advance.</p>
        <Button variant="primary" onclick={startDaemon}>Start daemon</Button>
      </div>
    {/if}
  </section>

  <section class="card">
    <h2>Ignored packages ({session.ignored.length})</h2>
    {#if session.ignored.length === 0}
      <p class="muted">No packages are ignored. Ignored packages are never suggested for removal.</p>
    {:else}
      <div class="chips">
        {#each session.ignored as name (name)}
          <span class="chip">
            <span class="mono">{name}</span>
            <button type="button" aria-label="Unignore {name}" onclick={() => unignore(name)}>✕</button>
          </span>
        {/each}
      </div>
    {/if}
  </section>

  <section class="card danger-card">
    <h2>Recorded data</h2>
    <p class="muted">
      Erase all samples the daemon has recorded. Your package database is untouched.
    </p>
    <Button variant="danger" onclick={() => (wipeOpen = true)}>Erase recorded data</Button>
  </section>
</div>

<Modal title="Erase recorded data" open={wipeOpen} onclose={() => (wipeOpen = false)}>
  <p>This permanently deletes the recorded usage database. It cannot be undone.</p>
  <div class="row end">
    <Button variant="ghost" onclick={() => (wipeOpen = false)}>Cancel</Button>
    <Button variant="danger" disabled={busy} onclick={wipe}>{busy ? 'Erasing…' : 'Erase'}</Button>
  </div>
</Modal>

<style>
  .card {
    margin-bottom: var(--sp-4);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    margin-top: var(--sp-2);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    border: 1px solid var(--border);
    border-radius: var(--r-full);
    padding: 2px 4px 2px 10px;
    font-size: var(--fs-sm);
  }
  .chip button {
    border: none;
    background: var(--surface-2);
    border-radius: var(--r-full);
    width: 18px;
    height: 18px;
    cursor: pointer;
    color: var(--text-lo);
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
    align-items: center;
  }
  .danger-card {
    border-color: color-mix(in srgb, var(--risk, #c62828) 45%, var(--border));
  }
  .end {
    justify-content: flex-end;
    margin-top: var(--sp-4);
  }
</style>
