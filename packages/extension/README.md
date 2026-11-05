# Aether VSCode Extension

![Aether Splash](https://github.com/AetherLabs/aether/raw/main/notes/aether_splash_8.avif)

This extension wraps functionality in Aether CLI to be used in your editor!

## Features:

- Auto-format RSX
- Convert HTML to RSX
- Convert HTML to Aether Component
- Format RSX

## Current commands:

### Convert HTML to RSX
Converts a selection of html to valid rsx.

### Convert HTML to Aether Component

Converts a selection of html to a valid Aether component with all SVGs factored out into their own module.

### Format RSX

Formats the current file as RSX.

## Building this extension from source
- make sure wasm-bindgen is installed to current version (cargo binstall wasm-bindgen-cli)
- run `npm install`
- run `npm run vsix`

# Working with Aether:

This overview provides a brief introduction to Aether. For a more in-depth guide, make sure to check out:

- [Getting Started](https://aetherlabs.com/learn/0.7/getting_started)
- [Book (0.6)](https://aetherlabs.com/learn/0.7/)

## Contributing
- Check out the website [section on contributing](https://aetherlabs.com/learn/0.7/beyond/contributing).
- Report issues on our [issue tracker](https://github.com/aetherlabs/aether/issues).
- [Join](https://discord.gg/XgGxMSkvUM) the discord and ask questions!


<a href="https://github.com/aetherlabs/aether/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=aetherlabs/aether&max=30&columns=10" />
</a>

## License
This project is licensed under the [MIT license].

[mit license]: https://github.com/AetherLabs/aether/blob/master/LICENSE-MIT

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Aether by you, shall be licensed as MIT, without any additional
terms or conditions.
