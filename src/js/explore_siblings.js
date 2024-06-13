const ALL_SELECTOR_PARTS_QUERY =
  '[data-kind="rule"] > [data-attr="selector"] [data-value]';

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
  for (let self_link of document.querySelectorAll(".self-link"))
    self_link.replaceWith(self_link.querySelector("[data-kind]"));
}

function find_idx_for(part) {
  let idx = 1;
  for (let p of document.querySelectorAll(ALL_SELECTOR_PARTS_QUERY)) {
    if (p === part) return idx;
    idx++;
  }
  throw new Error("can't find part :(");
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let part of document.querySelectorAll(ALL_SELECTOR_PARTS_QUERY)) {
    part.addEventListener("mousedown", async (_) => {
      let existing_dropdown_for_self = part
        .closest("[data-kind]")
        .querySelector(":scope > .siblings-root");
      remove_all_dropdowns();
      if (existing_dropdown_for_self) return;
      let idx = find_idx_for(part);
      let leaf_node = part.closest("[data-kind]");
      let siblings = await fetch(`${location.pathname}/${idx}/siblings`).then(
        (r) => r.json(),
      );
      // ahh this is kind of hacky, but I'm just replacing the part with a link... :| it works though
      let self_link = document.createElement("a");
      self_link.classList.add("self-link");
      self_link.href = `${location.pathname}/${idx}`;
      leaf_node.insertAdjacentElement("afterend", self_link);
      self_link.append(leaf_node);
      leaf_node.append(
        siblings_root(
          true,
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
