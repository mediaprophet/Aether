<div>
  <h1>📦✨ Aether CLI</h1>
  <p><strong>Tooling to supercharge Aether projects</strong></p>
</div>

The **aether-cli** (inspired by wasm-pack and webpack) is a tool for getting Aether projects up and running.
It handles building, bundling, development and publishing to simplify development.

## Installation

### Install the stable version (recommended)

```shell
cargo install aether-cli
```

### Install the latest development build through git

To get the latest bug fixes and features, you can install the development version from git.
However, this is not fully tested.
That means you're probably going to have more bugs despite having the latest bug fixes.

```shell
cargo install --git https://github.com/AetherLabs/aether aether-cli
```

This will download the CLI from the master branch,
and install it in Cargo's global binary directory (`~/.cargo/bin/` by default).

### Install from local folder
Note: The CLI will fail to build projects in debug profile. This is currently under investigation.
```shell
cargo install --path .
```

## Get started

Use `dx new` to initialize a new Aether project.
It will be cloned from the [aether-template](https://github.com/AetherLabs/aether-template) repository.

Alternatively, you can specify the template path:

```shell
dx new --template gh:aetherlabs/aether-template
```

Run `dx --help` for a list of all the available commands.
Furthermore, you can run `dx <command> --help` to get help with a specific command.

## Aether config file

You can use the `Aether.toml` file for further configuration.
Some fields are mandatory, but the CLI tool will tell you which ones are missing.
You can create a `Aether.toml` with all fields already set using `dx config init project-name`,
or you can use this bare-bones template (only mandatory fields) to get started:

```toml
[application]
name = "project-name"
# Currently supported platforms: web, desktop
default_platform = "web"

# Optional: enable copying from a static directory (e.g. "public")
public_dir = "public"

[web.app]
title = "Hello"

[web.resource.dev]
```
