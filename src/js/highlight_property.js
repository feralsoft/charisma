document.addEventListener("DOMContentLoaded", (_) => {
  let name = new URL(location.href).searchParams.get("highlight_property_name");
  if (!name) return;
  let elem = document.querySelector(
    `[data-kind="property"]:has(> [data-attr="name"] > [data-value="${name}"])`,
  );
  elem.classList.add("highlighted");
  elem.scrollIntoView();
});
