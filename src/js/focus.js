// editor focus
window.addEventListener("mousedown", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks

  // unfocus property
  document
    .querySelector('[data-kind="property"].focused')
    ?.classList?.remove("focused");

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

  // there should never be 2 focused editors
  document.querySelector(".--editor.focused")?.classList?.remove("focused");

  if (!found_editor) return;

  found_editor.classList.add("focused");
});

// property focus
window.addEventListener("click", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks

  // should only be 1 focused property at any given time.
  document
    .querySelector('[data-kind="property"].focused')
    ?.classList?.remove("focused");

  document
    .elementFromPoint(e.clientX, e.clientY)
    .closest('[data-kind="property"]')
    ?.classList?.add("focused");
});
