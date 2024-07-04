export function focus_editor(editor) {
  // there should never be 2 focused editors
  let previously_focused = document.querySelector(".--editor.focused");

  previously_focused?.classList?.remove("focused");
  previously_focused?.dispatchEvent(new Event("blur"));
  if (!editor) return;

  editor.classList.add("focused");
  editor.dispatchEvent(new Event("focus"));
}

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

  found_editor ??= document
    .elementFromPoint(e.clientX, e.clientY)
    .closest(".--editor");

  focus_editor(found_editor);
});

// property focus
function focus_property(property) {
  // should only be 1 focused property at any given time.
  document
    .querySelector('[data-kind="property"].focused')
    ?.classList?.remove("focused");

  // we may get nothing
  property?.classList?.add("focused");
}

window.addEventListener("click", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks

  focus_property(
    document
      .elementFromPoint(e.clientX, e.clientY)
      .closest('[data-kind="property"]'),
  );
});

// go up and down properties with arrow keys when a property is focused
window.addEventListener("keydown", (e) => {
  let focused_property = document.querySelector(
    '[data-kind="property"].focused',
  );
  if (!focused_property) return;

  if (e.key === "ArrowUp") {
    let previous_property = focused_property.previousElementSibling;
    if (!previous_property) {
      // hack
      previous_property = [
        ...focused_property
          .closest('[data-attr="properties"]')
          .querySelectorAll('[data-kind="property"]'),
      ].at(-1);
    }

    focus_property(previous_property);
  } else if (e.key === "ArrowDown") {
    let next_property = focused_property.nextElementSibling;

    if (!next_property?.matches('[data-kind="property"]'))
      next_property = focused_property
        .closest('[data-attr="properties"]')
        .querySelector('[data-kind="property"]');

    focus_property(next_property);
  }
});

window.addEventListener("tab-into", (e) => {
  document.querySelector(".--editor.focused")?.classList?.remove("focused");

  let editor = e.target.closest(".--editor");
  document.querySelector(":focus")?.blur();
  editor.classList.add("focused");
});
