const { invoke } = window.__TAURI__.tauri;

let currently_selected,
  originally_selected,
  is_selecting = false;

window.addEventListener("mousedown", (e) => {
  currently_selected = null;

  document
    .querySelector("[data-kind][data-string-value].selected")
    ?.classList?.remove("selected");

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
    await invoke("insert_empty_rule", {
      path: localStorage.getItem("current-path"),
      selector,
    });
    await add_editor(selector);
  }
});
