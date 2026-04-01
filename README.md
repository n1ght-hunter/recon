# Recon

A Windows-only native gaming companion app that runs alongside games with minimal CPU and RAM footprint. GUI built with [iced](https://github.com/iced-rs/iced).

Recon is a thin host shell. All functionality comes from plugins. The host provides the iced window, plugin loading, settings persistence, and an in-process pub/sub event bus.

## Setup

```bash
cargo build
cargo run
```

### WIT Dependencies

WASM plugin interfaces are defined in `wit/` and depend on the [`iced:app`](https://github.com/n1ght-hunter/igloo) WIT package hosted on GitHub Container Registry.

The `wit/deps/` directory is committed, so **no extra setup is needed to build**. If you need to update WIT dependencies, configure `wkg`:

```bash
wkg config --edit
```

Add the `iced` namespace mapping:

```toml
[namespace_registries]
iced = { registry = "iced", metadata = { preferredProtocol = "oci", "oci" = { registry = "ghcr.io", namespacePrefix = "n1ght-hunter/" } } }
```

Then fetch:

```bash
wkg wit fetch --wit-dir wit
```

### Building Plugins

```bash
mise run //plugins/test_plugin:build
```
