[![Crates.io](https://img.shields.io/crates/v/postal-rs.svg?style=for-the-badge)](https://crates.io/crates/postal-rs)
[![Documentation](https://img.shields.io/badge/docs.rs-postal_rs-blue?style=for-the-badge)](https://docs.rs/postal-rs)

`postal-rs` is a libarry which wraps HTTP API of [Postal](https://postal.atech.media/).
It uses https://krystal.github.io/postal-api/controllers/messages.html as a source of documentation.

# Get started

```rust
use postal_rs::{Client, DetailsInterest, Message, SendResult};
use std::env;

#[tokio::main]
async fn main() {
   let address = env::var("POSTAL_ADDRESS").unwrap_or_default();
   let token = env::var("POSTAL_TOKEN").unwrap_or_default();

   let message = Message::default()
       .to(&["example@gmail.com".to_owned()])
       .from("test@yourserver.io")
       .subject("Hello World")
       .text("A test message");
   let client = Client::new(address, token).unwrap();
   let _ = client
       .send(message)
       .await
       .unwrap();
}
```
