const { invoke } = window.__TAURI__.tauri;

// make number go up & down (unit)

let is_dragging, current_value, editor, start_y, lock;

function finish(_) {
  is_dragging = false;
  current_value = null;
  editor = null;
  start_y = null;
  document.body.classList.remove("dragging-unit");
}

window.addEventListener("mousedown", (e) => {
  let unit = e.target.closest('[data-kind="unit"]');
  if (!unit) return;
  is_dragging = true;

  document.body.classList.add("dragging-unit");
  editor = unit.closest(".--editor");
  current_value = unit.dataset.stringValue;
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
  try {
    let unit = editor.querySelector(
      '[data-string-value="' + current_value + '"]',
    );
    let { value, type, precision, factor, name } = parse_unit(unit);
    let diff = (start_y - e.clientY) * factor;
    start_y = e.clientY;

    await invoke("update_value", {
      path: localStorage.getItem("current-path"),
      selector: editor.dataset.selector,
      name,
      original_value: `${value.toFixed(precision)}${type}`,
      value: `${(value + diff).toFixed(precision)}${type}`,
    });
    unit.dispatchEvent(new Event("reload", { bubbles: true }));
    current_value = `${(value + diff).toFixed(precision)}${type}`;
  } catch (_) {
    finish();
  } finally {
    lock = false;
  }
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
    await invoke("update_value", {
      path: localStorage.getItem("current-path"),
      selector: editor.dataset.selector,
      name,
      original_value: `${value.toFixed(precision)}${type}`,
      value: `${(value + factor).toFixed(precision)}${type}`,
    });
    unit.dispatchEvent(new Event("reload", { bubbles: true }));
  } else if (e.key === "ArrowDown") {
    await invoke("update_value", {
      path: localStorage.getItem("current-path"),
      selector: editor.dataset.selector,
      name,
      original_value: `${value.toFixed(precision)}${type}`,
      value: `${(value - factor).toFixed(precision)}${type}`,
    });
    unit.dispatchEvent(new Event("reload", { bubbles: true }));
  }
});