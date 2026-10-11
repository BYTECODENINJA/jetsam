import { a5 as attr, a3 as derived, a8 as attr_style, a7 as stringify, a4 as ensure_array_like, a6 as attr_class } from "../../../chunks/index.js";
import { e as escape_html } from "../../../chunks/escaping.js";
import { b as loadIgnored, u as usageFor, s as session, i as isIgnored, d as daysSince } from "../../../chunks/session.svelte.js";
import { a as api, r as repoLabel } from "../../../chunks/api.js";
import { f as formatBytes, b as formatDate, a as formatDays } from "../../../chunks/format.js";
import { B as Badge } from "../../../chunks/Badge.js";
import { B as Button } from "../../../chunks/Button.js";
import { E as EmptyState } from "../../../chunks/EmptyState.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let query = "";
    let sortKey = "name";
    let scrollTop = 0;
    let viewport = 600;
    const rowHeight = 42;
    let selected = /* @__PURE__ */ new Set();
    function rowValue(p) {
      switch (sortKey) {
        case "last_used":
          return usageFor(p.name)?.last_used_epoch ?? -1;
        case "launches":
          return usageFor(p.name)?.launches ?? -1;
        case "installed_size":
          return p.installed_size;
        case "install_date":
          return p.install_date ?? -1;
        case "reverse_dep_count":
          return p.reverse_dep_count;
        case "repo":
          return repoLabel(p.repo);
        case "version":
          return p.version;
        default:
          return p.name;
      }
    }
    const rows = derived(() => {
      const all = session.packages ?? [];
      const q = query.trim().toLowerCase();
      const filtered = q ? all.filter((p) => p.name.toLowerCase().includes(q) || (p.description ?? "").toLowerCase().includes(q)) : all;
      return [...filtered].sort((a, b) => {
        const va = rowValue(a);
        const vb = rowValue(b);
        const cmp = typeof va === "number" && typeof vb === "number" ? va - vb : String(va).localeCompare(String(vb));
        return cmp;
      });
    });
    const total = derived(() => rows().length);
    const start = derived(() => Math.max(0, Math.floor(scrollTop / rowHeight) - 6));
    const visibleCount = derived(() => Math.ceil(viewport / rowHeight) + 14);
    const visible = derived(() => rows().slice(start(), start() + visibleCount()));
    const padTop = derived(() => start() * rowHeight);
    const padBottom = derived(() => Math.max(0, (total() - start() - visible().length) * rowHeight));
    async function ignoreSelected(ignored) {
      for (const name of selected) {
        await api.setIgnored(name, ignored);
      }
      selected = /* @__PURE__ */ new Set();
      await loadIgnored();
    }
    $$renderer2.push(`<div class="page"><div class="spread"><h1>Packages</h1> <div class="row"><input class="search svelte-disfw2" type="search" placeholder="Search or filter packages"${attr("value", query)} aria-label="Search packages"/> `);
    Badge($$renderer2, { text: `${total()} shown` });
    $$renderer2.push(`<!----></div></div> `);
    if (selected.size > 0) {
      $$renderer2.push(`<!--[0--><div class="card bulk svelte-disfw2"><span class="grow">${escape_html(selected.size)} selected</span> `);
      Button($$renderer2, {
        variant: "ghost",
        onclick: () => ignoreSelected(true),
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->Ignore`);
        }
      });
      $$renderer2.push(`<!----> `);
      Button($$renderer2, {
        variant: "ghost",
        onclick: () => ignoreSelected(false),
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->Unignore`);
        }
      });
      $$renderer2.push(`<!----> `);
      Button($$renderer2, {
        variant: "ghost",
        onclick: () => selected = /* @__PURE__ */ new Set(),
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->Clear`);
        }
      });
      $$renderer2.push(`<!----></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (!session.packages) {
      $$renderer2.push("<!--[0-->");
      EmptyState($$renderer2, {
        title: "Loading inventory",
        message: "Reading the package database…"
      });
    } else if (total() === 0) {
      $$renderer2.push("<!--[1-->");
      EmptyState($$renderer2, { title: "No matches", message: "Try a different search." });
    } else {
      $$renderer2.push(`<!--[-1--><div class="table card svelte-disfw2"><div class="tr head svelte-disfw2"><span class="c-sel svelte-disfw2"></span> <button class="th c-name svelte-disfw2">Package</button> <button class="th c-ver svelte-disfw2">Version</button> <button class="th c-repo svelte-disfw2">Repo</button> <button class="th c-num svelte-disfw2">Size</button> <button class="th c-num svelte-disfw2">Installed</button> <button class="th c-num svelte-disfw2">Last used</button> <button class="th c-num svelte-disfw2">Launches</button> <button class="th c-num svelte-disfw2">Revdeps</button></div> <div class="viewport svelte-disfw2"><div${attr_style(`height: ${stringify(padTop())}px`)}></div> <!--[-->`);
      const each_array = ensure_array_like(visible());
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let pkg = each_array[$$index];
        $$renderer2.push(`<div${attr_class("tr svelte-disfw2", void 0, { "sel": selected.has(pkg.name) })}><span class="c-sel svelte-disfw2"><input type="checkbox"${attr("checked", selected.has(pkg.name), true)}${attr("aria-label", `Select ${stringify(pkg.name)}`)}/></span> <span class="c-name svelte-disfw2"><a${attr("href", `/package?name=${stringify(pkg.name)}`)}>${escape_html(pkg.name)}</a> `);
        if (pkg.protected) {
          $$renderer2.push("<!--[0-->");
          Badge($$renderer2, { tone: "caution", text: "protected" });
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (isIgnored(pkg.name)) {
          $$renderer2.push("<!--[0-->");
          Badge($$renderer2, { text: "ignored" });
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--></span> <span class="c-ver dim mono svelte-disfw2">${escape_html(pkg.version)}</span> <span class="c-repo">`);
        Badge($$renderer2, {
          tone: pkg.repo === "foreign" || typeof pkg.repo === "object" ? "caution" : "neutral",
          text: repoLabel(pkg.repo)
        });
        $$renderer2.push(`<!----></span> <span class="c-num tnum svelte-disfw2">${escape_html(formatBytes(pkg.installed_size))}</span> <span class="c-num svelte-disfw2">${escape_html(formatDate(pkg.install_date))}</span> <span class="c-num svelte-disfw2">${escape_html(formatDays(daysSince(usageFor(pkg.name)?.last_used_epoch)))}</span> <span class="c-num tnum svelte-disfw2">${escape_html(usageFor(pkg.name)?.launches ?? "—")}</span> <span class="c-num tnum svelte-disfw2">${escape_html(pkg.reverse_dep_count)}</span></div>`);
      }
      $$renderer2.push(`<!--]--> <div${attr_style(`height: ${stringify(padBottom())}px`)}></div></div></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
