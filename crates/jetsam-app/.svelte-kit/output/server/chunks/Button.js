import { a5 as attr, a6 as attr_class, a7 as stringify } from "./index.js";
/* empty css                                     */
function Button($$renderer, $$props) {
  let {
    onclick,
    variant = "ghost",
    disabled = false,
    type = "button",
    children
  } = $$props;
  $$renderer.push(`<button${attr("type", type)}${attr_class(`btn ${stringify(variant)}`, "svelte-18sv61c")}${attr("disabled", disabled, true)}>`);
  children?.($$renderer);
  $$renderer.push(`<!----></button>`);
}
export {
  Button as B
};
