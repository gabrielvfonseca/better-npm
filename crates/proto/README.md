# bnpm-proto

DPR wire protocol client and server.

## Overview

Implements the Bnpm Protocol (DPR) wire format for peer-to-peer package transfer between nodes.

## Features

- DPR protocol client implementation
- DPR protocol server implementation
- Capsule and CID frame encoding
- Tower-based service layer

## Usage

```rust
use bnpm_proto::{DprClient, DprServer};

let mut client = DprClient::connect("dpr://node.bnpm.sh").await?;
let capsule = client.fetch_capsule(&cid).await?;
```
