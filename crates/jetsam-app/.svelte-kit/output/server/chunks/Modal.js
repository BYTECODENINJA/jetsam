import { a5 as attr } from "./index.js";
import { e as escape_html } from "./escaping.js";
function Modal($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { open, title, onclose, children } = $$props;
    if (open) {
      $$renderer2.push(`<!--[0--><div class="backdrop svelte-ta60gp" role="presentation"><div class="modal card svelte-ta60gp" role="dialog" aria-modal="true"${attr("aria-label", title)} tabindex="-1"><div class="spread"><h2>${escape_html(title)}</h2> <button class="x svelte-ta60gp" type="button" aria-label="Close">✕</button></div> <div class="body svelte-ta60gp">`);
      children?.($$renderer2);
      $$renderer2.push(`<!----></div></div></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]-->`);
  });
}
export {
  Modal as M
};
