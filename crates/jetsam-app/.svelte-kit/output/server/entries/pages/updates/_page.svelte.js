import { a5 as attr, a4 as ensure_array_like, a3 as derived } from "../../../chunks/index.js";
import { e as escape_html } from "../../../chunks/escaping.js";
import { a as api } from "../../../chunks/api.js";
import { B as Button } from "../../../chunks/Button.js";
import { E as EmptyState } from "../../../chunks/EmptyState.js";
import { B as Banner } from "../../../chunks/Banner.js";
import { M as Modal } from "../../../chunks/Modal.js";
import { B as Badge } from "../../../chunks/Badge.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let updates = [];
    let loading = true;
    let error = null;
    let confirmOpen = false;
    let busy = false;
    let outcome = null;
    let query = "";
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
    const filtered = derived(() => {
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
    $$renderer2.push(`<div class="page"><div class="spread"><h1>Updates</h1> `);
    Button($$renderer2, {
      variant: "ghost",
      onclick: check,
      disabled: loading,
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->${escape_html(loading ? "Checking…" : "Recheck")}`);
      }
    });
    $$renderer2.push(`<!----></div> `);
    Banner($$renderer2, {
      tone: "info",
      title: "Full-system upgrades only",
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->Jetsam never installs partial or selective updates — that is the safe path Arch
    supports. This runs a plain <span class="mono">pacman -Syu</span> after you confirm.`);
      }
    });
    $$renderer2.push(`<!----> `);
    if (error) {
      $$renderer2.push("<!--[0-->");
      Banner($$renderer2, {
        tone: "risk",
        title: "Update check failed",
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->${escape_html(error)}`);
        }
      });
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (outcome) {
      $$renderer2.push("<!--[0-->");
      Banner($$renderer2, {
        tone: outcome.ok ? "safe" : "risk",
        title: outcome.ok ? "Upgrade finished" : "Upgrade failed",
        children: ($$renderer3) => {
          if (outcome.stderr) {
            $$renderer3.push(`<!--[0--><span class="mono">${escape_html(outcome.stderr)}</span>`);
          } else {
            $$renderer3.push(`<!--[-1-->Exit code ${escape_html(outcome.exit_code)}.`);
          }
          $$renderer3.push(`<!--]-->`);
        }
      });
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (loading) {
      $$renderer2.push("<!--[0-->");
      EmptyState($$renderer2, {
        title: "Checking for updates…",
        message: "Querying the configured mirrors."
      });
    } else if (updates.length === 0) {
      $$renderer2.push("<!--[1-->");
      EmptyState($$renderer2, {
        title: "System is up to date",
        message: "No pending package updates were found."
      });
    } else {
      $$renderer2.push(`<!--[-1--><div class="card hero-card svelte-tk0old"><div><span class="muted">Pending</span> <div class="hero tnum">${escape_html(updates.length)}</div> <span class="muted">package${escape_html(updates.length === 1 ? "" : "s")} can be updated</span></div> `);
      Button($$renderer2, {
        variant: "primary",
        onclick: () => confirmOpen = true,
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->Update system`);
        }
      });
      $$renderer2.push(`<!----></div> <div class="spread filter svelte-tk0old"><input class="search svelte-tk0old" type="search" placeholder="Filter packages"${attr("value", query)} aria-label="Filter updates"/> `);
      Badge($$renderer2, { text: `${filtered().length} shown` });
      $$renderer2.push(`<!----></div> <div class="list card svelte-tk0old"><!--[-->`);
      const each_array = ensure_array_like(filtered());
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let u = each_array[$$index];
        $$renderer2.push(`<div class="row-item svelte-tk0old"><span class="mono name svelte-tk0old">${escape_html(u.name)}</span> <span class="vers mono svelte-tk0old"><span class="dim">${escape_html(u.installed)}</span> → <span>${escape_html(u.available)}</span></span></div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    Modal($$renderer2, {
      title: "Update system",
      open: confirmOpen,
      onclose: () => confirmOpen = false,
      children: ($$renderer3) => {
        $$renderer3.push(`<p>This will update <strong class="tnum">${escape_html(updates.length)}</strong> package(s) with <span class="mono">pacman -Syu</span>.</p> <p class="muted">You will be asked for your password. Do not power off during the upgrade.</p> <div class="row end svelte-tk0old">`);
        Button($$renderer3, {
          variant: "ghost",
          onclick: () => confirmOpen = false,
          children: ($$renderer4) => {
            $$renderer4.push(`<!---->Cancel`);
          }
        });
        $$renderer3.push(`<!----> `);
        Button($$renderer3, {
          variant: "primary",
          disabled: busy,
          onclick: runUpgrade,
          children: ($$renderer4) => {
            $$renderer4.push(`<!---->${escape_html(busy ? "Upgrading…" : "Upgrade now")}`);
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
