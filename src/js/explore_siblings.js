const ALL_SELECTOR_PARTS_QUERY =
  '[data-kind="rule"] > [data-attr="selector"] [data-value]';

`<div data-attr="selector">
  <div data-kind="complex-selector" data-combinator-type="descendent">
    <div data-attr="left"><div data-kind="class"><div data-value="main" contenteditable="">main</div></div></div>
    <div data-attr="right"><div data-kind="class"><div data-value="btn" contenteditable="">btn</div></div></div>
  </div>
</div>`;

class SerializationError extends Error {}
class ParseError extends Error {}

function serialize_leaf_node(node) {
  switch (node.dataset.kind) {
    case "class":
      let name = node.querySelector("[data-value]").dataset.value;
      return `.${name}`;
    default:
      throw new SerializationError(node.outerHTML);
  }
}

function serialize_complex_selector(complex_node, leaf_node) {
  let left = complex_node.querySelector('[data-attr="left"] > [data-kind]');
  let right = complex_node.querySelector('[data-attr="right"] > [data-kind]');
  if (complex_node.parentElement.dataset.attr !== "selector")
    throw new SerializationError(complex_node.parentElement.outerHTML);
  if (left === leaf_node) {
    return serialize_leaf_node(leaf_node);
  } else {
    if (right !== leaf_node) throw new SerializationError("path broken");
    let combinator;
    switch (complex_node.dataset.combinatorType) {
      case "descendent":
        combinator = " ";
        break;
      default:
        throw new SerializationError(
          `unknown combinator type (${complex_node.dataset.combinatorType})`,
        );
    }
    return serialize_leaf_node(left) + combinator + serialize_leaf_node(right);
  }
}

function discover_path_for_selector_part(value_node) {
  let leaf_node = value_node.closest("[data-kind]");
  if (leaf_node.parentElement.dataset.attr === "selector") {
    return serialize_leaf_node(leaf_node);
  } else {
    let parent_node = leaf_node.parentElement.closest("[data-kind]");

    switch (parent_node.dataset.kind) {
      case "complex-selector":
        return serialize_complex_selector(parent_node, leaf_node);
      default:
        throw new SerializationError(parent_node.outerHTML);
    }
  }
}

function siblings_root(should_show, options) {
  let elem = document.createElement("div");
  elem.classList.add("siblings-root");
  elem.dataset.show = should_show;
  elem.append(...options);
  return elem;
}

function option(path, html) {
  let elem = document.createElement("a");
  elem.classList.add("sibling-option");
  elem.href = path;
  elem.innerHTML = html;
  return elem;
}

function remove_all_dropdowns() {
  for (let dropdown of document.querySelectorAll(".siblings-root"))
    dropdown.remove();
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let part of document.querySelectorAll(ALL_SELECTOR_PARTS_QUERY)) {
    part.addEventListener("mousedown", async (_) => {
      let existing_dropdown_for_self = part
        .closest("[data-kind]")
        .querySelector(":scope > .siblings-root");

      remove_all_dropdowns();

      if (existing_dropdown_for_self) return;

      let path = discover_path_for_selector_part(part);
      let leaf_node = part.closest("[data-kind]");
      let siblings = await fetch(`/src/${path}/siblings`).then((r) => r.json());
      leaf_node.append(
        siblings_root(
          true,
          siblings.map((path) => {
            let [_, elem] = path.at(-1);
            return option(path.map(([part, _]) => part).join(""), elem);
          }),
        ),
      );
    });
  }
  window.addEventListener("mousedown", (e) => {
    if (e.target.matches(ALL_SELECTOR_PARTS_QUERY)) return;
    remove_all_dropdowns();
  });
});
