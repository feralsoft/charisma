const ALL_SELECTOR_PARTS_QUERY =
  '[data-kind="rule"] > [data-attr="selector"] [data-value]';

function siblings_dropdown(options) {
  let elem = document.createElement("div");
  elem.classList.add("siblings-root");
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

function remove_all_dropdowns(editor) {
  for (let dropdown of editor.querySelectorAll(".siblings-root"))
    dropdown.remove();
  for (let self_link of editor.querySelectorAll(".self-link"))
    self_link.replaceWith(self_link.querySelector(":scope > [data-kind]"));
}

function find_idx_for(editor, search_part) {
  let idx = 1;
  for (let iter_part of editor.querySelectorAll(ALL_SELECTOR_PARTS_QUERY)) {
    if (iter_part === search_part) return idx;
    idx++;
  }
  throw new Error("can't find part :(");
}

function create_self_link_for(leaf_node, idx) {
  let self_link = document.createElement("a");
  self_link.classList.add("self-link");
  let url = leaf_node.closest(".--editor").dataset.url;
  self_link.href = `${url}/at/${idx}`;
  leaf_node.insertAdjacentElement("afterend", self_link);
  self_link.append(leaf_node);
}

function init(editor) {
  for (let part of editor.querySelectorAll(ALL_SELECTOR_PARTS_QUERY)) {
    part.addEventListener("mousedown", async (_) => {
      // if we are in an existing dropdown don't rebuild
      let existing_dropdown_for_self = part
        .closest("[data-kind]")
        .querySelector(":scope > .siblings-root");
      if (existing_dropdown_for_self) return;
      // remove old dropdowns
      remove_all_dropdowns(editor);
      // find the selector idx of current part
      let idx = find_idx_for(editor, part);
      let leaf_node = part.closest("[data-kind]");
      let url = leaf_node.closest(".--editor").dataset.url;
      let siblings = await fetch(`${url}/at/${idx}/siblings`).then((r) =>
        r.json(),
      );
      // turn myself into a link
      create_self_link_for(leaf_node, idx);
      // create dropdown with sibling links
      leaf_node.append(
        siblings_dropdown(
          siblings.map((path) => {
            let [_, elem] = path.at(idx - 1);
            // this is broken for attribute selectors since
            // [data-value="hey"] when parsed to parts becomes ['[data-value]', '[data-value="hey"]']
            // so either on the backend we simplify selectors (i think this is the right call)
            // or on the frontend we de-dup this pattern (i don't think this is a good idea)
            return option(path.map(([part, _]) => part).join(""), elem);
          }),
        ),
      );
    });
  }
  window.addEventListener("mousedown", (e) => {
    if (e.target.matches(ALL_SELECTOR_PARTS_QUERY)) return;
    remove_all_dropdowns(editor);
  });
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
