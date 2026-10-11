import "clsx";
import "@tauri-apps/api/core";
/* empty css                                                  */
/* empty css                                                   */
import { E as EmptyState } from "../../../chunks/EmptyState.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    $$renderer2.push(`<div class="page">`);
    {
      $$renderer2.push("<!--[0-->");
      EmptyState($$renderer2, { title: "Loading…" });
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
