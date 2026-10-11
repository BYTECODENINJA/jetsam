import { a4 as ensure_array_like, a5 as attr, a6 as attr_class } from "../../chunks/index.js";
import { e as escape_html } from "../../chunks/escaping.js";
import { s as session } from "../../chunks/session.svelte.js";
import "@tauri-apps/api/core";
import { B as Banner } from "../../chunks/Banner.js";
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { children } = $$props;
    const items = [
      { href: "/", label: "Dashboard" },
      { href: "/packages", label: "Packages" },
      { href: "/cleanup", label: "Cleanup" },
      { href: "/updates", label: "Updates" },
      { href: "/settings", label: "Settings" },
      { href: "/onboarding", label: "Onboarding" }
    ];
    let path = "/";
    $$renderer2.push(`<div class="shell svelte-12qhfyh"><aside class="sidebar svelte-12qhfyh"><div class="brand svelte-12qhfyh"><span class="logo svelte-12qhfyh">J</span> <span class="name svelte-12qhfyh">Jetsam</span></div> <nav class="svelte-12qhfyh"><!--[-->`);
    const each_array = ensure_array_like(items);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let item = each_array[$$index];
      $$renderer2.push(`<a${attr("href", item.href)}${attr_class("svelte-12qhfyh", void 0, { "active": path === item.href })}>${escape_html(item.label)}</a>`);
    }
    $$renderer2.push(`<!--]--></nav></aside> <div class="main svelte-12qhfyh"><header class="topbar svelte-12qhfyh"><div class="row">`);
    if (session.status?.daemon) {
      $$renderer2.push(`<!--[0--><span class="chip ok svelte-12qhfyh">daemon v${escape_html(session.status.daemon.daemon_version)}</span>`);
    } else {
      $$renderer2.push(`<!--[-1--><span class="chip off svelte-12qhfyh">daemon offline</span>`);
    }
    $$renderer2.push(`<!--]--> `);
    if (session.status?.packages_loaded) {
      $$renderer2.push(`<!--[0--><span class="chip svelte-12qhfyh">${escape_html(session.status.packages)} packages</span>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div> <div class="row"><button class="theme svelte-12qhfyh" type="button" aria-label="Toggle theme">${escape_html("☾")}</button></div></header> <main class="content svelte-12qhfyh">`);
    if (!session.status?.daemon) {
      $$renderer2.push(`<!--[0--><div class="page-flush svelte-12qhfyh">`);
      Banner($$renderer2, {
        tone: "caution",
        title: "Usage daemon is not running",
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->Usage tracking is offline. Package data still works from metadata. → <button class="link svelte-12qhfyh" type="button">Start daemon</button>`);
        }
      });
      $$renderer2.push(`<!----></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    children?.($$renderer2);
    $$renderer2.push(`<!----></main></div></div>`);
  });
}
export {
  _layout as default
};
