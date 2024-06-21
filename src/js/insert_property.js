let all_properties;

function search_item(name) {
  let elem = document.createElement("div");
  elem.innerText = name;
  elem.classList.add("search-item");
  return elem;
}

function search_options(names) {
  let options = document.createElement("div");
  options.classList.add("search-options");
  options.append(...names.map(search_item));
  options.firstElementChild.classList.add("candidate");
  return options;
}

function input(editor) {
  let container = document.createElement("div");
  let input_elem = document.createElement("div");
  input_elem.classList.add("input");
  input_elem.contentEditable = true;
  input_elem.placeholder = "insert property...";
  input_elem.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      await fetch(editor.dataset.url, {
        method: "POST",
        body: e.target.innerText,
      });
      input_elem.dispatchEvent(new Event("reload", { bubbles: true }));
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
      // pick option
      let options = container.querySelector(".search-options");
      input_elem.innerText = options.querySelector(".candidate").innerText;
      options.remove();
      // start garbage internet code
      const range = document.createRange();
      const selection = window.getSelection();
      range.setStart(input_elem, input_elem.childNodes.length);
      range.collapse(true);
      selection.removeAllRanges();
      selection.addRange(range);
      // end of garbage internet code

      e.preventDefault();
    } else {
      // populate auto complete list
      setTimeout(() => {
        container.querySelector(".search-options")?.remove();
        if (input_elem.innerText.trim() === "") return;
        let options = all_properties.filter((name) =>
          name.includes(input_elem.innerText),
        );

        options.sort((a, b) => a.length - b.length);
        options = options.slice(0, 10);
        container.append(search_options(options));
      });
    }
  });
  input_elem.addEventListener("blur", (_) => (input_elem.innerText = ""));
  input_elem.addEventListener("click", (_) =>
    window.getSelection().selectAllChildren(input_elem),
  );

  container.append(input_elem);
  return container;
}

function init(editor) {
  let properties = editor.querySelector(
    "[data-kind=rule] > [data-attr=properties]",
  );

  properties.append(input(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  all_properties = eval(document.querySelector("#css-properties").innerHTML);

  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
