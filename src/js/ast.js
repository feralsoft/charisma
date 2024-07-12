export const property = {
  name(property) {
    return property.querySelector('[data-attr="name"] > [data-value]').dataset
      .value;
  },
  value(property) {
    return property.querySelector('[data-attr="value"] > [data-kind]');
  },
  is_commented(property) {
    return property.dataset.isCommented == "true";
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
    return Array.from(
      rule.querySelectorAll(
        '[data-attr="properties"] > [data-kind="property"]',
      ),
    );
  },
};
