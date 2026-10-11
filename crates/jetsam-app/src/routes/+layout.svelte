<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import {
    initialize,
    session,
    refreshStatus,
    loadUsage,
    loadIgnored
  } from '$lib/session.svelte';
  import { api } from '$lib/api';
  import Banner from '$lib/components/Banner.svelte';

  let { children } = $props();

  const items = [
    { href: '/', label: 'Dashboard' },
    { href: '/packages', label: 'Packages' },
    { href: '/cleanup', label: 'Cleanup' },
    { href: '/updates', label: 'Updates' },
    { href: '/settings', label: 'Settings' },
    { href: '/onboarding', label: 'Onboarding' }
  ];

  let theme = $state<'light' | 'dark'>('dark');
  let path = $state('/');

  function applyTheme(next: 'light' | 'dark') {
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('jetsam-theme', next);
  }

  async function startDaemon() {
    try {
      await api.startDaemon();
      await refreshStatus();
      await loadUsage();
      await loadIgnored();
    } catch (e) {
      session.error = String(e);
    }
  }

  onMount(() => {
    path = window.location.pathname;
    const saved = localStorage.getItem('jetsam-theme') as 'light' | 'dark' | null;
    theme = saved ?? 'dark';
    applyTheme(theme);
    void initialize();
  });
</script>

<div class="shell">
  <aside class="sidebar">
    <div class="brand">
      <span class="logo">J</span>
      <span class="name">Jetsam</span>
    </div>
    <nav>
      {#each items as item (item.href)}
        <a href={item.href} class:active={path === item.href}>{item.label}</a>
      {/each}
    </nav>
  </aside>

  <div class="main">
    <header class="topbar">
      <div class="row">
        {#if session.status?.daemon}
          <span class="chip ok">daemon v{session.status.daemon.daemon_version}</span>
        {:else}
          <span class="chip off">daemon offline</span>
        {/if}
        {#if session.status?.packages_loaded}
          <span class="chip">{session.status.packages} packages</span>
        {/if}
      </div>
      <div class="row">
        <button
          class="theme"
          type="button"
          aria-label="Toggle theme"
          onclick={() => {
            theme = theme === 'dark' ? 'light' : 'dark';
            applyTheme(theme);
          }}>{theme === 'dark' ? '☾' : '☀'}</button
        >
      </div>
    </header>

    <main class="content">
      {#if !session.status?.daemon}
        <div class="page-flush">
          <Banner tone="caution" title="Usage daemon is not running">
            Usage tracking is offline. Package data still works from metadata. →
            <button class="link" type="button" onclick={startDaemon}>Start daemon</button>
          </Banner>
        </div>
      {/if}
      {@render children?.()}
    </main>
  </div>
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: 232px 1fr;
    min-height: 100vh;
  }
  .sidebar {
    background: var(--surface-1);
    border-right: 1px solid var(--border);
    padding: var(--sp-4) var(--sp-3);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3) var(--sp-5);
  }
  .logo {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--accent);
    color: #fff;
    display: grid;
    place-items: center;
    font-weight: 800;
  }
  .name {
    font-weight: 700;
    font-size: 16px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  nav a {
    display: block;
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--r-sm);
    color: var(--text-lo);
    text-decoration: none;
    font-weight: 500;
  }
  nav a:hover {
    background: var(--surface-2);
    color: var(--text-hi);
  }
  nav a.active {
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    color: var(--text-hi);
  }
  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .topbar {
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--sp-5);
    border-bottom: 1px solid var(--border);
    background: var(--surface-0);
    position: sticky;
    top: 0;
    z-index: 10;
  }
  .chip {
    font-size: var(--fs-sm);
    padding: 2px 10px;
    border-radius: var(--r-full);
    border: 1px solid var(--border);
    color: var(--text-lo);
  }
  .chip.ok {
    color: #2e7d32;
    border-color: #2e7d32;
  }
  .chip.off {
    color: #b26a00;
    border-color: #b26a00;
  }
  .theme {
    border: 1px solid var(--border);
    background: var(--surface-1);
    border-radius: var(--r-sm);
    width: 32px;
    height: 32px;
    cursor: pointer;
  }
  .content {
    flex: 1;
    min-width: 0;
  }
  .page-flush {
    max-width: 1180px;
    margin: 0 auto;
    padding: var(--sp-5) var(--sp-5) 0;
  }
  .link {
    border: none;
    background: transparent;
    color: var(--accent);
    cursor: pointer;
    padding: 0;
    font-weight: 600;
  }
</style>
