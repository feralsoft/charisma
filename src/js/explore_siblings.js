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

function remove_all_dropdowns() {
  for (let dropdown of document.querySelectorAll(".siblings-root"))
    dropdown.remove();
  for (let self_link of document.querySelectorAll(".self-link"))
    self_link.replaceWith(self_link.querySelector(":scope > [data-kind]"));
}

function find_idx_for(search_part) {
  let idx = 1;
  for (let iter_part of document.querySelectorAll(ALL_SELECTOR_PARTS_QUERY)) {
    if (iter_part === search_part) return idx;
    idx++;
  }
  throw new Error("can't find part :(");
}

function create_self_link_for(leaf_node, idx) {
  let self_link = document.createElement("a");
  self_link.classList.add("self-link");
  self_link.href = `${location.pathname}/${idx}`;
  leaf_node.insertAdjacentElement("afterend", self_link);
  self_link.append(leaf_node);
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let part of document.querySelectorAll(ALL_SELECTOR_PARTS_QUERY)) {
    part.addEventListener("mousedown", async (_) => {
      // if we are in an existing dropdown don't rebuild
      let existing_dropdown_for_self = part
        .closest("[data-kind]")
        .querySelector(":scope > .siblings-root");
      if (existing_dropdown_for_self) return;
      // remove old dropdowns
      remove_all_dropdowns();
      // find the selector idx of current part
      let idx = find_idx_for(part);
      let leaf_node = part.closest("[data-kind]");
      let siblings = await fetch(`${location.pathname}/${idx}/siblings`).then(
        (r) => r.json(),
      );
      // turn myself into a link
      create_self_link_for(leaf_node, idx);
      // create dropdown with sibling links
      leaf_node.append(
        siblings_dropdown(
          siblings.map((path) => {
            let [_, elem] = path.at(idx - 1);
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
