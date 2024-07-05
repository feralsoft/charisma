import { h } from "./html.js";

const { invoke } = window.__TAURI__.tauri;

let all_properties;

let search_item = (name) => h("div", { class: "search-item" }, name);

function search_options(names) {
  // hack
  if (names.length === 0) return document.createElement("div");

  let options = h(
    "div",
    { class: "search-options" },
    ...names.map(search_item),
  );
  options.firstElementChild.classList.add("candidate");
  return options;
}

window.move_cursor_to_end_of_element = function (element) {
  // start garbage internet code to go the end of a text range
  let range = document.createRange();
  let selection = window.getSelection();
  range.setStart(element, element.childNodes.length);
  range.collapse(true);
  selection.removeAllRanges();
  selection.addRange(range);
  // end of garbage internet code
};

function accept_candidate(container, input_elem) {
  let options = container.querySelector(".search-options");
  input_elem.innerText = options.querySelector(".candidate").innerText + ":";
  options.remove();

  move_cursor_to_end_of_element(input_elem);
}

let input = (editor) =>
  h("div", {
    class: "input",
    contenteditable: true,
    placeholder: "insert property...",
    async "@keydown"(e) {
      let container = this.closest(".insert-property-container");
      if (e.key === "Enter") {
        e.preventDefault();
        // if there's a candidate auto complete it
        // important! this does not mean it submits something
        if (container.querySelector(".search-options .candidate")) {
          return accept_candidate(container, this);
        } else {
          // otherwise submit & reload
          await invoke("insert_property", {
            selector: editor.dataset.selector,
            property: e.target.innerText,
          });
          this.dispatchEvent(new Event("reload", { bubbles: true }));
        }
      } else if (e.key === "Escape") {
        this.blur();
      } else if (e.key === "ArrowUp") {
        // go up in search options
        e.preventDefault();
        let options = container.querySelector(".search-options");
        let elem = options.querySelector(".candidate");
        if (elem.previousElementSibling) {
          elem.classList.remove("candidate");
          elem.previousElementSibling.classList.add("candidate");
        }
      } else if (e.key === "ArrowDown") {
        // go down in search options
        e.preventDefault();
        let options = container.querySelector(".search-options");
        let elem = options.querySelector(".candidate");
        if (elem.nextElementSibling) {
          elem.classList.remove("candidate");
          elem.nextElementSibling.classList.add("candidate");
        }
      } else if (e.key === "Tab") {
        accept_candidate(container, this);
        e.preventDefault();
      } else if (!e.shiftKey) {
        // populate auto complete list
        // setTimeout is needed so that `this.innerText` gets populated :facepalm:
        setTimeout(() => {
          container.querySelector(".search-options")?.remove();
          let text = this.innerText.trim();
          if (text === "") return;
          let options = all_properties.filter((name) => name.includes(text));
          options.sort((a, b) => {
            if (a.startsWith(text)) {
              if (b.startsWith(text)) {
                return a.length - b.length;
              } else {
                return -1;
              }
            } else {
              return 1;
            }
          });
          // for now we only get the first 10 results, and we don't allow you
          // to arrow-down beyond 10.. it would be nice if this was added.
          options = options.slice(0, 10);
          container.append(search_options(options));
        });
      }
    },
    "@blur"(_) {
      this.innerText = "";
      container.querySelector(".search-options")?.remove();
    },
    "@click"(_) {
      window.getSelection().selectAllChildren(this);
    },
  });

let input_container = (editor) =>
  h("div", { class: "insert-property-container" }, input(editor));

function init(editor) {
  editor
    .querySelector("[data-kind=rule] > [data-attr=properties]")
    .append(input_container(editor));
}

window.addEventListener("keydown", (e) => {
  if (document.activeElement?.closest(".insert-property-container")) return;
  if (!document.querySelector(".--editor.focused")) return;
  if (document.querySelector('[data-kind="property"].focused')) return;

  if (e.key === "/") {
    e.preventDefault();
    document
      .querySelector(".--editor.focused .insert-property-container .input")
      .click();
  }
});

document.addEventListener("DOMContentLoaded", async (_) => {
  all_properties = await invoke("get_all_properties");

  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
