[![Try on playground](https://img.shields.io/badge/Playground-Node_Template-brightgreen?logo=Parity%20Substrate)](https://playground.substrate.dev/?deploy=node-template) [![Matrix](https://img.shields.io/matrix/substrate-technical:matrix.org)](https://matrix.to/#/#substrate-technical:matrix.org)

A fresh FRAME-based [Substrate](https://www.substrate.io/) node, ready for hacking :rocket:

# Project Overview

This repository contains a research-oriented modification of the Substrate Node Template, developed as an experimental platform for investigating consensus mechanisms in Polkadot-like networks.

The project is based on the Polkadot consensus model but introduces non-trivial modifications to the original BABE + GRANDPA architecture. Specifically, the original consensus pipeline has been redesigned by replacing BABE block production and GRANDPA finality with a PRAOS-style consensus protocol, enabling probabilistic leader election without an explicit finality gadget.

In addition, this node integrates an adjust module, derived from the analytical model proposed in our accompanying research work (arXiv:2601.00370). The module is designed to dynamically adjust protocol behavior according to network conditions and protocol parameters defined in the paper.

Experimental Status

✅ Consensus logic migrated from BABE to PRAOS

✅ Session-based epoch transitions implemented via the session pallet

✅ Dynamic validator set updates supported (without stake ratio adjustment)

✅ Node successfully operates in a 3-node local network

❌ No GRANDPA-based finality mechanism is currently implemented

❌ Large-scale multi-node experiments and stress testing have not yet been conducted

At the current stage, the system demonstrates functional block production and validator rotation under a small-scale network setting. While no explicit finality gadget is deployed, blocks are produced continuously, allowing observation of fork behavior and protocol liveness under PRAOS-style assumptions.

Research Context

This codebase serves as an experimental infrastructure rather than a production-ready blockchain implementation. It is primarily intended to:

Validate theoretical assumptions proposed in the paper

Explore the feasibility of PRAOS-style consensus in a Substrate-based environment

Observe protocol behavior under simplified network conditions

Act as a foundation for future experiments involving larger validator sets and adversarial scenarios

The experiments related to this repository were partially completed and not included in the final version of the paper. However, the implementation reflects the concrete system-level exploration that motivated the theoretical analysis.

Limitations and Future Work

The current implementation has several known limitations:

No block finality mechanism (e.g., GRANDPA) is included

Experiments are limited to small-scale (3-node) deployments

No performance benchmarking or stress testing has been conducted

Adversarial network conditions have not yet been simulated

Future work includes extending the system to multi-node environments, conducting performance and liveness evaluations, and integrating finality or hybrid consensus mechanisms for comparison.

Branch Information

Active Branch: origin/babe

Target: Transition consensus from BABE to PRAOS

Author Joshua022

contact: joshuashen183@gmail.com



## Getting Started

Follow the steps below to get started with the Node Template, or get it up and running right from
your browser in just a few clicks using [Playground](https://playground.substrate.dev/)
:hammer_and_wrench:

### Using Nix

Install [nix](https://nixos.org/) and optionally [direnv](https://github.com/direnv/direnv) and
[lorri](https://github.com/target/lorri) for a fully plug and play experience for setting up the
development environment. To get all the correct dependencies activate direnv `direnv allow` and
lorri `lorri shell`.

### Rust Setup

First, complete the [basic Rust setup instructions](./doc/rust-setup.md).

### Run

Use Rust's native `cargo` command to build and launch the template node:

```sh
cargo run --release -- --dev --tmp
```

### Build

The `cargo run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
cargo build --release
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/node-template -h
```

## Run

The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/node-template --dev
```

Purge the development chain's state:

```bash
./target/release/node-template purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_BACKTRACE=1 ./target/release/node-template -ldebug --dev
```

### Connect with Polkadot-JS Apps Front-end

Once the node template is running locally, you can connect it with **Polkadot-JS Apps** front-end
to interact with your chain. [Click
here](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) connecting the Apps to your
local node template.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to our
[Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

Start a test net use `alice.sh` & `bob.sh`, remember to reset boot node id before start `bob.sh`.
### Interact With Local Chain
JS codes is used to interact with the local chain.
```shell
node scripts/javascripts/ *.js
```

### Add Validator from seed

Start node with seed by
```shell
bash /scripts/cmd/unknown_node.sh
```
Add session key to the validator set by
```shell
node /scripts/javascripts/extrincs session_addKey.js
```

Wait for log to show keys has been successfully add to validator set

```
2022-03-05 15:30:30 ********************************
2022-03-05 15:30:30 Receiving extrincs of adding key
2022-03-05 15:30:30 Adding key success
2022-03-05 15:30:30 KeyOwner contains 3
2022-03-05 15:30:30 ********************************
```

And wait for the next session begin, and it could take a while

When success, you can see log as
```
2022-03-05 15:46:06 🙌 Starting consensus session on top of parent 0x3ba5fa5704a0b8f6c3283fba9459b3cf6d3c8c522bd03e2391b3ac48b3a583d7
2022-03-05 15:46:06 Last Block slot[274411061]. Total difference [16]. Epoch Duration [20]
2022-03-05 15:46:06 Block[36]
2022-03-05 15:46:06 🎁 Prepared block for proposing at 36 (0 ms) [hash: 0x85517be92e111f221b22b5dd7b6caa356439bdf3fce35fb75376c3f6e38350a2; parent_hash: 0x3ba5…83d7; extrinsics (1): [0x0e05…c60a]]
2022-03-05 15:46:06 🔖 Pre-sealed block for proposal at 36. Hash now 0x0aa7da5f758fc97cc4503253baac248e9e9260218dc69cc08fc7dded27ebc6aa, previously 0x85517be92e111f221b22b5dd7b6caa356439bdf3fce35fb75376c3f6e38350a2.
```

More info at [polkadot.{js}](https://polkadot.js.org/docs/)
## Template Structure

A Substrate project such as this consists of a number of components that are spread across a few
directories.

### Node

A blockchain node is an application that allows users to participate in a blockchain network.
Substrate-based blockchain nodes expose a number of capabilities:

- Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking stack to allow the
  nodes in the network to communicate with one another.
- Consensus: Blockchains must have a way to come to
  [consensus](https://substrate.dev/docs/en/knowledgebase/advanced/consensus) on the state of the
  network. Substrate makes it possible to supply custom consensus engines and also ships with
  several consensus mechanisms that have been built on top of
  [Web3 Foundation research](https://research.web3.foundation/en/latest/polkadot/NPoS/index.html).
- RPC Server: A remote procedure call (RPC) server is used to interact with Substrate nodes.

There are several files in the `node` directory - take special note of the following:

- [`chain_spec.rs`](./node/src/chain_spec.rs): A
  [chain specification](https://substrate.dev/docs/en/knowledgebase/integrate/chain-spec) is a
  source code file that defines a Substrate chain's initial (genesis) state. Chain specifications
  are useful for development and testing, and critical when architecting the launch of a
  production chain. Take note of the `development_config` and `testnet_genesis` functions, which
  are used to define the genesis state for the local development chain configuration. These
  functions identify some
  [well-known accounts](https://substrate.dev/docs/en/knowledgebase/integrate/subkey#well-known-keys)
  and use them to configure the blockchain's initial state.
- [`service.rs`](./node/src/service.rs): This file defines the node implementation. Take note of
  the libraries that this file imports and the names of the functions it invokes. In particular,
  there are references to consensus-related topics, such as the
  [longest chain rule](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#longest-chain-rule),
  the [Aura](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#aura) block authoring
  mechanism and the
  [GRANDPA](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#grandpa) finality
  gadget.

After the node has been [built](#build), refer to the embedded documentation to learn more about the
capabilities and configuration parameters that it exposes:

```shell
./target/release/node-template --help
```

### Runtime

In Substrate, the terms
"[runtime](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#runtime)" and
"[state transition function](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#stf-state-transition-function)"
are analogous - they refer to the core logic of the blockchain that is responsible for validating
blocks and executing the state changes they define. The Substrate project in this repository uses
the [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) framework to construct a
blockchain runtime. FRAME allows runtime developers to declare domain-specific logic in modules
called "pallets". At the heart of FRAME is a helpful
[macro language](https://substrate.dev/docs/en/knowledgebase/runtime/macros) that makes it easy to
create pallets and flexibly compose them to create blockchains that can address
[a variety of needs](https://www.substrate.io/substrate-users/).

Review the [FRAME runtime implementation](./runtime/src/lib.rs) included in this template and note
the following:

- This file configures several pallets to include in the runtime. Each pallet configuration is
  defined by a code block that begins with `impl $PALLET_NAME::Config for Runtime`.
- The pallets are composed into a single runtime by way of the
  [`construct_runtime!`](https://crates.parity.io/frame_support/macro.construct_runtime.html)
  macro, which is part of the core
  [FRAME Support](https://substrate.dev/docs/en/knowledgebase/runtime/frame#support-library)
  library.

### Pallets

The runtime in this project is constructed using many FRAME pallets that ship with the
[core Substrate repository](https://github.com/paritytech/substrate/tree/master/frame) and a
template pallet that is [defined in the `pallets`](./pallets/template/src/lib.rs) directory.

A FRAME pallet is compromised of a number of blockchain primitives:

- Storage: FRAME defines a rich set of powerful
  [storage abstractions](https://substrate.dev/docs/en/knowledgebase/runtime/storage) that makes
  it easy to use Substrate's efficient key-value database to manage the evolving state of a
  blockchain.
- Dispatchables: FRAME pallets define special types of functions that can be invoked (dispatched)
  from outside of the runtime in order to update its state.
- Events: Substrate uses [events](https://substrate.dev/docs/en/knowledgebase/runtime/events) to
  notify users of important changes in the runtime.
- Errors: When a dispatchable fails, it returns an error.
- Config: The `Config` configuration interface is used to define the types and parameters upon
  which a FRAME pallet depends.

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

#### Using Makefile and Docker Compose

Build the Docker image using Makefile:

```bash
make image
```

This will create a Docker image named `substrate-node:0.0.15`. Then start the node using Docker Compose:

```bash
docker compose up
```

To run in detached mode (background):

```bash
docker compose up -d
```

To stop the node:

```bash
docker compose down
```

#### Using Docker Run Script

Alternatively, you can use the provided script to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command
(`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/node-template --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/node-template purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```

# ISSUES


## CARGO BUILD FAILED
When running `cargo build` using rust version `^1.70.0` may fail.
Change version to stable and change rust version under `1.70.0`,
known worked version is `1.56.0`
**Solution**
```shell
rustup toolchain remove nightly
rustup default 1.56.0
```
