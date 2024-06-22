// prototype.. yucky

function num(px_str) {
  return Number(px_str.split("px")[0]);
}

const DIR = {
  BOTTOM: "BOTTOM",
  TOP: "TOP",
  LEFT: "LEFT",
  RIGHT: "RIGHT",
};

// bottom of me (contained)
//   |--|
//   |me|
//   |--|
//    | <- connection is between my bottom & your top, connecting @ our x midpoints
// |-----|
// | you |
// |-----|

// top of me (contained)
// |-----|
// | you |
// |-----|
//    | <- connection is between my top & your bottom, connecting @ our x midpoints
//   |--|
//   |me|
//   |--|

// left of me (contained)
// |-----|
// | you | - [me] | connection between you.right & me.left between our y midpoints
// |-----|

// right of me (contained)
//        |-----|
// [me] - | you |   | connection between me.right & you.left between our y midpoints
//        |-----|

// bottom right of me
// [me]
//    \  <- connection between me.bottom & you.top between our x midpoints
//   |-----|
//   | you |
//   |-----|

// bottom right of me
//       [me]
//        / <- connection between me.bottom & you.top between our x midpoints
//   |-----|
//   | you |
//   |-----|

let me, you;

function calc_dir(me_elem, you_elem) {
  let me_rect = me_elem.getBoundingClientRect();
  let you_rect = you_elem.getBoundingClientRect();
  me = {
    left: num(me_elem.style.left),
    top: num(me_elem.style.top),
  };
  me = {
    ...me,
    right: me.left + me_rect.width,
    bottom: me.top + me_rect.height,
  };
  you = {
    left: num(you_elem.style.left),
    top: num(you_elem.style.top),
  };
  you = {
    ...you,
    right: you.left + you_rect.width,
    bottom: you.top + you_rect.height,
  };

  let dist_between_my_bottom_and_your_top = you.top - me.bottom;
  let dist_between_my_top_and_your_bottom = me.top - you.bottom;
  let dist_between_my_left_and_your_right = me.left - you.right;
  let dist_between_me_right_and_your_left = you.left - me.right;

  if (dist_between_my_bottom_and_your_top > 0) {
    return DIR.BOTTOM;
  } else if (dist_between_my_top_and_your_bottom > 0) {
    return DIR.TOP;
  } else if (dist_between_my_left_and_your_right > 0) {
    return DIR.LEFT;
  } else if (dist_between_me_right_and_your_left > 0) {
    return DIR.RIGHT;
  }

  throw new Error("");
}

