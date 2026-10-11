import { a5 as attr, a6 as attr_class, a8 as attr_style, a3 as derived, a7 as stringify } from "./index.js";
import { e as escape_html } from "./escaping.js";
function ScoreBar($$renderer, $$props) {
  let { value, label = "" } = $$props;
  const tone = derived(() => value >= 66 ? "risk" : value >= 33 ? "caution" : "safe");
  $$renderer.push(`<div class="scorebar svelte-1szkqx" role="img"${attr("aria-label", `${stringify(label)} score ${stringify(value)} of 100`)}><div class="track svelte-1szkqx"><div${attr_class(`fill ${tone()}`, "svelte-1szkqx")}${attr_style(`width: ${stringify(Math.max(0, Math.min(100, value)))}%`)}></div></div> <span class="num tnum svelte-1szkqx">${escape_html(value)}</span></div>`);
}
export {
  ScoreBar as S
};
