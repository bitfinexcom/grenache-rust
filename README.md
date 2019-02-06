# [Grenache](https://github.com/bitfinexcom/grenache) Rust HTTP implementation

<img src="https://raw.githubusercontent.com/bitfinexcom/grenache-nodejs-http/master/logo.png" width="15%" />

Grenache is a micro-framework for connecting microservices. Its simple and optimized for performance.

Internally, Grenache uses Distributed Hash Tables (DHT, known from Bittorrent) for Peer to Peer connections. You can find more details how Grenche internally works at the [Main Project Homepage](https://github.com/bitfinexcom/grenache)

 - [Setup](#setup)
 - [Examples](#examples)

## Setup

### Install
Add grenache-rust to your `cargo.toml` file:
```
grenache-rust = { git = "https://github.com/bitfinexcom/grenache-rust.git" }
```

### Other Requirements

Install `Grenache Grape`: https://github.com/bitfinexcom/grenache-grape:

```bash
npm i -g grenache-grape
```

```
// Start 2 Grapes
grape --dp 20001 --aph 30001 --bn '127.0.0.1:20002'
grape --dp 20002 --aph 40001 --bn '127.0.0.1:20001'
```

### Examples
The following will annonce the `rest:net:util` service on port 31337 and then confirm that the service can be looked up using the `GrenacheClient` object.
```rust
extern crate grenache_rust;

use grenache_rust::GrenacheClient;
use grenache_rust::Grenache;
use std::{thread, time};

fn main(){
    let service = "rest:net:util";
    let service_port = 31_337u16;
    let api_url = "http://127.0.0.1:30001";
    let mut client = GrenacheClient::new(api_url);
    client.start_announcing(service, service_port ).unwrap();
    thread::sleep(time::Duration::from_secs(1));
    println!("Service at: {}",client.lookup(service).unwrap());
    client.stop_announcing(service).unwrap();
}
```

## Licensing
Licensed under Apache License, Version 2.0
 
### Licenses for dependencies
- Rust - [
Rust is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0), with portions covered by various BSD-like licenses.](https://github.com/rust-lang/rust#license)
- serde_json - [Apache License, Version 2.0 or MIT](https://github.com/serde-rs/json/tree/493bad102fa42fea6f1365fc5809bacfbd423adb#license)
- uuid - [Apache License, Version 2.0 or MIT](https://github.com/uuid-rs/uuid/blob/cdd5528d46ec8f7c7595615b366b4fe301818f9f/COPYRIGHT)
- log - [Apache License, Version 2.0](https://github.com/rust-lang-nursery/log/blob/1a9a8275f5d84d50b756437524da5ff8273ef99b/LICENSE-APACHE) or [MIT](https://github.com/rust-lang-nursery/log/blob/1a9a8275f5d84d50b756437524da5ff8273ef99b/LICENSE-MIT)
- hyper - [MIT license](https://github.com/hyperium/hyper/blob/fdd04134187fe0c1a0b446577300e0bb391183ae/LICENSE)
- tokio - [MIT license](https://github.com/tokio-rs/tokio/blob/11e2af66a82c1cb710f9a07ae9450fcb25e67a1e/LICENSE)