function calc_connnection(connection_elem, me_elem, you_elem) {
  let dir = calc_dir(me_elem, you_elem);
  if (dir === DIR.LEFT) {
    //       |----|
    // [you] | me |
    //       |----|
    // your right & midpoints between top & bottom
    //
    // mount connection on left
    let you_x = you.right;
    let you_y = (you.top + you.bottom) / 2;
    let me_x = me.left;
    let me_y = (me.top + me.bottom) / 2;
    let hypo = Math.sqrt((me_x - you_x) ** 2 + (me_y - you_y) ** 2);
    connection_elem.style.setProperty("--dist", `${hypo}px`);
    connection_elem.dataset.mounted = "left";
    connection_elem.style.setProperty("--x", `${you_x}px`);
    connection_elem.style.setProperty("--y", `${you_y}px`);
    // are you above me, or nah
    if (you_y > me_y) {
      //       |----|
      //    /  | me |
      // [you] |----|
      let opposite = you_y - me_y;

      connection_elem.style.setProperty(
        "--angle",
        `-${Math.asin(opposite / hypo)}rad`,
      );
    } else {
      // [you] |----|
      //    \  | me |
      //       |----|
      let opposite = me_y - you_y;
      connection_elem.style.setProperty(
        "--angle",
        `${Math.asin(opposite / hypo)}rad`,
      );
    }
  } else if (dir === DIR.RIGHT) {
    // |----|
    // | me | [you]
    // |----|
    // your left & midpoints between top & bottom
    //
    // mount connection on left
    let you_x = you.left;
    let you_y = (you.top + you.bottom) / 2;
    let me_x = me.right;
    let me_y = (me.top + me.bottom) / 2;
    let hypo = Math.sqrt((me_x - you_x) ** 2 + (me_y - you_y) ** 2);
    connection_elem.style.setProperty("--dist", `${hypo}px`);
    connection_elem.dataset.mounted = "left";
    connection_elem.style.setProperty("--x", `${me_x}px`);
    connection_elem.style.setProperty("--y", `${me_y}px`);
    // are you above me, or nah
    if (you_y > me_y) {
      // |----|
      // | me | \
      // |----| [you]
      let opposite = you_y - me_y;

      connection_elem.style.setProperty(
        "--angle",
        `${Math.asin(opposite / hypo)}rad`,
      );
    } else {
      // |----| [you]
      // | me |  /
      // |----|
      let opposite = me_y - you_y;
      connection_elem.style.setProperty(
        "--angle",
        `-${Math.asin(opposite / hypo)}rad`,
      );
    }
  } else if (dir === DIR.BOTTOM) {
    // |----|
    // | me |
    // |----|
    //   |
    //  [you]
    // your bottom & midpoints between left & right
    //
    // mount connection on top
    let you_y = you.top;
    let you_x = (you.left + you.right) / 2;
    let me_y = me.bottom;
    let me_x = (me.left + me.right) / 2;
    let hypo = Math.sqrt((me_x - you_x) ** 2 + (me_y - you_y) ** 2);
    connection_elem.style.setProperty("--dist", `${hypo}px`);
    connection_elem.dataset.mounted = "top";
    connection_elem.style.setProperty("--x", `${me_x}px`);
    connection_elem.style.setProperty("--y", `${me_y}px`);
    // are you right of me, or nah
    if (you_x > me_x) {
      // |-----|
      // | me  |
      // |-(a)-|
      //    \  | (op)
      //    [you]
      let opposite = you_y - me_y;

      connection_elem.style.setProperty(
        "--angle",
        `${Math.asin(opposite / hypo)}rad`,
      );
    } else {
      //     |----|
      //     | me |
      //     |-(a)|
      //(op) | /
      //    [you]
      let opposite = you_y - me_y;
      connection_elem.style.setProperty(
        "--angle",
        `${-Math.asin(opposite / hypo) + Math.PI}rad`,
      );
    }
  } else if (dir === DIR.TOP) {
    //  [you]
    //   |
    // |----|
    // | me |
    // |----|
    // your top & midpoints between left & right
    //
    // mount connection on top
    let you_y = you.bottom;
    let you_x = (you.left + you.right) / 2;
    let me_y = me.top;
    let me_x = (me.left + me.right) / 2;
    let hypo = Math.sqrt((me_x - you_x) ** 2 + (me_y - you_y) ** 2);
    connection_elem.style.setProperty("--dist", `${hypo}px`);
    connection_elem.dataset.mounted = "top";
    connection_elem.style.setProperty("--x", `${you_x}px`);
    connection_elem.style.setProperty("--y", `${you_y}px`);
    // are you right of me, or nah
    if (you_x < me_x) {
      //    [you]
      //    /  | (op)
      // |-(a)-|
      // | me  |
      // |-----|
      let opposite = me_y - you_y;
      connection_elem.style.setProperty(
        "--angle",
        `${Math.asin(opposite / hypo)}rad`,
      );
    } else {
      //    [you]
      //(op) | \
      //     |-(a)|
      //     | me |
      //     |----|
      let opposite = me_y - you_y;
      connection_elem.style.setProperty(
        "--angle",
        `${-Math.asin(opposite / hypo) + Math.PI}rad`,
      );
    }
  }
}

function init(editor) {
  for (let connection of document.querySelectorAll(".connection"))
    connection.remove();
  let canvas = document.querySelector(".canvas");
  let my_selector = editor.querySelector('[data-attr="selector"] > [data-kind]')
    .dataset.stringValue;

  for (let other_editor of document.querySelectorAll(".--editor")) {
    if (other_editor === editor) continue;
    let other_selector = other_editor.querySelector(
      '[data-attr="selector"] > [data-kind]',
    ).dataset.stringValue;
    if (my_selector.includes(other_selector)) {
      let connection = document.createElement("div");
      connection.classList.add("connection");
      calc_connnection(connection, editor, other_editor);
      canvas.append(connection);
    }
  }
}

// a little too yucky to enable by default
//
// document.addEventListener("DOMContentLoaded", (_) => {
//   let canvas = document.querySelector(".canvas");
//   canvas.addEventListener("new-editor", (_) => {
//     let editor = document.querySelector(".--editor:last-child");
//     init(editor);
//     editor.addEventListener("loaded", (_) => init(editor));
//     editor.addEventListener("moved", (_) => init(editor));
//   });
// });
