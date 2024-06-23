function num(px_str) {
  return Number(px_str.split("px")[0]);
}

const parents = Symbol("parents");

function calc_connnection(connection_elem, me_elem, you_elem) {
  connection_elem[parents] = { me: me_elem, you: you_elem };
  let me_rect = me_elem.getBoundingClientRect();
  let you_rect = you_elem.getBoundingClientRect();
  let me = {
    x: num(me_elem.style.left) + me_rect.width / 2,
    y: num(me_elem.style.top) + me_rect.height / 2,
  };
  let you = {
    x: num(you_elem.style.left) + you_rect.width / 2,
    y: num(you_elem.style.top) + you_rect.height / 2,
  };
  connection_elem.setAttribute("x1", me.x);
  connection_elem.setAttribute("x2", you.x);
  connection_elem.setAttribute("y1", me.y);
  connection_elem.setAttribute("y2", you.y);
}

function update_connections_for(editor) {
  for (let connection of document.querySelectorAll(".connection")) {
    if (connection[parents].me === editor) {
      calc_connnection(
        connection,
        connection[parents].me,
        connection[parents].you,
      );
    }
  }
}

function init(editor) {
  for (let connection of document.querySelectorAll(".connection"))
    connection.remove();
  let svg = document.querySelector(".canvas svg");
  let my_selector = editor.querySelector('[data-attr="selector"] > [data-kind]')
    .dataset.stringValue;

  for (let other_editor of document.querySelectorAll(".--editor")) {
    if (other_editor === editor) continue;
    let other_selector = other_editor.querySelector(
      '[data-attr="selector"] > [data-kind]',
    ).dataset.stringValue;
    if (my_selector.includes(other_selector)) {
      let connection = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "line",
      );
      connection.classList.add("connection");
      calc_connnection(connection, editor, other_editor);
      svg.append(connection);
    }
  }
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  let svg_canvas = document.createElementNS(
    "http://www.w3.org/2000/svg",
    "svg",
  );
  canvas.append(svg_canvas);
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
    editor.addEventListener("moved", (_) => update_connections_for(editor));
  });
});
