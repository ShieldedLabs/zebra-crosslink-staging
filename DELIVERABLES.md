# Engineering Deliverables for Shielded Lab's Crosslink Deployment

This is a draft list of SL's engineering deliverables as part of our Crosslink Deployment project. This list may be refined as the project develops, both to provide greater precision or clarity around deliverables, or to add or potentially remove deliverables (ideally only in early phases of the project lifecycle).

## Code Products

The active list of [Code Deliverables](https://github.com/ShieldedLabs/zebra-crosslink/labels/Code%20Deliverable) is evolving on GitHub, and includes these major components:

- a full node implementation of the Crosslink protocol - intended for use by all full node users
  - [GH #105](https://github.com/ShieldedLabs/zebra-crosslink/issues/105) augmentation of existing RPC APIs for blocks and transactions to include finality status
  - [GH #104](https://github.com/ShieldedLabs/zebra-crosslink/issues/104) new RPC APIs related to Proof-of-Stake: finalizer identities, staking weights, delegation info
  - [GH #172](https://github.com/ShieldedLabs/zebra-crosslink/issues/172) new RPC APIs related to Crosslink: contingency mode, finality traversal
- [GH #154](https://github.com/ShieldedLabs/zebra-crosslink/issues/154) a basic working commandline finalizer (probably as a configured mode of the full node) - potentially used by production finalizers
- [GH #106](https://github.com/ShieldedLabs/zebra-crosslink/issues/106) a basic, minimal stake delegation tool - this will not be an easily usable or recommended product for end users
- [GH #112](https://github.com/ShieldedLabs/zebra-crosslink/issues/112) all necessary Continuous Integration / Continuous Deployment source code is an explicit code deliverable

All code products will be open source and suitable for merging or bundling with [zebra](https://github.com/ZcashFoundation/zebra) source and binary distributions.

All code products will be first developed and released as testnet prototypes which are then matured into production-ready status as the target activation date approaches. 

## Technical Documentation

The active list of [Documentation Deliverables](https://github.com/ShieldedLabs/zebra-crosslink/labels/Documentation%20Deliverable) is evolving on GitHub, and includes these important components:

- User facing documentation about Crosslink finality and safety considerations, including links to assessments, analyses, and audits by other parties.
- User facing documentation for users of all code deliverables.
- Design notes on prototype design trade-offs, delegation secure UX considerations, and a "Staking in Zcash" primer for PoS protocol-expert audiences. 
- A Productionization Plan describing the process to develop the prototype into a safe mainnet production deployment and activation process.
- Process documentation for code releases, testnet deployments, and documentation releases intended for current and future maintainers, and developers.

## External Validation Partnerships

While not deliverables per se, a key strategy we'll use for gauging our success is to establish working relationships with partner teams which are representative stakeholders for these categories:

- Wallet developers
- Finalizers
- Exchanges
- Bridges, DEXes, or other protocol-integration product teams

Our success criteria require user-facing in-production products in these categories (with the first three prioritized). In order to meet these criteria, we need to engage with high quality partners to serve as representative stakeholders relatively early in the process, as the prototype approaches a usable state. We track ongoing [Partner Validation](https://github.com/ShieldedLabs/zebra-crosslink/labels/Partner%20Validation) goals on GitHub.
