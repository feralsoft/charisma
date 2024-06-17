document.addEventListener("DOMContentLoaded", (_) => {
  let name = new URL(location.href).searchParams.get("highlight_property_name");
  if (!name) return;
  document
    .querySelector(
      `[data-kind="property"]:has(> [data-attr="name"] > [data-value="${name}"])`,
    )
    .classList.add("highlighted");
});
