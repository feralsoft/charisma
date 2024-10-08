import invoke from "./invoke.js";

// make number go up & down (unit)

let is_dragging, unit, editor, start_y, lock;

function finish(_) {
  is_dragging = false;
  unit = null;
  editor = null;
  start_y = null;
  document.body.classList.remove("dragging-unit");
}

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  let candidate = e.target.closest('[data-kind="unit"]');
  if (!candidate) return;
  is_dragging = true;

  unit = candidate;
  document.body.classList.add("dragging-unit");
  editor = unit.closest(".--editor");
  start_y = e.clientY;
});

function rep_for_type(type) {
  switch (type) {
    case "px":
      return "px";
    case "em":
      return "em";
    case "rem":
      return "rem";
    case "percentage":
      return "%";
    default:
      throw new Error("unknown unit type [" + type + "]");
  }
}

function factor_for_value(string_value) {
  let factor = string_value.split(".")[1]?.length ?? 0;
  return 1 / 10 ** factor;
}

function parse_unit(unit) {
  let string_value = unit.querySelector("[data-value]").dataset.value;
  let factor = factor_for_value(string_value);
  return {
    value: Number(string_value),
    type: rep_for_type(unit.dataset.unitType),
    precision: String(factor).split(".")[1]?.length ?? 0,
    name: unit
      .closest('[data-kind="property"]')
      .querySelector('[data-attr="name"] [data-value]').dataset.value,
    factor,
  };
}

window.addEventListener("mousemove", async (e) => {
  if (!is_dragging) return;
  if (lock) return;
  lock = true;
  let { value, type, precision, factor, name } = parse_unit(unit);
  let diff = (start_y - e.clientY) * factor;
  start_y = e.clientY;

  let original_unit_value = `${value.toFixed(precision)}${type}`;

  let updated_unit_value = `${(value + diff).toFixed(precision)}${type}`;

  let original_value = unit
    .closest('[data-attr="value"]')
    .querySelector(":scope > [data-kind]").dataset.stringValue;

  assert(
    unit
      .closest('[data-attr="value"]')
      .querySelectorAll(
        `[data-value="${CSS.escape(value.toFixed(precision))}"]`,
      ).length === 1,
    "duplicate values are being scrubbed",
  );

  let updated_value = original_value.replace(
    original_unit_value,
    updated_unit_value,
  );

  await invoke(editor, "update_value", {
    path: localStorage.getItem("current-path"),
    selector: editor.dataset.selector,
    name,
    original_value,
    value: updated_value,
  });

  lock = false;
});

window.addEventListener("mouseup", finish);
window.addEventListener("blur", finish);

window.addEventListener("keydown", async (e) => {
  let unit = document.querySelector(
    '[data-attr="value"].focused [data-kind="unit"]',
  );
  if (!unit) return;

  let { value, type, precision, factor, name } = parse_unit(unit);
  let editor = unit.closest(".--editor");

  if (e.key === "ArrowUp") {
    await invoke(editor, "update_value", {
      path: localStorage.getItem("current-path"),
      selector: editor.dataset.selector,
      name,
      original_value: `${value.toFixed(precision)}${type}`,
      value: `${(value + factor).toFixed(precision)}${type}`,
    });
  } else if (e.key === "ArrowDown") {
    await invoke(editor, "update_value", {
      path: localStorage.getItem("current-path"),
      selector: editor.dataset.selector,
      name,
      original_value: `${value.toFixed(precision)}${type}`,
      value: `${(value - factor).toFixed(precision)}${type}`,
    });
  }
});
