# dash-proto

DPR wire protocol client and server.

## Overview

Implements the Dash Protocol (DPR) wire format for peer-to-peer package transfer between nodes.

## Features

- DPR protocol client implementation
- DPR protocol server implementation
- Capsule and CID frame encoding
- Tower-based service layer

## Usage

```rust
use dash_proto::{DprClient, DprServer};

let mut client = DprClient::connect("dpr://node.dash.sh").await?;
let capsule = client.fetch_capsule(&cid).await?;
```
