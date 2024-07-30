import { find_map } from "./iter.js";

function unfocus_all() {
  for (let focused_elem of document.querySelectorAll(".focused")) {
    focused_elem.classList.remove("focused");
    focused_elem.dispatchEvent(new Event("blur"));
  }
}

export function focus(elem) {
  if (!elem) return;
  elem.classList.add("focused");
  elem.dispatchEvent(new Event("focus"));
}

// editor focus
window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  if (!e.isTrusted) return; // don't trust simulated mouse clicks

  let found_editor;

  // `elementsFromPoint` doesn't necessarily order by which element is on top of another
  // even though I believe the spec says it should
  //
  // so instead, when you click and there's a focused editor
  // in case there is multiple editors where I'm clicking
  // prefer the currently focused editor since that's the one which will
  // have a higher z-index
  //
  // TODO: Just a thought... since we are snapping..
  //       maybe we don't need to be as aggro with this anymore?
  //       ^ I don't wanna risk it lmao
  if (document.querySelector(".--editor.focused")) {
    found_editor = find_map(
      document.elementsFromPoint(e.clientX, e.clientY),
      (elem) => elem.closest(".--editor.focused"),
    );
  } else {
    found_editor = document
      .elementFromPoint(e.clientX, e.clientY)
      .closest(".--editor");
  }

  unfocus_all();
  focus(found_editor);
});

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  if (!e.isTrusted) return; // don't trust simulated mouse clicks

  let property = document
    .elementFromPoint(e.clientX, e.clientY)
    .closest('[data-kind="property"]');
  if (property) {
    unfocus_all();
    focus(property);
  }
});

// delete editor
window.addEventListener("keydown", (e) => {
  let focused_editor = document.querySelector(".--editor.focused");
  if (!focused_editor) return;

  if (e.target.closest('[data-attr="properties"]')) return;
  if (e.target.closest('[data-attr="selector"]')) return;
  if (e.target.closest(".search-box")) return;

  if (e.key === "Backspace") {
    let group = focused_editor.closest(".--editor-group");
    focused_editor.remove();
    if (!group.querySelector(".--editor")) group.remove();
  }
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

    focus(previous_property);
  } else if (e.key === "ArrowDown") {
    let next_property = focused_property.nextElementSibling;

    if (!next_property?.matches('[data-kind="property"]'))
      next_property = focused_property
        .closest('[data-attr="properties"]')
        .querySelector('[data-kind="property"]');

    focus(next_property);
  }
});

// focus value
window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  // me & the homies don't trust simulated mouse clicks
  if (!e.isTrusted) return;

  let value = document
    .elementFromPoint(e.clientX, e.clientY)
    .closest('[data-kind="property"] > [data-attr="value"]');

  if (value) {
    unfocus_all();
    focus(value);
  }
});
