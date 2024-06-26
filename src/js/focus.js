window.addEventListener("mousedown", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks

  let found_editor;

  // `elementsFromPoint` doesn't necessarily order by which element is on top of another
  // even though I believe the spec says it should
  //
  // so instead, when you click and there's a focused editor
  // in case there is multiple editors where I'm clicking
  // prefer the currently focused editor since that's the one which will
  // have a higher z-index
  if (document.querySelector(".--editor.focused")) {
    for (let elem of document.elementsFromPoint(e.clientX, e.clientY)) {
      let candidate_editor = elem.closest(".--editor.focused");
      if (candidate_editor) {
        found_editor = candidate_editor;
        break;
      }
    }
  }

  // if we are not hovering over a focused editor, look for any editor
  found_editor ??= document
    .elementFromPoint(e.clientX, e.clientY)
    .closest(".--editor");

  if (!found_editor) return;

  for (let focused_editor of document.querySelectorAll(".--editor.focused"))
    focused_editor.classList.remove("focused");

  found_editor.classList.add("focused");
});

window.addEventListener("dblclick", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks
  for (let focused of document.querySelectorAll(".focused"))
    focused.classList.remove("focused");
  let elements = document.elementsFromPoint(e.clientX, e.clientY);

  for (let elem of elements) {
    let result = elem.closest('[data-kind="property"]');
    if (!result) continue;
    result.classList.add("focused");
    break;
  }
});
