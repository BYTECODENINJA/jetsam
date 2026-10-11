import { a6 as attr_class, a7 as stringify } from "./index.js";
import { e as escape_html } from "./escaping.js";
function Banner($$renderer, $$props) {
  let { tone = "info", title, children } = $$props;
  $$renderer.push(`<div${attr_class(`banner ${stringify(tone)}`, "svelte-6wdrfy")} role="status"><strong>${escape_html(title)}</strong> <div class="detail svelte-6wdrfy">`);
  children?.($$renderer);
  $$renderer.push(`<!----></div></div>`);
}
export {
  Banner as B
};
