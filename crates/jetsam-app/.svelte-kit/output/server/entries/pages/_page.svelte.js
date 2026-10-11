import { a5 as attr, a4 as ensure_array_like, a7 as stringify, a3 as derived } from "../../chunks/index.js";
import { e as escape_html } from "../../chunks/escaping.js";
import { s as session } from "../../chunks/session.svelte.js";
import "@tauri-apps/api/core";
import { f as formatBytes, a as formatDays } from "../../chunks/format.js";
import { S as ScoreBar } from "../../chunks/ScoreBar.js";
import { E as EmptyState } from "../../chunks/EmptyState.js";
import { B as Banner } from "../../chunks/Banner.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let updates = [];
    const reclaimable = derived(() => 0);
    const topUnused = derived(() => []);
    const dayNumber = derived(() => session.status?.daemon ? Math.max(1, session.status.daemon.days_of_history) : 0);
    const usageKnown = derived(() => false);
    $$renderer2.push(`<div class="page"><div class="spread"><h1>Dashboard</h1> <button class="refresh svelte-1uha8ag" type="button"${attr("disabled", session.loading, true)}>${escape_html(session.loading ? "Scanning…" : "Rescan")}</button></div> `);
    if (session.error) {
      $$renderer2.push("<!--[0-->");
      Banner($$renderer2, {
        tone: "risk",
        title: "Something went wrong",
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->${escape_html(session.error)}`);
        }
      });
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (!usageKnown()) {
      $$renderer2.push("<!--[0-->");
      Banner($$renderer2, {
        tone: "info",
        title: "Usage data: accumulating",
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->Cleanup scores currently use package metadata only (size, explicitness, risk).
      Staleness will appear as the daemon observes your usage.`);
          if (dayNumber() > 0) {
            $$renderer3.push(`<!--[0-->Day ${escape_html(dayNumber())} of ~14.`);
          } else {
            $$renderer3.push("<!--[-1-->");
          }
          $$renderer3.push(`<!--]--> <a href="/onboarding">Read what we record →</a>`);
        }
      });
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <section class="card hero-card svelte-1uha8ag"><span class="muted">Reclaimable</span> <div class="hero tnum">${escape_html(formatBytes(reclaimable()))}</div> <div class="muted">${escape_html(session.summary ? `${formatBytes(session.summary.orphan_bytes)} orphans · ${formatBytes(session.summary.foreign_bytes)} foreign · ${formatBytes(session.summary.total_bytes)} installed` : "loading…")}</div> <a class="cta svelte-1uha8ag" href="/cleanup">Review cleanup →</a></section> <div class="cols svelte-1uha8ag"><section class="card"><div class="spread"><h2>Updates</h2> <a href="/updates">Open</a></div> `);
    if (updates.length === 0) {
      $$renderer2.push(`<!--[0--><p class="muted">Up to date.</p>`);
    } else {
      $$renderer2.push(`<!--[-1--><p class="big tnum svelte-1uha8ag">${escape_html(updates.length)}</p> <p class="muted">packages pending a full system upgrade.</p>`);
    }
    $$renderer2.push(`<!--]--></section> <section class="card"><h2>Disk breakdown</h2> `);
    if (session.summary) {
      $$renderer2.push(`<!--[0--><ul class="stats svelte-1uha8ag"><li class="svelte-1uha8ag"><span>Installed packages</span><span class="tnum">${escape_html(session.summary.packages)}</span></li> <li class="svelte-1uha8ag"><span>Orphans</span><span class="tnum">${escape_html(session.summary.orphans)}</span></li> <li class="svelte-1uha8ag"><span>Protected</span><span class="tnum">${escape_html(session.summary.protected)}</span></li> <li class="svelte-1uha8ag"><span>Last sync</span><span>${escape_html(session.lastSync ? new Date(session.lastSync).toLocaleTimeString() : "—")}</span></li></ul>`);
    } else {
      $$renderer2.push(`<!--[-1--><p class="muted">Loading inventory…</p>`);
    }
    $$renderer2.push(`<!--]--></section></div> <section class="card"><div class="spread"><h2>Most dormant</h2> <a href="/cleanup">All suggestions</a></div> `);
    if (topUnused().length === 0) {
      $$renderer2.push("<!--[0-->");
      EmptyState($$renderer2, {
        title: "No usage history yet",
        message: "Once the daemon has observed your usage, dormant packages will appear here."
      });
    } else {
      $$renderer2.push(`<!--[-1--><table class="svelte-1uha8ag"><thead><tr><th class="svelte-1uha8ag">Package</th><th class="svelte-1uha8ag">Last used</th><th class="svelte-1uha8ag">Reclaimable</th><th class="svelte-1uha8ag">Score</th></tr></thead><tbody><!--[-->`);
      const each_array = ensure_array_like(topUnused());
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let c = each_array[$$index];
        $$renderer2.push(`<tr><td class="mono svelte-1uha8ag"><a${attr("href", `/package?name=${stringify(c.name)}`)}>${escape_html(c.name)}</a></td><td class="svelte-1uha8ag">${escape_html(formatDays(c.days_since_last_use))}</td><td class="tnum svelte-1uha8ag">${escape_html(formatBytes(c.reclaimable_bytes))}</td><td class="svelte-1uha8ag">`);
        ScoreBar($$renderer2, { value: c.score.total, label: c.name });
        $$renderer2.push(`<!----></td></tr>`);
      }
      $$renderer2.push(`<!--]--></tbody></table>`);
    }
    $$renderer2.push(`<!--]--></section></div>`);
  });
}
export {
  _page as default
};
