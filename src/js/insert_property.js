function input(editor) {
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
    }
  });
  input_elem.addEventListener("blur", (_) => (input_elem.innerText = ""));
  input_elem.addEventListener("click", (_) =>
    window.getSelection().selectAllChildren(input_elem),
  );

  return input_elem;
}

function init(editor) {
  let properties = editor.querySelector(
    "[data-kind=rule] > [data-attr=properties]",
  );

  properties.append(input(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let editor of document.querySelectorAll(".--editor")) {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  }
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});
