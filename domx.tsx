declare let render: any;
declare let get: any;
declare let set: any;
declare let remove: any;
declare let React: any;

export default {
  name: "flex-preview",
  query:
    "[data-kind=property]:has(> [data-attr=name] [data-value=display], > [data-attr=expr] [data-value=flex])",
  insert_after: (
    <div
      class="flex-direction-preview"
      data-flex-preview={get("flex-direction")}
      on:click={function* () {
        if (yield get("flex-direction") === "column") {
          yield remove("flex-direction");
        } else {
          yield set("flex-direction", "column");
        }
      }}
    />
  ),
};
