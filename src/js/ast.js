export const property = {
  name(property) {
    return property.querySelector('[data-attr="name"] > [data-value]').dataset
      .value;
  },
  value(property) {
    let value = property.querySelector('[data-attr="value"] > [data-kind]');
    assert(value.hasAttribute("data-string-value"));
    return value;
  },
};

export const value = {
  eq(a, b) {
    assert(a.dataset.stringValue !== "");
    assert(b.dataset.stringValue !== "");
    return a.dataset.stringValue === b.dataset.stringValue;
  },
};

export const rule = {
  properties(rule) {
    return rule.querySelectorAll(
      '[data-attr="properties"] > [data-kind="property"]',
    );
  },
};
