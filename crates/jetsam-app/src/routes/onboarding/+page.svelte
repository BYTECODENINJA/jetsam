<script lang="ts">
  import { onMount } from 'svelte';
  import { session, refreshStatus } from '$lib/session.svelte';
  import { api } from '$lib/api';
  import Button from '$lib/components/Button.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import Banner from '$lib/components/Banner.svelte';

  let busy = $state(false);

  onMount(() => {
    void refreshStatus();
  });

  const optedIn = $derived(session.status?.daemon?.opted_in ?? false);

  async function toggle(next: boolean) {
    busy = true;
    try {
      await api.setOptIn(next);
      await api.startDaemon().catch(() => {});
      await refreshStatus();
      window.location.href = '/';
    } catch (e) {
      session.error = String(e);
    } finally {
      busy = false;
    }
  }

  const records = [
    ['Executable path', 'Which binary a launch resolved to, so a package can be credited.'],
    ['Launch count', 'How many times that executable started.'],
    ['Cumulative CPU time', 'CPU seconds a process tree used, summed per package.'],
    ['Coarse memory sample', 'Approximate resident memory, at a low interval — never contents.'],
    ['Timestamps', 'First seen, last seen, so staleness can be computed.']
  ];

  const never = [
    'Command-line arguments',
    'Environment variables',
    'Keystrokes or input',
    'Window titles or screen content',
    'File contents or names you open'
  ];
</script>

<div class="page narrow">
  <h1>Welcome to Jetsam</h1>
  <p class="lead">
    Jetsam shows what you actually use, then helps you reclaim space safely. It needs a
    small amount of local, opt-in usage data to do that.
  </p>

  <section class="card">
    <h2>What is recorded</h2>
    <p class="muted">Only for your own processes, only when enabled, only stored here on this machine.</p>
    <ul class="records">
      {#each records as [what, why] (what)}
        <li><strong>{what}</strong><span class="muted">{why}</span></li>
      {/each}
    </ul>
  </section>

  <section class="card">
    <h2>What is never recorded</h2>
    <ul class="never">
      {#each never as item (item)}<li>{item}</li>{/each}
    </ul>
  </section>

  <section class="card">
    <h2>Where it is stored</h2>
    <ul class="stats">
      <li><span>Data (samples)</span><span class="mono">~/.local/share/jetsam</span></li>
      <li><span>Configuration</span><span class="mono">~/.config/jetsam</span></li>
      <li><span>Socket (runtime)</span><span class="mono">$XDG_RUNTIME_DIR/jetsam</span></li>
    </ul>
    <p class="muted">No telemetry, no network access. Erase everything at any time from Settings.</p>
  </section>

  {#if session.error}
    <Banner tone="risk" title="Could not start">{session.error}</Banner>
  {/if}

  <div class="actions">
    <div class="row">
      <Switch checked={optedIn} label="Enable usage recording" onchange={toggle} />
      <span>{optedIn ? 'Enabled' : 'Disabled'}</span>
    </div>
    <Button variant="primary" disabled={busy} onclick={() => toggle(true)}>
      {busy ? 'Starting…' : 'Enable and continue'}
    </Button>
  </div>
  <button class="skip" type="button" onclick={() => (window.location.href = '/')}>
    Skip — use metadata only
  </button>
</div>

<style>
  .narrow {
    max-width: 760px;
  }
  .lead {
    font-size: 16px;
    color: var(--text-lo);
    margin-bottom: var(--sp-5);
  }
  .card {
    margin-bottom: var(--sp-4);
  }
  .records {
    list-style: none;
    padding: 0;
    margin: var(--sp-2) 0 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .records li {
    display: flex;
    flex-direction: column;
  }
  .records span {
    font-size: var(--fs-sm);
  }
  .never {
    margin: var(--sp-2) 0 0;
    padding-left: var(--sp-5);
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
  }
</style>
