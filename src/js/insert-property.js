import { h } from "./html.js";
import invoke from "./invoke.js";

let { div } = h;

// top level await seems to break safari

let ALL_PROPERTIES = window.__TAURI__.tauri
  .invoke("all_properties")
  .then(JSON.parse);

let ALL_PROPERTY_NAMES = ALL_PROPERTIES.then(Object.keys);

let search_item = ({ value, description }) => [
  div({ class: "search-item-value" }, value),
  div(
    {
      class: "search-item-description",
      "data-is-empty": description.length === 0,
    },
    description,
  ),
];

function search_options(options, has_description = false) {
  // hack
  if (options.length === 0) return document.createElement("div");

  if (!has_description)
    options = options.map((name) => ({ value: name, description: "" }));

  let elem = div({ class: "search-options" }, ...options.flatMap(search_item));

  return elem;
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
  let candidate =
    options.querySelector(".candidate") ?? options.firstElementChild;

  if (input_elem.innerText.includes(":")) {
    // we are accepting a value
    let [name, _] = input_elem.innerText.split(":");
    input_elem.innerText = name.trim() + ": " + candidate.innerText.trim();
  } else {
    // we are accepting a name
    input_elem.innerText = candidate.innerText.trim() + ":";
  }
  options.remove();

  move_cursor_to_end_of_element(input_elem);
}

let input = (editor) =>
  div({
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
          await invoke(editor, "insert_property", {
            path: localStorage.getItem("current-path"),
            selector: editor.dataset.selector,
            property: e.target.innerText.trim(),
          });
          this.innerText = "";
          container.querySelector(".search-options")?.remove();
          this.click();
        }
      } else if (e.key === "Escape") {
        this.blur();
      } else if (e.key === "ArrowUp") {
        // go up in search options
        e.preventDefault();
        let options = container.querySelector(".search-options");
        let elem =
          options.querySelector(".candidate") ?? options.firstElementChild;
        if (elem.previousElementSibling) {
          let prev_elem = elem.previousElementSibling;
          if (!prev_elem.matches(".search-item-value"))
            prev_elem = prev_elem.previousElementSibling;
          elem.classList.remove("candidate");
          prev_elem.classList.add("candidate");
        }
      } else if (e.key === "ArrowDown") {
        // go down in search options
        e.preventDefault();
        let options = container.querySelector(".search-options");
        let elem =
          options.querySelector(".candidate") ?? options.firstElementChild;
        if (elem.nextElementSibling) {
          let next_elem = elem.nextElementSibling;
          if (!next_elem.matches(".search-item-value"))
            next_elem = next_elem.nextElementSibling;
          elem.classList.remove("candidate");
          next_elem.classList.add("candidate");
        }
      } else if (e.key === "Tab") {
        accept_candidate(container, this);
        e.preventDefault();
      } else {
        // ^ why do we care about shift key
        // populate auto complete list
        // setTimeout is needed so that `this.innerText` gets populated :facepalm:
        setTimeout(async () => {
          container.querySelector(".search-options")?.remove();
          let text = this.innerText.trim();
          if (text === "") return;
          // if the search contains a property name, let's search within it
          if (text.includes(":")) {
            let possible_property_name = text.split(":")[0];
            if ((await ALL_PROPERTIES)[possible_property_name.trim()]) {
              let search_text = text.split(":")[1].trim();
              let list = (await ALL_PROPERTIES)[possible_property_name.trim()];
              let options = list.filter(({ value }) =>
                value.includes(search_text),
              );

              options.sort((a, b) => {
                if (a.value.startsWith(text)) {
                  if (b.value.startsWith(text)) {
                    return a.value.length - b.value.length;
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
              container.append(search_options(options, true));
            }
          } else {
            let options = (await ALL_PROPERTY_NAMES).filter((name) =>
              name.includes(text),
            );

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
          }
        });
      }
    },
    "@blur"(_) {
      let container = this.closest(".insert-property-container");
      this.innerText = "";
      container.querySelector(".search-options")?.remove();
    },
    "@click"(_) {
      window.getSelection().selectAllChildren(this);
    },
  });

let input_container = (editor) =>
  div({ class: "insert-property-container" }, input(editor));

function init(editor) {
  editor
    .querySelector("[data-kind=rule] > [data-attr=properties]")
    .append(input_container(editor));
}

window.addEventListener("keydown", (e) => {
  let editor = document.querySelector(".--editor.focused");
  if (!editor) return;
  if (document.activeElement.closest('[data-kind="property"]')) return;

  if (e.key === "/") {
    e.preventDefault();
    editor.querySelector(".insert-property-container .input").focus();
  }
});

function reload(editor) {
  let input = editor.querySelector(
    "[data-kind=rule] > [data-attr=properties] .insert-property-container",
  );
  editor
    .querySelector("[data-kind=rule] > [data-attr=properties]")
    .append(input);
}

document.addEventListener("DOMContentLoaded", async (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    init(editor);
    editor.addEventListener("loaded", (_) => reload(editor));
  });
});
