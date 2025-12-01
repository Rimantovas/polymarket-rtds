# Polymarket Real-Time Data Streaming SDK (Rust)

> **⚠️ Early Alpha**: Some stuff may not work. Please open an issue if you encounter problems.

A Rust port of the official [Polymarket Real-Time Data Streaming TypeScript SDK](https://github.com/Polymarket/real-time-data-client). This client provides a wrapper to connect to the `real-time-data-streaming` WebSocket service.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
polymarket-rtds = "0.1.0"
```

### TLS features

The crate defaults to `native-tls`, but you can pick any TLS backend supported by `tokio-tungstenite` via features: `native-tls`, `native-tls-vendored`, `rustls-tls-native-roots`, `rustls-tls-webpki-roots`. Disable default features to opt into a different backend, e.g.:

```toml
[dependencies]
polymarket-rtds = { version = "0.1.0", default-features = false, features = ["rustls-tls-webpki-roots"] }
```

## Quick Start

Here's a quick example of how to connect to the service and start receiving messages:

```rust
use polymarket_rtds::{
    RealTimeDataClient, Message, Subscription,
    Topic, MessageType, SubscriptionFilter
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let mut client = RealTimeDataClient::new();

    // Connect to the WebSocket service
    client.connect().await?;

    // Subscribe to a topic using type-safe enums
    let subscription = Subscription::new(Topic::Comments, MessageType::All)
        .with_filter(SubscriptionFilter::parent_entity(100, "Event"))?;

    client.subscribe(vec![subscription]).await?;

    // Handle incoming messages
    while let Some(message) = client.recv().await {
        match message {
            Ok(msg) => {
                println!("Topic: {}, Type: {}, Payload: {:?}",
                    msg.topic, msg.message_type, msg.payload);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

## Usage

### Subscribing to Topics

Subscribe to 'trades' messages from the topic 'activity' and to all comments messages:

```rust
use polymarket_rtds::{Subscription, Topic, MessageType};

// Subscribe to trades
let subscription = Subscription::new(Topic::Activity, MessageType::Trades);
client.subscribe(vec![subscription]).await?;

// Subscribe to all comment types
let subscription = Subscription::new(Topic::Comments, MessageType::All);
client.subscribe(vec![subscription]).await?;
```

### Using Filters

You can filter subscriptions using the `SubscriptionFilter` enum:

```rust
use polymarket_rtds::{Subscription, Topic, MessageType, SubscriptionFilter};

// Filter by market slug
let subscription = Subscription::new(Topic::Activity, MessageType::Trades)
    .with_filter(SubscriptionFilter::market_slug("will-bitcoin-hit-100k"))?;
client.subscribe(vec![subscription]).await?;

// Filter crypto prices by symbol
let subscription = Subscription::new(Topic::CryptoPrices, MessageType::Update)
    .with_filter(SubscriptionFilter::symbol("BTCUSDT"))?;
client.subscribe(vec![subscription]).await?;

// Filter CLOB market by token IDs
let subscription = Subscription::new(Topic::ClobMarket, MessageType::PriceChange)
    .with_filter(SubscriptionFilter::token_ids(vec!["100".to_string(), "200".to_string()]))?;
client.subscribe(vec![subscription]).await?;
```

### Unsubscribing from Topics

Unsubscribe from the new trades messages of the topic 'activity':

```rust
let subscription = Subscription::new(Topic::Activity, MessageType::Trades);
client.unsubscribe(vec![subscription]).await?;
```

### Disconnecting

Disconnect from the WebSocket server:

```rust
client.disconnect().await?;
```

## Authenticated Topics

Some topics require CLOB authentication credentials. Here's how to use them:

```rust
use polymarket_rtds::{Subscription, Topic, MessageType, ClobApiKeyCreds};

let auth = ClobApiKeyCreds {
    key: "your-api-key".to_string(),
    secret: "your-api-secret".to_string(),
    passphrase: "your-passphrase".to_string(),
};

let subscription = Subscription::new(Topic::ClobUser, MessageType::All)
    .with_clob_auth(auth);

client.subscribe(vec![subscription]).await?;
```

## Available Topics and Types

| Topic                     | Type               | Auth     | Filters                                                         | Description              |
| ------------------------- | ------------------ | -------- | --------------------------------------------------------------- | ------------------------ |
| `activity`                | `trades`           | -        | `{"event_slug":"string"}` OR `{"market_slug":"string"}`         | Real-time trade activity |
| `activity`                | `orders_matched`   | -        | `{"event_slug":"string"}` OR `{"market_slug":"string"}`         | Matched orders           |
| `comments`                | `comment_created`  | -        | `{"parentEntityID":number,"parentEntityType":"Event / Series"}` | New comments             |
| `comments`                | `comment_removed`  | -        | `{"parentEntityID":number,"parentEntityType":"Event / Series"}` | Removed comments         |
| `comments`                | `reaction_created` | -        | `{"parentEntityID":number,"parentEntityType":"Event / Series"}` | New reactions            |
| `comments`                | `reaction_removed` | -        | `{"parentEntityID":number,"parentEntityType":"Event / Series"}` | Removed reactions        |
| `rfq`                     | `request_created`  | -        | -                                                               | RFQ request created      |
| `rfq`                     | `request_edited`   | -        | -                                                               | RFQ request edited       |
| `rfq`                     | `request_canceled` | -        | -                                                               | RFQ request canceled     |
| `rfq`                     | `request_expired`  | -        | -                                                               | RFQ request expired      |
| `rfq`                     | `quote_created`    | -        | -                                                               | RFQ quote created        |
| `rfq`                     | `quote_edited`     | -        | -                                                               | RFQ quote edited         |
| `rfq`                     | `quote_canceled`   | -        | -                                                               | RFQ quote canceled       |
| `rfq`                     | `quote_expired`    | -        | -                                                               | RFQ quote expired        |
| `crypto_prices`           | `update`           | -        | `{"symbol":"BTCUSDT"}`                                          | Crypto price updates     |
| `crypto_prices_chainlink` | `update`           | -        | `{"symbol":"BTCUSDT"}`                                          | Chainlink crypto prices  |
| `equity_prices`           | `update`           | -        | `{"symbol":"AAPL"}`                                             | Equity price updates     |
| `clob_user`               | `order`            | ClobAuth | -                                                               | User order updates       |
| `clob_user`               | `trade`            | ClobAuth | -                                                               | User trade updates       |
| `clob_market`             | `price_change`     | -        | `["100","200",...]` (mandatory)                                 | Market price changes     |
| `clob_market`             | `agg_orderbook`    | -        | `["100","200",...]`                                             | Aggregated orderbook     |
| `clob_market`             | `last_trade_price` | -        | `["100","200",...]`                                             | Last trade price         |
| `clob_market`             | `tick_size_change` | -        | `["100","200",...]`                                             | Tick size changes        |
| `clob_market`             | `market_created`   | -        | -                                                               | New market created       |
| `clob_market`             | `market_resolved`  | -        | -                                                               | Market resolved          |

## Examples

### Subscribe to Market Activity

```rust
use polymarket_rtds::{
    RealTimeDataClient, Subscription, Topic, MessageType, SubscriptionFilter
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RealTimeDataClient::new();
    client.connect().await?;

    let subscription = Subscription::new(Topic::Activity, MessageType::Trades)
        .with_filter(SubscriptionFilter::market_slug("will-bitcoin-hit-100k"))?;

    client.subscribe(vec![subscription]).await?;

    while let Some(Ok(message)) = client.recv().await {
        println!("New trade: {:?}", message.payload);
    }

    Ok(())
}
```

### Monitor Crypto Prices

```rust
use polymarket_rtds::{
    RealTimeDataClient, Subscription, Topic, MessageType, SubscriptionFilter
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RealTimeDataClient::new();
    client.connect().await?;

    let subscription = Subscription::new(Topic::CryptoPrices, MessageType::Update)
        .with_filter(SubscriptionFilter::symbol("BTCUSDT"))?;

    client.subscribe(vec![subscription]).await?;

    while let Some(Ok(message)) = client.recv().await {
        println!("BTC price update: {:?}", message.payload);
    }

    Ok(())
}
```

### Track User Orders (Authenticated)

```rust
use polymarket_rtds::{
    RealTimeDataClient, Subscription, Topic, MessageType, ClobApiKeyCreds
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RealTimeDataClient::new();
    client.connect().await?;

    let auth = ClobApiKeyCreds {
        key: std::env::var("CLOB_API_KEY")?,
        secret: std::env::var("CLOB_API_SECRET")?,
        passphrase: std::env::var("CLOB_PASSPHRASE")?,
    };

    let subscription = Subscription::new(Topic::ClobUser, MessageType::Order)
        .with_clob_auth(auth);

    client.subscribe(vec![subscription]).await?;

    while let Some(Ok(message)) = client.recv().await {
        println!("Order update: {:?}", message.payload);
    }

    Ok(())
}
```

## Message Schemas

For detailed message schemas for each topic type, please refer to the [official TypeScript SDK documentation](https://github.com/Polymarket/real-time-data-client).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

This is a Rust port of the official [Polymarket Real-Time Data Streaming TypeScript SDK](https://github.com/Polymarket/real-time-data-client).
