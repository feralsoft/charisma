function insert_comment_button(src) {
  let button = document.createElement("button");
  button.innerHTML = "<div class='text'>⤫</div>";
  button.classList.add("toggle-comment");
  button.addEventListener("mousedown", async (_) => {
    let name = src.querySelector("[data-attr=name]").textContent;
    let is_commented =
      src.closest('[data-kind="property"]').dataset.commented === "true";
    let action = is_commented ? "enable" : "disable";
    await fetch(`${location.pathname}/${name}/${action}`, { method: "POST" });
    src.dispatchEvent(new Event("reload", { bubbles: true }));
  });
  src.prepend(button);
}

function init(editor) {
  for (let property of editor.querySelectorAll(
    "[data-attr=properties] > [data-kind=property]",
  )) {
    insert_comment_button(property);
  }
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let editor of document.querySelectorAll(".--editor")) {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  }
});
