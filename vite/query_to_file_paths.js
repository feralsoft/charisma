function* c_product(left, right) {
  for (let l of left) {
    for (let r of right) {
      yield [l, r];
    }
  }
}

function query_to_file_paths(query) {
  let out = [],
    lhs,
    rhs,
    name,
    value,
    args;
  switch (query.kind) {
    case "class_selector":
      return ["." + query.name];
    case "selector_modifier":
      out = [];
      for (let [a, b] of c_product(
        query_to_file_paths(query.lhs),
        query_to_file_paths(query.rhs)
      )) {
        out.push(a + "/&/" + b);
      }
      return out;
    case "child_selector":
      out = [](({ lhs, rhs } = query));
      for (let [a, b] of c_product(
        query_to_file_paths(lhs),
        query_to_file_paths(rhs)
      ))
        out.push(a + "/" + b);
      return out;
    case "attribute_selector_equals":
      ({ name, value } = query);
      return ["[" + name + '="' + value + '"]'];
    case "attribute_selector_contains":
      ({ name, value } = query);
      return ["[" + name + '*="' + value + '"]'];
    case "attribute_selector_no_match":
      ({ name } = query);
      return ["[" + name + "]"];
    case "element_selector":
      ({ name } = query);
      return [name];
    case "id_selector":
      ({ name } = query);
      return ["#" + name];
    case "pseudo_selector_without_args":
      ({ name } = query);
      return [":" + name];
    case "pseudo_selector_with_args":
      ({ name, args } = query);
      return [":" + name + "(" + args.map(compile).join(", ") + ")"];
    case "pseudo_element_selector":
      ({ name } = query);
      return ["::" + name];
    case "direct_descendent":
      ({ lhs, rhs } = query);
      if (lhs) {
        for (let [a, b] of c_product(
          query_to_file_paths(lhs),
          query_to_file_paths(rhs)
        ))
          out.push(a + "/>/" + b);
        return out;
      } else {
        return query_to_file_paths(rhs).map((x) => "/>/" + x);
      }

    case "sibling_selector":
      ({ lhs, rhs } = query);
      for (let [a, b] of c_product(
        query_to_file_paths(lhs),
        query_to_file_paths(rhs)
      ))
        out.push(a + "/~/" + b);
      return out;
    case "or":
      ({ lhs, rhs } = query);
      return [...query_to_file_paths(lhs), ...query_to_file_paths(rhs)];
  }
}

export default query_to_file_paths;
