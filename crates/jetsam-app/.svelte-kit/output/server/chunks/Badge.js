import { a6 as attr_class, a7 as stringify } from "./index.js";
/* empty css                                    */
import { e as escape_html } from "./escaping.js";
function Badge($$renderer, $$props) {
  let { text, tone = "neutral" } = $$props;
  $$renderer.push(`<span${attr_class(`badge ${stringify(tone)}`, "svelte-dtbgkf")}>${escape_html(text)}</span>`);
}
export {
  Badge as B
};
