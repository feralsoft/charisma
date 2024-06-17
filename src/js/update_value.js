const VALUE_SELECTOR =
  '[data-attr="properties"] > [data-kind="property"] > [data-attr="value"] > [data-kind]';

function plain_text_node(name, text) {
  let node = document.createElement("div");
  node.classList.add("plain-text-node");
  node.innerText = text;
  node.contentEditable = true;
  setTimeout(() => window.getSelection().selectAllChildren(node));
  node.addEventListener("keydown", async (e) => {
    if (e.key === "Escape") {
      location.search = "";
    } else if (e.key === "Enter") {
      e.preventDefault();
      await fetch(`${location.pathname}/${name}/value`, {
        method: "POST",
        body: node.innerText,
      });
      location.search = "";
    }
  });
  node.addEventListener("blur", (_) => {
    location.search = "";
  });
  return node;
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let value of document.querySelectorAll(VALUE_SELECTOR)) {
    let name = value
      .closest('[data-kind="property"]')
      .querySelector('[data-attr="name"]').innerText;
    value.addEventListener("dblclick", (_) => {
      value.replaceWith(plain_text_node(name, value.dataset.stringValue));
    });
  }
});
