import { a4 as ensure_array_like, a5 as attr, a3 as derived, a7 as stringify } from "../../../chunks/index.js";
import { e as escape_html } from "../../../chunks/escaping.js";
import { s as session, r as refreshStatus, c as loadUsage } from "../../../chunks/session.svelte.js";
import { a as api } from "../../../chunks/api.js";
import { B as Button } from "../../../chunks/Button.js";
import { S as Switch } from "../../../chunks/Switch.js";
import { B as Badge } from "../../../chunks/Badge.js";
import { B as Banner } from "../../../chunks/Banner.js";
import { M as Modal } from "../../../chunks/Modal.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let busy = false;
    let message = null;
    let wipeOpen = false;
    const status = derived(() => session.status);
    async function toggleOptIn(next) {
      busy = true;
      try {
        await api.setOptIn(next);
        message = next ? "Usage recording enabled." : "Usage recording disabled.";
        await refreshStatus();
        await loadUsage();
      } catch (e) {
        message = String(e);
      } finally {
        busy = false;
      }
    }
    async function wipe() {
      busy = true;
      try {
        await api.wipeData();
        wipeOpen = false;
        message = "Recorded usage data erased.";
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
    $$renderer2.push(`<div class="page"><h1>Settings</h1> `);
    if (message) {
      $$renderer2.push("<!--[0-->");
      Banner($$renderer2, {
        tone: "info",
        title: "Done",
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->${escape_html(message)}`);
        }
      });
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <section class="card svelte-1i19ct2"><div class="spread"><div><h2>Usage recording</h2> <p class="muted">When on, the Jetsam user service records which known executables your own
          processes launch, their cumulative CPU time, and a coarse memory sample. It never
          reads command-line arguments, environment, keystrokes, window titles, or file
          contents, and nothing leaves this machine.</p> <a href="/onboarding">What exactly is recorded →</a></div> `);
    Switch($$renderer2, {
      checked: status()?.daemon?.opted_in ?? false,
      label: "Usage recording",
      onchange: toggleOptIn
    });
    $$renderer2.push(`<!----></div></section> <section class="card svelte-1i19ct2"><h2>Daemon</h2> `);
    if (status()?.daemon) {
      $$renderer2.push(`<!--[0--><ul class="stats svelte-1i19ct2"><li class="svelte-1i19ct2"><span>State</span>`);
      Badge($$renderer2, { tone: "safe", text: "running" });
      $$renderer2.push(`<!----></li> <li class="svelte-1i19ct2"><span>Version</span><span class="mono">${escape_html(status().daemon.daemon_version)}</span></li> <li class="svelte-1i19ct2"><span>Packages observed</span><span class="tnum">${escape_html(status().daemon.packages_observed)}</span></li> <li class="svelte-1i19ct2"><span>History</span><span class="tnum">${escape_html(status().daemon.days_of_history)} day(s)</span></li> <li class="svelte-1i19ct2"><span>Sampling every</span><span class="tnum">${escape_html(status().daemon.sampling_interval_secs)}s</span></li></ul>`);
    } else {
      $$renderer2.push(`<!--[-1--><div class="spread"><p class="muted">The daemon is not reachable. Usage data will not advance.</p> `);
      Button($$renderer2, {
        variant: "primary",
        onclick: startDaemon,
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->Start daemon`);
        }
      });
      $$renderer2.push(`<!----></div>`);
    }
    $$renderer2.push(`<!--]--></section> <section class="card svelte-1i19ct2"><h2>Ignored packages (${escape_html(session.ignored.length)})</h2> `);
    if (session.ignored.length === 0) {
      $$renderer2.push(`<!--[0--><p class="muted">No packages are ignored. Ignored packages are never suggested for removal.</p>`);
    } else {
      $$renderer2.push(`<!--[-1--><div class="chips svelte-1i19ct2"><!--[-->`);
      const each_array = ensure_array_like(session.ignored);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let name = each_array[$$index];
        $$renderer2.push(`<span class="chip svelte-1i19ct2"><span class="mono">${escape_html(name)}</span> <button type="button"${attr("aria-label", `Unignore ${stringify(name)}`)} class="svelte-1i19ct2">✕</button></span>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></section> <section class="card danger-card svelte-1i19ct2"><h2>Recorded data</h2> <p class="muted">Erase all samples the daemon has recorded. Your package database is untouched.</p> `);
    Button($$renderer2, {
      variant: "danger",
      onclick: () => wipeOpen = true,
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->Erase recorded data`);
      }
    });
    $$renderer2.push(`<!----></section></div> `);
    Modal($$renderer2, {
      title: "Erase recorded data",
      open: wipeOpen,
      onclose: () => wipeOpen = false,
      children: ($$renderer3) => {
        $$renderer3.push(`<p>This permanently deletes the recorded usage database. It cannot be undone.</p> <div class="row end svelte-1i19ct2">`);
        Button($$renderer3, {
          variant: "ghost",
          onclick: () => wipeOpen = false,
          children: ($$renderer4) => {
            $$renderer4.push(`<!---->Cancel`);
          }
        });
        $$renderer3.push(`<!----> `);
        Button($$renderer3, {
          variant: "danger",
          disabled: busy,
          onclick: wipe,
          children: ($$renderer4) => {
            $$renderer4.push(`<!---->${escape_html(busy ? "Erasing…" : "Erase")}`);
          }
        });
        $$renderer3.push(`<!----></div>`);
      }
    });
    $$renderer2.push(`<!---->`);
  });
}
export {
  _page as default
};
