## Adjust Module: Adaptive Round Length Mechanism

### Overview

This project implements an **adaptive round-length adjustment mechanism** inspired by our research on time-based synchrony in permissionless Proof-of-Stake systems.

The module is designed for a **Praos-style slot-based consensus protocol** built on Substrate, replacing the original **BABE + GRANDPA** design used in Polkadot.
Its purpose is to dynamically adjust the round (slot) length at the beginning of each epoch based on **observed network latency and on-chain adjustment messages**, without relying on a global synchronized clock.

This implementation corresponds to the **Adjust** component described in the accompanying paper and serves as an experimental prototype.

---

### Motivation

In permissionless PoS systems, participants may join and leave freely, and network conditions (latency, message delay) are inherently dynamic.
A static round length can lead to:

* wasted slots under low latency,
* frequent forks under high latency,
* degraded liveness or safety guarantees.

The **Adjust module** enables the protocol to **self-tune its round duration** using only on-chain information derived from recent epochs, improving robustness under varying network conditions.

---

### Epoch-Based Adjustment Strategy

At the **start of every epoch**, each participant computes a new round length using historical observations recorded on-chain.

To mitigate the **common prefix issue**, the adjustment algorithm selectively samples rounds from recent epochs:

* **(i)** the *first half* of rounds (R/2) in the **last epoch**
* **(ii)** the *second half* of rounds (R/2) in the **epoch before the last one**

If the current epoch is the first epoch, only case **(i)** is used.

Because the round lengths differ between these two segments, they are evaluated **separately** using different parameters:

* `t_round^1` for segment (i)
* `t_round^2` for segment (ii)

---

### Adjustment Formula

### Adjustment Formula

For segment (i), the new round length is computed as:

t_round_new^1 =
  ω1 * ( (1/m) * Σ t_i^a  - t_round^1 )
+ ω2 * ( (1/m) * Σ t_i^b  - t_round^1 )

Where:

- t_i^a: latency between sending and receiving block B_last
- t_i^b: delay of the corresponding adjustment message
- m: number of sampled rounds
- ω1, ω2: weighting parameters


Parameter constraints:

* ( 0 < ω2 < ω1 < 1 )
* ( ω1 + ω2 < 1 )

In practice, we typically set:

* ( ω1 = 0.3 )
* ( ω2 = 0.1 )

The **final round length** for the next epoch is obtained by averaging the results from segments (i) and (ii).

---

### Implementation Status

* ✅ Consensus modified from **BABE** to a **Praos-like protocol**
* ✅ Adjust module implemented and integrated into the runtime
* ✅ Session-based epoch transition supported
* ✅ Tested on a **3-node local network**, where nodes can:

  * produce blocks normally
  * adjust round length at epoch boundaries

---

### Limitations & Future Work

This implementation is a **research prototype** and has the following limitations:

* ❌ No large-scale multi-node deployment
* ❌ No stress testing or adversarial network simulation
* ❌ Adjustment parameters are static and not yet adaptive themselves

Future work includes:

* multi-node and WAN testing,
* formal evaluation of liveness and fork rates,
* integration with a lightweight finality mechanism,
* parameter learning or adaptive weighting strategies.

---

### Relation to the Paper

This module corresponds to the **Adjust** component described in:

> *Ouroboros AutoSyn: Time-Based Permissionless Synchrony Model for PoS*
> (see paper for formal definitions and analysis)

The current codebase serves as an **experimental validation** of the proposed adjustment mechanism.
