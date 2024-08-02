import { focus } from "../focus.js";
import { add_css_editor } from "../editor.js";
import { move_cursor_to_end_of_element } from "../utils/contenteditable.js";
import { h } from "../html.js";

const { invoke } = window.__TAURI__.tauri;

let active_search_errors = [];

function push_errors(new_errors) {
  if (new_errors.every((e) => active_search_errors.includes(e))) return;
  active_search_errors = new_errors;

  let errors_toast = document.querySelector(".errors-toast-box");
  errors_toast.replaceChildren(
    ...active_search_errors.map((e) => {
      if (typeof e === "string") {
        return h.div(
          {
            class: "search-error",
            "@click"(_) {
              this.remove();
            },
          },
          e,
        );
      } else {
        let kind = Object.keys(e)[0];
        assert(Object.keys(e).length === 1);
        let value = e[kind];

        return h.div(
          {
            class: "search-error",
            "data-error-type": kind,
            "@click"(_) {
              this.remove();
            },
          },
          value,
        );
      }
    }),
  );
  setTimeout(() => {
    errors_toast.replaceChildren();
    active_search_errors = [];
  }, 5000);
}

document.addEventListener("DOMContentLoaded", (_) => {
  let input = document.querySelector(".search");
  let container = input.closest(".search-box");
  let options = document.createElement("div");
  options.classList.add("search-options");
  container.append(options);
  function clear() {
    input.classList.remove("active", "selector-search");
    input.innerText = "";
    options.innerHTML = "";
  }
  input.dataset.empty = true;

  window.addEventListener("keydown", async (e) => {
    if (e.key === "p" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      if (input.classList.contains("active")) {
        clear();
      } else {
        input.dataset.empty = true;
        input.classList.add("active", "selector-search");
        input.focus();
      }
    }
  });

  window.addEventListener("mousedown", (e) => {
    if (!e.target.closest(".search-box")) clear();
  });

  options.addEventListener("mousedown", async (e) => {
    if (e.button !== 0) return;
    if (!input.classList.contains("selector-search")) return;
    let selector = e.target.closest(".search-options > [data-kind]");
    if (!selector) return;
    await add_css_editor(selector.dataset.stringValue);
    clear();
  });

  input.addEventListener("keydown", async (e) => {
    if (!input.classList.contains("selector-search")) return;
    if (e.key === "Enter") {
      e.preventDefault();
      let candidate = options.querySelector(".candidate");

      if (!candidate) {
        let selector = input.innerText;
        // do we already have this rule? (remember that empty rules get filtered out from search)
        let existing_rule = document.querySelector(
          `.--editor:has([data-attr="selector"] > [data-kind][data-string-value="${CSS.escape(selector)}"]`,
        );

        if (existing_rule) {
          focus(existing_rule);
        } else {
          try {
            await invoke("insert_empty_rule", {
              path: localStorage.getItem("current-path"),
              selector,
            });
            await add_css_editor(selector);
          } catch (e) {
            console.log(e);
            throw e;
          }
        }
      } else {
        // do we already have this rule?
        let selector = candidate.dataset.stringValue;
        let existing_rule = document.querySelector(
          `.--editor:has([data-attr="selector"] > [data-kind][data-string-value="${CSS.escape(selector)}"]`,
        );

        if (existing_rule) {
          focus(existing_rule);
        } else {
          await add_css_editor(selector);
        }
      }

      clear();
    } else if (e.key === "Escape") {
      clear();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      let current_candidate = options.querySelector(".candidate");
      if (!current_candidate || !current_candidate.previousElementSibling)
        return;
      current_candidate.classList.remove("candidate");
      current_candidate.previousElementSibling.classList.add("candidate");
      input.innerText =
        current_candidate.previousElementSibling.dataset.stringValue;
      move_cursor_to_end_of_element(input);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      let current_candidate = options.querySelector(".candidate");
      if (!current_candidate) {
        let candidate = options.firstElementChild;
        candidate.classList.add("candidate");
        input.innerText = candidate.dataset.stringValue;
        move_cursor_to_end_of_element(input);
      } else {
        if (!current_candidate.nextElementSibling) return;
        current_candidate.classList.remove("candidate");
        current_candidate.nextElementSibling.classList.add("candidate");
        input.innerText =
          current_candidate.nextElementSibling.dataset.stringValue;
        move_cursor_to_end_of_element(input);
      }
    } else if (e.key === "Tab") {
      e.preventDefault();
      let candidate =
        options.querySelector(".candidate") ?? options.firstElementChild;
      let selector = candidate.dataset.stringValue
        .replaceAll(/\/\*.*\*\//g, "")
        .trim();

      input.innerText = selector;

      move_cursor_to_end_of_element(input);
      setTimeout(async () => {
        input.dataset.empty = input.innerText.trim().length === 0;
        // setTimeout so that innerText gets populated
        let results = await invoke("search", {
          path: localStorage.getItem("current-path"),
          q: input.innerText,
        });

        push_errors(results.errors);

        options.innerHTML = results.html;
      });
    } else {
      let old_text = input.innerText;
      setTimeout(async () => {
        input.dataset.empty = input.innerText.trim().length === 0;
        // setTimeout so that innerText gets populated
        if (input.innerText.trim() === "") {
          options.innerHTML = "";
        } else if (old_text !== input.innerText) {
          let results = await invoke("search", {
            path: localStorage.getItem("current-path"),
            q: input.innerText,
          });
          push_errors(results.errors);

          options.innerHTML = results.html;
        }
      });
    }
  });
});
