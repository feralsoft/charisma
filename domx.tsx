declare let get: any;
declare let set: any;
type CSSRule = any;
declare let remove: any;
declare let React: any;

export default {
  name: "flex-preview",
  // query to search for `display: flex`
  query:
    "[data-kind=property]:has(> [data-attr=name] [data-value=display], > [data-attr=expr] [data-value=flex])",
  insert_after: (
    <div
      class="flex-direction-preview"
      data-flex-preview={get("flex-direction")}
      on:click={function* (self: CSSRule) {
        if (yield self.get("flex-direction") === "column") {
          yield self.remove("flex-direction");
        } else {
          yield self.set("flex-direction", "column");
        }
      }}
    />
  ),
};
