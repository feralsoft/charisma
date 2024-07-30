export function move_cursor_to_end_of_element(element) {
  // start garbage internet code to go the end of a text range
  let range = document.createRange();
  let selection = window.getSelection();
  range.setStart(element, element.childNodes.length);
  range.collapse(true);
  selection.removeAllRanges();
  selection.addRange(range);
  // end of garbage internet code
}
