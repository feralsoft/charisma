function insert_delete_button(src) {
  let button = document.createElement("button");
  button.addEventListener("click", (_) => {
    let name = src.querySelector("[data-attr=name]").textContent;
    fetch(`${location.pathname}/${name}`, { method: "DELETE" }).then((_) =>
      location.reload(),
    );
  });
  button.innerText = "delete";
  src.append(button);
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let property of document.querySelectorAll(
    "[data-attr=properties] > [data-kind=property]",
  )) {
    insert_delete_button(property);
  }
});
