# recon_guest

Guest SDK for Recon WASM plugins. Provides bindings for the `recon-app` world which includes:

- **iced UI widgets** — re-exported from `igloo_guest`
- **Event bus** — `publish` and `subscribe` to topics via `recon:event-bus/bus`

## Usage

```toml
[dependencies]
igloo_guest = { workspace = true }
recon_guest = { workspace = true }
```

```rust
use igloo_guest::{Element, widgets::{button, text, column, container}};
use recon_guest::bus;

fn update(&mut self, message: Message) {
    match message {
        Message::Click => {
            let _ = bus::publish("my/topic", "hello");
        }
    }
}
```

Plugins use `igloo_guest::export_guest!` to export the iced view/update functions.
