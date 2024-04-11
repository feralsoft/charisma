import {
  Test,
  Chomp,
  Capture,
  TrimmedSafe,
  Trim,
  TrimBefore,
  Debug,
  Trimmed,
} from "./parse-utils.js";
let digit_regex = /^[+-]?([0-9]*[.])?[0-9]+/;

let attr = (value) => ({ attr_only: true, value });

function* parse_class_selector() {
  yield new Chomp(".");
  let name = yield new Capture(/^(\w|-)+/);
  return { kind: "class_selector", name };
}

function* parse_id_selector() {
  yield new Chomp("#");
  let name = yield new Capture(/^(\w|-)+/);
  return { kind: "id_selector", name };
}

function* parse_selector_arg() {
  if (yield new Test(/^\d+n/)) {
    let factor = yield new Capture(/^(\d)+n/, 1);
    if (yield new TrimmedSafe(new Test("+"))) {
      yield new Trimmed(new Chomp("+"));
      let offset = yield new Capture(/^\d+/);
      return { kind: "nth_expr_with_plus", factor, offset };
    } else {
      return { kind: "nth_expr_without_plus", factor };
    }
  } else {
    let skip_step = yield new Test(/(>|~| )/);
    return yield* parse_query(/^\s*(,|\))/, skip_step);
  }
}

function* parse_pseudo_selector_with_args() {
  yield new Chomp(":");
  let name = yield new Capture(/^(\w|-)+/);
  yield new Chomp("(");
  let args = [];
  while (!(yield new Test(")"))) {
    args.push(yield* parse_selector_arg());
    if (!(yield new TrimmedSafe(new Test(")")))) {
      yield new Trimmed(new Chomp(","));
    }
  }
  yield new TrimBefore(new Chomp(")"));
  return { kind: "pseudo_selector_with_args", name, args };
}

function* parse_pseudo_selector_without_args() {
  yield new Chomp(":");
  let name = yield new Capture(/^(\w|-)+/);
  return { kind: "pseudo_selector_without_args", name };
}

function* parse_pseudo_element_selector() {
  yield new Chomp("::");
  let name = yield new Capture(/^(\w|-)+/);
  return { kind: "pseudo_element_selector", name };
}

function* parse_attribute_selector_no_match() {
  yield new Chomp("[");
  let name = yield new Capture(/^(\w|-)+/);
  yield new Chomp("]");
  return { kind: "attribute_selector_no_match", name };
}

