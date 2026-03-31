# recon_bus

In-process async topic-based pub/sub event bus with wildcard matching.

## Features

- **Topic wildcards** with `*` (single-level) and `**` (multi-level)
- **Latest-value semantics** — subscribers see only the most recent value, no queue backlog
- **Zero-copy fan-out** — payloads are `Arc<str>`, shared across subscribers without cloning
- **Thread-safe** — `Bus` is `Clone + Send + Sync`, publish is synchronous
- **Auto-unsubscribe** — dropping a `Subscription` cleans up automatically
- **Optional serde** — `serde` feature adds `publish_serde()` and `Envelope::deserialize()`

## Usage

```rust
use recon_bus::{Bus, Topic};

#[tokio::main]
async fn main() {
    let bus = Bus::new();

    // Subscribe with a wildcard filter
    let mut sub = bus.subscribe("game/*/status").unwrap();

    // Publish to a concrete topic
    bus.publish("game/valorant/status", r#"{"online": true}"#).unwrap();

    // Receive the latest value
    let envelope = sub.recv().await.unwrap();
    println!("{}: {}", envelope.topic, envelope.payload);
}
```

### With serde feature

```toml
[dependencies]
recon_bus = { workspace = true, features = ["serde"] }
```

```rust
use recon_bus::Bus;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct GameState { health: u32 }

let bus = Bus::new();
let mut sub = bus.subscribe("state").unwrap();

bus.publish_serde("state", &GameState { health: 100 }).unwrap();

let envelope = sub.recv().await.unwrap();
let state: GameState = envelope.deserialize().unwrap();
```

## Topic Matching

| Filter | Topic | Match? |
|--------|-------|--------|
| `game/valorant/status` | `game/valorant/status` | Yes |
| `game/*/status` | `game/apex/status` | Yes |
| `game/*/status` | `game/valorant/health` | No |
| `game/*/status` | `game/status` | No |
| `game/**` | `game/valorant/player/hp` | Yes |
| `game/**` | `game` | Yes |
| `**` | (anything) | Yes |
