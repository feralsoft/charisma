const { invoke } = window.__TAURI__.tauri;
import { add_editor } from "./editor.js";

function start_search() {
  let input = document.querySelector(".search");
  let options = input.closest(".search-box").querySelector(".search-options");

  input.classList.add("active", "find-property");
  input.focus();

  input.dataset.empty = true;

  function clear() {
    input.classList.remove("active", "find-property");
    input.innerText = "";
    options.innerHTML = "";
    input.removeEventListener("keydown", on_keydown);
    options.removeEventListener("click", on_click);
  }

  async function on_click(e) {
    let result = e.target.closest(".find-search-result");
    if (!result) return;

    let selector = result.querySelector(".selector > [data-kind]").dataset
      .stringValue;

    await add_editor(selector);
    clear();
  }

  function on_keydown(e) {
    input.dataset.empty = false;
    if (!input.classList.contains("find-property")) return;
    if (e.key === "Escape" || (e.key == "p" && (e.metaKey || e.ctrlKey))) {
      clear();
    } else {
      setTimeout(async () => {
        // setTimeout so that innerText gets populated
        if (input.innerText.trim() === "") {
          options.innerHTML = "";
          input.dataset.empty = true;
        } else {
          let offset = 0;
          let [properties, total] = await invoke("find_property", {
            path: localStorage.getItem("current-path"),
            q: input.textContent.trim(),
            offset,
          });

          let load_more_results = document.createElement("button");
          load_more_results.classList.add("load-more-result");
          load_more_results.innerText = "load more";
          load_more_results.addEventListener("mousedown", async function (_) {
            offset += 50;
            let [properties, total] = await invoke("find_property", {
              path: localStorage.getItem("current-path"),
              q: input.textContent.trim(),
              offset,
            });
            options.removeChild(this);
            options.innerHTML += properties
              .map(
                ([property, selector]) =>
                  // todo: handle errors
                  `<div class='find-search-result'>
                  <div class='property'>${property.html}</div>
                  <div class='selector'>${selector.html}</div>
                </div>
                `,
              )
              .join("");

            if (total - offset > 50) options.append(this);
          });

          options.innerHTML = properties
            .map(
              ([property, selector]) =>
                // todo: handle errors
                `<div class='find-search-result'>
                  <div class='property'>${property.html}</div>
                  <div class='selector'>${selector.html}</div>
                </div>
                `,
            )
            .join("");
          if (total > 50) options.append(load_more_results);
        }
      });
    }
  }

  input.addEventListener("keydown", on_keydown);
  options.addEventListener("click", on_click);
}

window.addEventListener("keydown", (e) => {
  if (e.target.closest(".--editor-group")) return;
  if (e.key === "f" && (e.metaKey || e.ctrlKey)) {
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();
    start_search();
  }
});
