import { a4 as ensure_array_like, a3 as derived } from "../../../chunks/index.js";
import { e as escape_html } from "../../../chunks/escaping.js";
import { r as refreshStatus, s as session } from "../../../chunks/session.svelte.js";
import { a as api } from "../../../chunks/api.js";
import { B as Button } from "../../../chunks/Button.js";
import { S as Switch } from "../../../chunks/Switch.js";
import { B as Banner } from "../../../chunks/Banner.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let busy = false;
    const optedIn = derived(() => session.status?.daemon?.opted_in ?? false);
    async function toggle(next) {
      busy = true;
      try {
        await api.setOptIn(next);
        await api.startDaemon().catch(() => {
        });
        await refreshStatus();
        window.location.href = "/";
      } catch (e) {
        session.error = String(e);
      } finally {
        busy = false;
      }
    }
    const records = [
      [
        "Executable path",
        "Which binary a launch resolved to, so a package can be credited."
      ],
      ["Launch count", "How many times that executable started."],
      [
        "Cumulative CPU time",
        "CPU seconds a process tree used, summed per package."
      ],
      [
        "Coarse memory sample",
        "Approximate resident memory, at a low interval — never contents."
      ],
      [
        "Timestamps",
        "First seen, last seen, so staleness can be computed."
      ]
    ];
    const never = [
      "Command-line arguments",
      "Environment variables",
      "Keystrokes or input",
      "Window titles or screen content",
      "File contents or names you open"
    ];
    $$renderer2.push(`<div class="page narrow svelte-fpvdp2"><h1>Welcome to Jetsam</h1> <p class="lead svelte-fpvdp2">Jetsam shows what you actually use, then helps you reclaim space safely. It needs a
    small amount of local, opt-in usage data to do that.</p> <section class="card svelte-fpvdp2"><h2>What is recorded</h2> <p class="muted">Only for your own processes, only when enabled, only stored here on this machine.</p> <ul class="records svelte-fpvdp2"><!--[-->`);
    const each_array = ensure_array_like(records);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let [what, why] = each_array[$$index];
      $$renderer2.push(`<li class="svelte-fpvdp2"><strong>${escape_html(what)}</strong><span class="muted svelte-fpvdp2">${escape_html(why)}</span></li>`);
    }
    $$renderer2.push(`<!--]--></ul></section> <section class="card svelte-fpvdp2"><h2>What is never recorded</h2> <ul class="never svelte-fpvdp2"><!--[-->`);
    const each_array_1 = ensure_array_like(never);
    for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
      let item = each_array_1[$$index_1];
      $$renderer2.push(`<li>${escape_html(item)}</li>`);
    }
    $$renderer2.push(`<!--]--></ul></section> <section class="card svelte-fpvdp2"><h2>Where it is stored</h2> <ul class="stats svelte-fpvdp2"><li class="svelte-fpvdp2"><span>Data (samples)</span><span class="mono">~/.local/share/jetsam</span></li> <li class="svelte-fpvdp2"><span>Configuration</span><span class="mono">~/.config/jetsam</span></li> <li class="svelte-fpvdp2"><span>Socket (runtime)</span><span class="mono">$XDG_RUNTIME_DIR/jetsam</span></li></ul> <p class="muted">No telemetry, no network access. Erase everything at any time from Settings.</p></section> `);
    if (session.error) {
      $$renderer2.push("<!--[0-->");
      Banner($$renderer2, {
        tone: "risk",
        title: "Could not start",
        children: ($$renderer3) => {
          $$renderer3.push(`<!---->${escape_html(session.error)}`);
        }
      });
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="actions"><div class="row">`);
    Switch($$renderer2, {
      checked: optedIn(),
      label: "Enable usage recording",
      onchange: toggle
    });
    $$renderer2.push(`<!----> <span>${escape_html(optedIn() ? "Enabled" : "Disabled")}</span></div> `);
    Button($$renderer2, {
      variant: "primary",
      disabled: busy,
      onclick: () => toggle(true),
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->${escape_html(busy ? "Starting…" : "Enable and continue")}`);
      }
    });
    $$renderer2.push(`<!----></div> <button class="skip" type="button">Skip — use metadata only</button></div>`);
  });
}
export {
  _page as default
};
