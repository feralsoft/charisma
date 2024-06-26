let all_properties;

function search_item(name) {
  let elem = document.createElement("div");
  elem.innerText = name;
  elem.classList.add("search-item");
  return elem;
}

function search_options(names) {
  // hack
  if (names.length === 0) return document.createElement("div");
  let options = document.createElement("div");
  options.classList.add("search-options");
  options.append(...names.map(search_item));
  options.firstElementChild.classList.add("candidate");
  return options;
}

function accept_candidate(container, input_elem) {
  let options = container.querySelector(".search-options");
  input_elem.innerText = options.querySelector(".candidate").innerText + ":";
  options.remove();

  // start garbage internet code
  let range = document.createRange();
  let selection = window.getSelection();
  range.setStart(input_elem, input_elem.childNodes.length);
  range.collapse(true);
  selection.removeAllRanges();
  selection.addRange(range);
  // end of garbage internet code
}

function input(editor) {
  let container = document.createElement("div");
  container.classList.add("insert-property-container");
  let input_elem = document.createElement("div");
  input_elem.classList.add("input");
  input_elem.contentEditable = true;
  input_elem.placeholder = "insert property...";
  input_elem.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      if (container.querySelector(".search-options .candidate")) {
        return accept_candidate(container, input_elem);
      } else {
        await fetch(editor.dataset.url, {
          method: "POST",
          body: e.target.innerText,
        });
        input_elem.dispatchEvent(new Event("reload", { bubbles: true }));
      }
    } else if (e.key === "Escape") {
      input_elem.blur();
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
      accept_candidate(container, input_elem);
      e.preventDefault();
    } else if (!e.shiftKey) {
      // populate auto complete list
      setTimeout(() => {
        container.querySelector(".search-options")?.remove();
        let text = input_elem.innerText.trim();
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
        options = options.slice(0, 10);
        container.append(search_options(options));
      });
    }
  });
  input_elem.addEventListener("blur", (_) => (input_elem.innerText = ""));
  input_elem.addEventListener("click", (_) => {
    window.getSelection().selectAllChildren(input_elem);
  });

  container.append(input_elem);
  return container;
}

function init(editor) {
  let properties = editor.querySelector(
    "[data-kind=rule] > [data-attr=properties]",
  );

  properties.append(input(editor));
}
window.addEventListener("keydown", (e) => {
  if (
    document.activeElement?.closest(
      ":is(.insert-property-container, [data-value])",
    )
  )
    return;
  if (e.key === "/") {
    e.preventDefault();
    document
      .querySelector(".--editor.focused .insert-property-container .input")
      .click();
  }
});

document.addEventListener("DOMContentLoaded", (_) => {
  all_properties = eval(document.querySelector("#css-properties").innerHTML);

  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
