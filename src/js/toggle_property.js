function insert_comment_button(src) {
  let button = document.createElement("button");
  button.innerHTML = "<div class='text'>⤫</div>";
  button.classList.add("toggle-comment");
  button.addEventListener("click", async (_) => {
    let name = src.querySelector("[data-attr=name]").textContent;
    let is_commented =
      src.closest('[data-kind="property"]').dataset.commented === "true";
    let action = is_commented ? "enable" : "disable";
    await fetch(`${location.pathname}/${name}/${action}`, { method: "POST" });
    location.search = "";
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
