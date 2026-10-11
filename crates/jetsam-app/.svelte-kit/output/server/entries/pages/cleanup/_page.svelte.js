import { a5 as attr, a4 as ensure_array_like, a6 as attr_class, a7 as stringify, a3 as derived } from "../../../chunks/index.js";
import { e as escape_html } from "../../../chunks/escaping.js";
import { s as session, l as loadInventory, a as loadCleanup } from "../../../chunks/session.svelte.js";
import { a as api } from "../../../chunks/api.js";
import { f as formatBytes, a as formatDays } from "../../../chunks/format.js";
import { B as Badge } from "../../../chunks/Badge.js";
import { B as Button } from "../../../chunks/Button.js";
import { S as ScoreBar } from "../../../chunks/ScoreBar.js";
import { M as Modal } from "../../../chunks/Modal.js";
import { E as EmptyState } from "../../../chunks/EmptyState.js";
import { B as Banner } from "../../../chunks/Banner.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let selected = /* @__PURE__ */ new Set();
    let minScore = 0;
    let showProtected = false;
    let confirmOpen = false;
    let plan = null;
    let outcome = null;
    let busy = false;
    const cleanup = derived(() => session.cleanup);
    const candidates = derived(() => (cleanup()?.candidates ?? []).filter((c) => c.score.total >= minScore));
    const selectedCandidates = derived(() => (cleanup()?.candidates ?? []).filter((c) => selected.has(c.name)));
    const selectedFreed = derived(() => selectedCandidates().reduce((sum, c) => sum + Math.max(0, c.reclaimable_bytes), 0));
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
          selected = /* @__PURE__ */ new Set();
          await loadInventory(true);
          await loadCleanup();
        }
        confirmOpen = false;
      } catch (e) {
        outcome = {
          action: "remove",
          ok: false,
          exit_code: null,
          stdout: "",
          stderr: String(e)
        };
      } finally {
        busy = false;
      }
    }
    $$renderer2.push(`<div class="page"><div class="spread"><h1>Cleanup</h1> <button class="refresh svelte-ivdpw3" type="button"${attr("disabled", busy, true)}>Recompute</button></div> `);
    if (cleanup() && !cleanup().usage_known) {
      $$renderer2.push("<!--[0-->");
      Banner($$renderer2, {
        tone: "info",
        title: "Usage still accumulating",
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->These suggestions rank mostly by size, explicitness and risk — staleness is not
      yet known. Revisit after a couple of weeks of data.`);
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
        title: outcome.ok ? "Removal completed" : "Removal failed",
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
    if (!cleanup()) {
      $$renderer2.push("<!--[0-->");
      EmptyState($$renderer2, { title: "Analyzing…" });
    } else if (candidates().length === 0) {
      $$renderer2.push("<!--[1-->");
      EmptyState($$renderer2, {
        title: "Nothing to suggest",
        message: "No cleanup candidates above the current threshold."
      });
    } else {
      $$renderer2.push(`<!--[-1--><div class="toolbar card svelte-ivdpw3"><label class="row svelte-ivdpw3"><input type="checkbox"${attr("checked", selected.size === candidates().length && candidates().length > 0, true)}${attr("indeterminate", selected.size > 0 && selected.size < candidates().length, true)}/> <span>${escape_html(selected.size)} selected · ${escape_html(formatBytes(selectedFreed()))} reclaimable</span></label> <label class="row dim svelte-ivdpw3">Min score <input type="range" min="0" max="90" step="5"${attr("value", minScore)}/> <span class="tnum">${escape_html(minScore)}</span></label> `);
      Button($$renderer2, {
        variant: "primary",
        disabled: selected.size === 0 || busy,
        onclick: openConfirm,
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->Review &amp; remove`);
        }
      });
      $$renderer2.push(`<!----></div> <div class="list card svelte-ivdpw3"><!--[-->`);
      const each_array = ensure_array_like(candidates());
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let c = each_array[$$index];
        $$renderer2.push(`<label${attr_class("cand svelte-ivdpw3", void 0, { "sel": selected.has(c.name) })}><input type="checkbox"${attr("checked", selected.has(c.name), true)}${attr("aria-label", `Select ${stringify(c.name)}`)}/> <div class="info svelte-ivdpw3"><div class="row wrap"><a class="mono name svelte-ivdpw3"${attr("href", `/package?name=${stringify(c.name)}`)}>${escape_html(c.name)}</a> `);
        if (c.days_since_last_use == null) {
          $$renderer2.push("<!--[0-->");
          Badge($$renderer2, { text: "usage unknown" });
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (c.exclusive_orphans.length > 0) {
          $$renderer2.push("<!--[0-->");
          Badge($$renderer2, {
            tone: "neutral",
            text: `+${c.exclusive_orphans.length} orphan${c.exclusive_orphans.length === 1 ? "" : "s"}`
          });
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (c.breaks.length > 0) {
          $$renderer2.push("<!--[0-->");
          Badge($$renderer2, { tone: "risk", text: `breaks ${c.breaks.length}` });
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--></div> <div class="dim meta svelte-ivdpw3">${escape_html(formatBytes(c.reclaimable_bytes))} · last used ${escape_html(formatDays(c.days_since_last_use))} ·
              ${escape_html(c.reverse_dep_count)} revdep${escape_html(c.reverse_dep_count === 1 ? "" : "s")}</div></div> <div class="score svelte-ivdpw3">`);
        ScoreBar($$renderer2, { value: c.score.total, label: c.name });
        $$renderer2.push(`<!----> <button class="ignore svelte-ivdpw3" type="button">ignore</button></div></label>`);
      }
      $$renderer2.push(`<!--]--></div> <details class="protected card svelte-ivdpw3"${attr("open", showProtected, true)}><summary class="svelte-ivdpw3">Protected packages (${escape_html(cleanup().protected.length)})</summary> <p class="dim">Jetsam never suggests removing these.</p> <div class="chips svelte-ivdpw3"><!--[-->`);
      const each_array_1 = ensure_array_like(cleanup().protected);
      for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
        let p = each_array_1[$$index_1];
        $$renderer2.push(`<span class="chip svelte-ivdpw3"${attr("title", p.reason)}><span class="mono">${escape_html(p.name)}</span> · <span class="dim">${escape_html(p.reason)}</span></span>`);
      }
      $$renderer2.push(`<!--]--></div></details>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    Modal($$renderer2, {
      title: "Confirm removal",
      open: confirmOpen,
      onclose: () => confirmOpen = false,
      children: ($$renderer3) => {
        if (plan) {
          $$renderer3.push(`<!--[0--><p>This will free <strong class="tnum">${escape_html(formatBytes(plan.freed_bytes))}</strong>.</p> <p class="muted">${escape_html(plan.remove.length)} package(s) removed `);
          if (plan.exclusive_orphans.length > 0) {
            $$renderer3.push(`<!--[0-->, including ${escape_html(plan.exclusive_orphans.length)} exclusive orphan(s)`);
          } else {
            $$renderer3.push("<!--[-1-->");
          }
          $$renderer3.push(`<!--]-->.</p> `);
          if (plan.breaks.length > 0) {
            $$renderer3.push("<!--[0-->");
            Banner($$renderer3, {
              tone: "risk",
              title: "Dependency conflict",
              children: ($$renderer4) => {
                $$renderer4.push(`<!---->${escape_html(plan.breaks.length)} retained package(s) depend on this set; pacman will refuse the transaction.`);
              }
            });
          } else {
            $$renderer3.push("<!--[-1-->");
          }
          $$renderer3.push(`<!--]--> <ul class="removeList mono svelte-ivdpw3"><!--[-->`);
          const each_array_2 = ensure_array_like(plan.remove.slice(0, 40));
          for (let $$index_2 = 0, $$length = each_array_2.length; $$index_2 < $$length; $$index_2++) {
            let n = each_array_2[$$index_2];
            $$renderer3.push(`<li>${escape_html(n)}</li>`);
          }
          $$renderer3.push(`<!--]--> `);
          if (plan.remove.length > 40) {
            $$renderer3.push(`<!--[0--><li class="dim">+${escape_html(plan.remove.length - 40)} more</li>`);
          } else {
            $$renderer3.push("<!--[-1-->");
          }
          $$renderer3.push(`<!--]--></ul> <div class="row end svelte-ivdpw3">`);
          Button($$renderer3, {
            variant: "ghost",
            onclick: () => confirmOpen = false,
            children: ($$renderer4) => {
              $$renderer4.push(`<!---->Cancel`);
            }
          });
          $$renderer3.push(`<!----> `);
          Button($$renderer3, {
            variant: "danger",
            disabled: busy,
            onclick: apply,
            children: ($$renderer4) => {
              $$renderer4.push(`<!---->${escape_html(busy ? "Removing…" : "Remove (asks for password)")}`);
            }
          });
          $$renderer3.push(`<!----></div>`);
        } else {
          $$renderer3.push(`<!--[-1--><p>Computing plan…</p>`);
        }
        $$renderer3.push(`<!--]-->`);
      }
    });
    $$renderer2.push(`<!---->`);
  });
}
export {
  _page as default
};
