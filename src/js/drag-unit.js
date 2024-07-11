const { invoke } = window.__TAURI__.tauri;

// drag unit up & down
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
  e.stopImmediatePropagation();
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

function factor_for_value(value) {
  let factor = value.split(".")[1]?.length ?? 0;
  return 1 / 10 ** factor;
}

window.addEventListener("mousemove", async (e) => {
  if (!is_dragging) return;
  if (lock) return;
  lock = true;
  try {
    let unit = editor.querySelector(
      '[data-string-value="' + current_value + '"]',
    );
    let value = unit.querySelector("[data-value]").dataset.value;
    let type = rep_for_type(unit.dataset.unitType);
    let factor = factor_for_value(value);
    console.log(current_value, factor);
    let diff = (start_y - e.clientY) * factor;
    let fix_percision = String(factor).split(".")[1]?.length ?? 0; // to fucking deal with floating point issues :(
    start_y = e.clientY;
    let name = unit
      .closest('[data-kind="property"]')
      .querySelector('[data-attr="name"] [data-value]').dataset.value;
    let current_number_value = Number(value);

    await invoke("update_value", {
      selector: editor.dataset.selector,
      name,
      original_value: `${current_number_value.toFixed(fix_percision)}${type}`,
      value: `${(current_number_value + diff).toFixed(fix_percision)}${type}`,
    });
    unit.dispatchEvent(new Event("reload", { bubbles: true }));
    current_value = `${(current_number_value + diff).toFixed(fix_percision)}${type}`;
  } finally {
    lock = false;
  }
});

window.addEventListener("mouseup", finish);
window.addEventListener("blur", finish);
