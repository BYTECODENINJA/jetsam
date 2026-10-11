import { a6 as attr_class, a5 as attr } from "./index.js";
function Switch($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { checked, onchange, label = "" } = $$props;
    $$renderer2.push(`<button type="button"${attr_class("switch svelte-1eq0goo", void 0, { "on": checked })} role="switch"${attr("aria-checked", checked)}${attr("aria-label", label)}><span class="knob svelte-1eq0goo"></span></button>`);
  });
}
export {
  Switch as S
};
