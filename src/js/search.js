import { focus } from "./focus.js";
import { add_editor } from "./editor.js";

const { invoke } = window.__TAURI__.tauri;

document.addEventListener("DOMContentLoaded", (_) => {
  let input = document.querySelector(".search");
  let container = input.closest(".search-box");
  let options = document.createElement("div");
  options.classList.add("search-options");
  container.append(options);
  function clear() {
    input.classList.remove("active");
    input.innerText = "";
    options.innerHTML = "";
  }

  window.addEventListener("keydown", async (e) => {
    if (e.key === "p" && e.metaKey) {
      e.preventDefault();
      if (input.classList.contains("active")) {
        clear();
      } else {
        input.classList.add("active");
        input.focus();
      }
    }
  });

  window.addEventListener("mousedown", (e) => {
    if (!e.target.closest(".search-box")) clear();
  });

  options.addEventListener("mousedown", async (e) => {
    if (e.button !== 0) return;
    let selector = e.target.closest(".search-options > [data-kind]");
    if (!selector) return;
    await add_editor(selector.dataset.stringValue);
    clear();
  });

  input.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      let candidate = options.querySelector(".candidate");

      if (!candidate) {
        let selector = input.innerText;
        // do we already have this rule? (remember that empty rules get filtered out from search)
        let existing_rule = document.querySelector(
          `.--editor:has([data-attr="selector"] > [data-kind][data-string-value="${selector.replaceAll('"', '\\"')}"]`,
        );

        if (existing_rule) {
          focus(existing_rule);
        } else {
          await invoke("insert_empty_rule", {
            path: localStorage.getItem("current-path"),
            selector,
          });
          await add_editor(selector);
        }
      } else {
        // do we already have this rule?
        let selector = candidate.dataset.stringValue;
        let existing_rule = document.querySelector(
          `.--editor:has([data-attr="selector"] > [data-kind][data-string-value="${selector}"]`,
        );

        if (existing_rule) {
          focus(existing_rule);
        } else {
          await add_editor(selector);
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
      if (!current_candidate) current_candidate = options.firstElementChild;
      if (!current_candidate.nextElementSibling) return;
      current_candidate.classList.remove("candidate");
      current_candidate.nextElementSibling.classList.add("candidate");
      input.innerText =
        current_candidate.nextElementSibling.dataset.stringValue;
      move_cursor_to_end_of_element(input);
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
        // setTimeout so that innerText gets populated
        let results = await invoke("search", {
          path: localStorage.getItem("current-path"),
          q: input.innerText,
        });
        options.innerHTML = results.slice(0, 20).join("");
      });
    } else {
      let old_text = input.innerText;
      setTimeout(async () => {
        // setTimeout so that innerText gets populated
        if (input.innerText.trim() === "") {
          options.innerHTML = "";
        } else if (old_text !== input.innerText) {
          let results = await invoke("search", {
            path: localStorage.getItem("current-path"),
            q: input.innerText,
          });

          options.innerHTML = results.slice(0, 20).join("");
        }
      });
    }
  });
});
