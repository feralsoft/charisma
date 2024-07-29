export function once(elem, event_name, handler) {
  elem.addEventHandler(event_name, function self(e) {
    handler(e);
    elem.removeEventHandler(event_name);
  });
}
