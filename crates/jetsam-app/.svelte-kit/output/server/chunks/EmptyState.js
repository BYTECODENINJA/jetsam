import { e as escape_html } from "./escaping.js";
import "clsx";
function EmptyState($$renderer, $$props) {
  let { title, message = "", children } = $$props;
  $$renderer.push(`<div class="empty svelte-13862ru"><h2 class="svelte-13862ru">${escape_html(title)}</h2> `);
  if (message) {
    $$renderer.push(`<!--[0--><p class="muted">${escape_html(message)}</p>`);
  } else {
    $$renderer.push("<!--[-1-->");
  }
  $$renderer.push(`<!--]--> `);
  children?.($$renderer);
  $$renderer.push(`<!----></div>`);
}
export {
  EmptyState as E
};
