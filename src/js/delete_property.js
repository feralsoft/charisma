function insert_comment_button(src) {
  let button = document.createElement("button");
  button.innerText = "⤫";
  button.classList.add("soft-delete");
  button.dataset.active = src.dataset.commented === "true";
  button.addEventListener("click", async (_) => {
    button.dataset.active = button.dataset.active !== "true";
    let name = src.querySelector("[data-attr=name]").textContent;
    await fetch(`${location.pathname}/${name}/toggle_comment`, {
      method: "POST",
    });
    location.reload();
  });
  src.prepend(button);
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let property of document.querySelectorAll(
    "[data-attr=properties] > [data-kind=property]",
  )) {
    insert_comment_button(property);
  }
});
