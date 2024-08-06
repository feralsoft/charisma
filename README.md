# Charisma CSS editor

Charisma is a CSS editor that takes a different approach from text based editors.

## Running the project

### Prerequisites

- [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)
- Tauri cli

### Tauri CLI
To install the [Tauri CLI](https://tauri.app/v1/api/cli) run the following command on a terminal:
```shell
cargo install tauri-cli
```

### Start the dev server

The project is ran using the `cargo tauri` command to start it from the rust side.
To do that, run the following command on the root of the project:

```shell
cargo tauri dev
```

The project should run the `vite` server for node and then open a new window with the rust application.

## Building the project

Building the project is similar to running it. All happens from the rust side.
To build the project, run the following command on the root of the project:

```shell
cargo tauri build
```
