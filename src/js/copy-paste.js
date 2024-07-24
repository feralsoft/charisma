import { add_editor } from "./editor.js";

const { invoke } = window.__TAURI__.tauri;

let currently_selected,
  originally_selected,
  is_selecting = false,
  is_pressing_shift_key = false;

window.addEventListener("keydown", (e) => {
  is_pressing_shift_key = e.shiftKey;
});

window.addEventListener("keyup", (_) => {
  is_pressing_shift_key = false;
});

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  currently_selected = null;

  document
    .querySelector("[data-kind][data-string-value].selected")
    ?.classList?.remove("selected");

  if (!is_pressing_shift_key) return;
  if (!e.target.closest('[data-attr="selector"]')) return;
  select(e.target.closest("[data-kind][data-string-value]"));
  originally_selected = currently_selected;
});

function select(element) {
  is_selecting = true;
  document
    .querySelector("[data-kind][data-string-value].selected")
    ?.classList?.remove("selected");
  currently_selected = element;
  currently_selected.classList.add("selected");
}

window.addEventListener("mousemove", (e) => {
  if (!is_selecting) return;

  for (let part of document.elementsFromPoint(e.clientX, e.clientY)) {
    if (!part.matches("[data-kind][data-string-value]")) continue;
    // first see if its closer to the original (this allows us to go backwards)
    if (part.contains(originally_selected) || part === originally_selected) {
      select(part);
      break;
      // then see if its a parent
    } else if (part.contains(currently_selected)) {
      select(part);
      break;
    }
  }
});

window.addEventListener("mouseup", (_) => {
  is_selecting = false;
});

let copied = false;

window.addEventListener("keydown", async (e) => {
  if (!currently_selected) return;
  if (e.key === "c" && e.metaKey) {
    currently_selected.classList.add("copied");
    copied = true;
  } else if (e.key === "v" && e.metaKey && copied) {
    currently_selected.classList.remove("copied");
    currently_selected.classList.remove("selected");
    copied = false;
    let selector = currently_selected.dataset.stringValue;
    if (e.target.matches(".search.active")) {
      // pasting into search
      e.preventDefault();
      e.target.innerText = selector;
    } else {
      let current_group = currently_selected.closest(".--editor-group");
      await invoke("insert_empty_rule", {
        path: localStorage.getItem("current-path"),
        selector,
      });
      let editor = await add_editor(selector, current_group);
    }
  }
});

// paste from out of the app
window.addEventListener("paste", async (e) => {
  if (currently_selected) return;
  let rule = e.clipboardData.getData("text");
  if (!rule) return;
  let selector = await invoke("load_rule", {
    path: localStorage.getItem("current-path"),
    rule,
  });
  await add_editor(selector);
});
