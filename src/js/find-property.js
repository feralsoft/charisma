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

    await invoke("insert_empty_rule", {
      path: localStorage.getItem("current-path"),
      selector,
    });
    await add_editor(selector);
    clear();
  }

  function on_keydown(e) {
    input.dataset.empty = false;
    if (!input.classList.contains("find-property")) return;
    if (e.key === "Escape" || (e.key == "p" && e.ctrlKey)) {
      clear();
    } else {
      setTimeout(async () => {
        // setTimeout so that innerText gets populated
        if (input.innerText.trim() === "") {
          options.innerHTML = "";
          input.dataset.empty = true;
        } else {
          let properties = await invoke("find_property", {
            path: localStorage.getItem("current-path"),
            q: input.textContent.trim(),
          });

          options.innerHTML = properties
            .map(
              ([property, selector]) =>
                `<div class='find-search-result'>
                  <div class='property'>${property}</div>
                  <div class='selector'>${selector}</div>
                </div>
                `,
            )
            .join("");
        }
      });
    }
  }

  input.addEventListener("keydown", on_keydown);
  options.addEventListener("click", on_click);
}

window.addEventListener("keydown", (e) => {
  if (e.target.closest(".--editor-group")) return;
  if (e.key === "f" && e.ctrlKey) {
    e.preventDefault();
    start_search();
  }
});
