import { h } from "./html.js";

document.addEventListener("DOMContentLoaded", (_) => {
  document.body.append(
    h(
      "button",
      {
        class: "open-help",
        "@click"(_) {
          this.classList.toggle("active");
        },
      },
      "help",
    ),
  );
  document.body.append(
    h(
      "article",
      { class: "help-modal" },
      h(
        "button",
        {
          class: "close",
          "@click"(_) {
            document.querySelector("button.open-help").click();
          },
        },
        "close",
      ),
      h("h2", {}, "shortcuts"),
      h(
        "ul",
        {},
        h("li", {}, "cmd+p => search"),
        h("li", {}, "/ => insert property"),
        h("li", {}, "delete => remove selected property"),
      ),
    ),
  );
});