function* parse_attribute_selector_equals() {
  yield new Chomp("[");
  let name = yield new Capture(/^(\w|-)+/);
  yield new Trimmed(new Chomp("="));
  yield new Chomp('"');
  let value = yield new Capture(/^((?!").)+/);
  yield new Chomp('"');
  yield new TrimBefore(new Chomp("]"));
  return { kind: "attribute_selector_equals", name, value };
}

function* parse_attribute_selector_contains() {
  yield new Chomp("[");
  let name = yield new Capture(/^(\w|-)+/);
  yield new Trimmed(new Chomp("*="));
  yield new Chomp('"');
  let value = yield new Capture(/^((?!").)+/);
  yield new Chomp('"');
  yield new TrimBefore(new Chomp("]"));
  return { kind: "attribute_selector_contains", name, value };
}

function* parse_element_selector() {
  let name = yield new Capture(/^(\w|-)+/);
  return { kind: "element_selector", name };
}

function* parse_query_step() {
  if (yield new Test("*")) {
    yield new Chomp("*");
    return { kind: "wildcard" };
  } else if (yield new Test(".")) {
    return yield* parse_class_selector();
  } else if (yield new Test("#")) {
    return yield* parse_id_selector();
  } else if (yield new Test("::")) {
    return yield* parse_pseudo_element_selector();
  } else if (yield new Test(/^(\w|-)+/)) {
    return yield* parse_element_selector();
  } else if (yield new Test(/^:(\w|-)+\(/)) {
    return yield* parse_pseudo_selector_with_args();
  } else if (yield new Test(":")) {
    return yield* parse_pseudo_selector_without_args();
  } else if (yield new Test(/^\[(\w|-)+=".*"\]/)) {
    return yield* parse_attribute_selector_equals();
  } else if (yield new Test(/^\[(\w|-)+\*=".*"\]/)) {
    return yield* parse_attribute_selector_contains();
  } else if (yield new Test(/^\[(\w|-)+\]/)) {
    return yield* parse_attribute_selector_no_match();
  } else {
    (yield new Debug()).log();
    throw "parse fail";
  }
}

export function* parse_query(end_pattern, skip_step) {
  let query = null;
  // this is a fucking mess, please unify this with self_sub_rule
  if (yield new TrimmedSafe(new Test("&"))) {
    yield new Trim();
    yield new Chomp("&");
    query = { kind: "self_query", value: yield* parse_query_step() };
  } else if (!skip_step) {
    query = yield* parse_query_step();
  }
  while (!(yield new Test(end_pattern))) {
    let kind = null;
    if (yield new TrimmedSafe(new Test(">"))) {
      yield new Trimmed(new Chomp(">"));
      kind = "direct_descendent";
    } else if (yield new TrimmedSafe(new Test(","))) {
      yield new Trimmed(new Chomp(","));
      let rhs = yield* parse_query(end_pattern);
      return { kind: "or", lhs: query, rhs };
    } else if (yield new TrimmedSafe(new Test("~"))) {
      yield new Trimmed(new Chomp("~"));
      kind = "sibling_selector";
    } else if (yield new TrimmedSafe(new Test("+"))) {
      yield new Trimmed(new Chomp("+"));
      kind = "adjacent_sibling_selector";
    } else if (yield new Test(" ")) {
      yield new Trim();
      kind = "child_selector";
    } else if (query.rhs) {
      // `a > b"hover"` => `a > (b"hover")` not `(a > b)"hover"`
      query = {
        ...query,
        rhs: {
          kind: "selector_modifier",
          lhs: query.rhs,
          rhs: yield* parse_query_step(),
          modifier: attr(true),
        },
      };
      continue;
    } else {
      kind = "selector_modifier";
    }
    let rhs = yield* parse_query_step();
    query = { kind, lhs: query, rhs, modifier: attr(true) };
  }
  return query;
}

function* parse_px_value() {
  let value = yield new Capture(digit_regex);
  yield new Chomp("px");
  return { kind: "px", value };
}

function* parse_vw_value() {
  let value = yield new Capture(digit_regex);
  yield new Chomp("vw");
  return { kind: "vw", value };
}

function* parse_vh_value() {
  let value = yield new Capture(digit_regex);
  yield new Chomp("vh");
  return { kind: "vh", value };
}

function* parse_pct_value() {
  let value = yield new Capture(digit_regex);
  yield new Chomp("%");
  return { kind: "pct", value };
}

function* parse_rgb_value() {
  yield new Chomp("rgb");
  yield new Trimmed(new Chomp("("));
  let r = yield new Capture(digit_regex);
  yield new Trimmed(new Chomp(","));
  let g = yield new Capture(digit_regex);
  yield new Trimmed(new Chomp(","));
  let b = yield new Capture(digit_regex);
  yield new Trimmed(new Chomp(")"));
  return { kind: "rgb", r, g, b };
}

function* parse_rgba_value() {
  yield new Chomp("rgba");
  yield new Trimmed(new Chomp("("));
  let r = yield new Capture(digit_regex);
  yield new Trimmed(new Chomp(","));
  let g = yield new Capture(digit_regex);
  yield new Trimmed(new Chomp(","));
  let b = yield new Capture(digit_regex);
  yield new Trimmed(new Chomp(","));
  let a = yield new Capture(digit_regex);
  yield new Trimmed(new Chomp(")"));
  return { kind: "rgba", r, g, b, a };
}

let hex_part_regex = /^[a-fA-F0-9]{2}/;
let hex_part_regex_1 = /^[a-fA-F0-9]{1}/;

function* parse_hex_value() {
  yield new Chomp("#");
  if (yield new Test(/^[a-fA-F0-9]{3}\s*;/)) {
    let r = yield new Capture(hex_part_regex_1);
    let g = yield new Capture(hex_part_regex_1);
    let b = yield new Capture(hex_part_regex_1);
    return { kind: "hex_short", r, g, b };
  } else {
    let r = yield new Capture(hex_part_regex);
    let g = yield new Capture(hex_part_regex);
    let b = yield new Capture(hex_part_regex);
    if (yield new Test(hex_part_regex)) {
      let a = yield new Capture(hex_part_regex);
      return { kind: "hex_a", r, g, b, a };
    } else {
      return { kind: "hex", r, g, b };
    }
  }
}

function* parse_text_value() {
  let value = yield new Capture(/^[^; ]+/);
  return { kind: "plain_text", value };
}

function* parse_rem_value() {
  let value = yield new Capture(digit_regex);
  yield new Chomp("rem");
  return { kind: "rem", value };
}

function* parse_em_value() {
  let value = yield new Capture(digit_regex);
  yield new Chomp("em");
  return { kind: "em", value };
}

function* parse_var_value() {
  yield new Chomp("var");
  yield new Trimmed(new Chomp("("));
  let name = yield new Capture(/^--(\w|-)+/);
  yield new Trimmed(new Chomp(")"));
  return { kind: "var", name };
}

function* parse_var_with_default_value() {
  yield new Chomp("var");
  yield new Trimmed(new Chomp("("));
  let name = yield new Capture(/^--(\w|-)+/);
  yield new Trimmed(new Chomp(","));
  let default_value = yield* parse_value_step();
  yield new Trimmed(new Chomp(")"));
  return { kind: "var_with_default", name, default_value };
}

function* parse_string_value() {
  let value = yield new Capture(/^".*"/);
  return { kind: "string", value };
}

function* parse_calc_value() {
  yield new Chomp("calc");
  yield new Trimmed(new Chomp("("));
  let expr = yield* parse_value_step();
  while (!(yield new TrimmedSafe(new Test(")")))) {
    if (yield new TrimmedSafe(new Test("*"))) {
      yield new Trimmed(new Chomp("*"));
      expr = { kind: "multiply", lhs: expr, rhs: yield* parse_value_step() };
    } else if (yield new TrimmedSafe(new Test("-"))) {
      yield new Trimmed(new Chomp("-"));
      expr = { kind: "subtract", lhs: expr, rhs: yield* parse_value_step() };
    } else {
      expr
        .log()(yield debug())
        .log();
      throw "invalid operator";
    }
  }
  yield new Trimmed(new Chomp(")"));
  return { kind: "calc", expr };
}

function* parse_num_value() {
  let value = yield new Capture(digit_regex);
  return { kind: "num", value };
}

export function* parse_value_step() {
  if (yield new Test("rgba")) {
    return yield* parse_rgba_value();
  } else if (yield new Test("rgb")) {
    return yield* parse_rgb_value();
  } else if (yield new Test("#")) {
    return yield* parse_hex_value();
  } else if (yield new Test('"')) {
    return yield* parse_string_value();
  } else if (yield new Test(new RegExp(digit_regex.source + "px"))) {
    return yield* parse_px_value();
  } else if (yield new Test(new RegExp(digit_regex.source + "vw"))) {
    return yield* parse_vw_value();
  } else if (yield new Test(new RegExp(digit_regex.source + "vh"))) {
    return yield* parse_vh_value();
  } else if (yield new Test(new RegExp(digit_regex.source + "rem"))) {
    return yield* parse_rem_value();
  } else if (yield new Test(new RegExp(digit_regex.source + "em"))) {
    return yield* parse_em_value();
  } else if (yield new Test(new RegExp(digit_regex.source + "%"))) {
    return yield* parse_pct_value();
  } else if (yield new Test(/^var\(--(\w|-)+\)/)) {
    return yield* parse_var_value();
  } else if (yield new Test(digit_regex)) {
    return yield* parse_num_value();
  } else if (yield new Test("var")) {
    // .. this should probably just be done in parse_var_value
    return yield* parse_var_with_default_value();
  } else if (yield new Test("calc")) {
    return yield* parse_calc_value();
  } else {
    return yield* parse_text_value();
  }
}

export function* parse_value(property_name) {
  // fonts can have spaces in them, but other values shouldn't
  if (property_name == "font-family") {
    let value = yield new Capture(/^((?!;).)*/);
    return { kind: "plain_text", value };
  }

  let parts = [yield* parse_value_step()];
  // TODO ; or }
  if (!(yield new Test(/.*;/))) throw "expected `;`";

  while (!(yield new TrimmedSafe(new Test(";")))) {
    yield new Trim();
    parts.push(yield* parse_value_step());
  }

  if (parts.length == 1) {
    return parts[0];
  } else {
    return { kind: "multi_part_value", parts };
  }
}

export function* parse_property() {
  let name = yield new Trimmed(new Capture(/^(\w|-)+/));
  yield new Trimmed(new Chomp(":"));
  let expr = yield* parse_value(name);
  yield new Trimmed(new Chomp(";"));
  return { kind: "property", name, expr };
}

function* parse_variable() {
  let name = yield new Trimmed(new Capture(/^--(\w|-)+/));
  yield new Trimmed(new Chomp(":"));
  let expr = yield* parse_value();
  yield new Trimmed(new Chomp(";"));
  return { kind: "variable", name, expr };
}

function* parse_self_sub_rule() {
  yield new Chomp("&");
  let skip_part = yield new Test(" ");
  let query = yield* parse_query(/^\s*{/, skip_part);
  yield new Trimmed(new Chomp("{"));
  let properties = [];
  while (!(yield new TrimmedSafe(new Test("}")))) {
    properties.push(yield* parse_statement());
  }
  yield new Trimmed(new Chomp("}"));
  return { kind: "rule", query, properties, self_sub_rule: attr(true) };
}

function* parse_sub_rule() {
  let query = yield* parse_query(/^\s*{/);
  yield new Trimmed(new Chomp("{"));
  let properties = [];
  while (!(yield new TrimmedSafe(new Test("}")))) {
    properties.push(yield* parse_statement());
  }
  yield new Trimmed(new Chomp("}"));
  return { kind: "rule", query, properties, sub_rule: attr(true) };
}

function* parse_statement() {
  if (yield new Test("//")) {
    return yield* parse_variable();
  } else if (yield new Test("&")) {
    return yield* parse_self_sub_rule();
  } else if (yield new Test(/^(\w|-)+\s*:/)) {
    return yield* parse_property();
  } else {
    return yield* parse_sub_rule();
  }
}

function* parse_keyframe() {
  let pct = yield new Trimmed(new Capture(/(\d+)%/, 1));
  yield new Trimmed(new Chomp("{"));
  let properties = [];
  while (!(yield new TrimmedSafe(new Test("}")))) {
    properties.push(yield* parse_statement());
  }
  yield new Trimmed(new Chomp("}"));
  return { kind: "keyframe", pct, properties };
}

export function* parse_rule() {
  yield new Trim();
  if (yield new Test("@")) {
    yield new Chomp("@");
    let at_rule_name = yield new Capture(/^\w+/);
    if (at_rule_name !== "keyframes")
      throw "@" + at_rule_name + " not supported";
    let frames = [];
    let name = yield new Trimmed(new Capture(/^\w+/));
    yield new Trimmed(new Chomp("{"));
    while (!(yield new TrimmedSafe(new Test("}")))) {
      frames.push(yield* parse_keyframe());
    }
    yield new Trimmed(new Chomp("}"));
    return { kind: "keyframes", name, frames };
  } else {
    let query = yield* parse_query(/^\s*{/);
    yield new Trimmed(new Chomp("{"));
    let properties = [];
    while (!(yield new TrimmedSafe(new Test("}")))) {
      properties.push(yield* parse_statement());
    }
    yield new Trimmed(new Chomp("}"));
    return { kind: "rule", query, properties };
  }
}
