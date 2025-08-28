# Crosslink Functionality

This [zebra-crosslink](https://github.com/ShieldedLabs/zebra-crosslink) codebase is an in-progress implementation of the *Crosslink* hybrid Proof-of-Work / Proof-of-Stake consensus protocol described in [Zcash Trailing Finality Layer](https://electric-coin-company.github.io/tfl-book/).

**Warning:** All other chapters from 2 on are *not yet updated* for Crosslink.

## For Users

Presently, this implementation is not suitable for general users. There is no functioning testnet support. There is no guarantee Crosslink will activate on the Zcash network, although every indication we've seen is enthusiastic support from both the userbase and Zcash developer communities, so we expect it will be when the time is right.

## For Developers and Protocol Aficionados

While inspired by the [Zcash Trailing Finality Layer](https://electric-coin-company.github.io/tfl-book/) book, this project relies on a suite of scope, goals, and specifications to achieve a successful safe Mainnet deployment with a priority of quick time-to-market.

### Product Definition

Our project [SCOPING.md](crosslink/SCOPING.md) in this repository is a user-facing description of what we aim to accomplish.

(**TODO:** Incorporate user stories.)

Our engineering goals are defined in `DELIVERABLES.md` in this repository. (**TODO:** Not yet merged.)

### Design Documentation

As we proceed we intend to produce design documentation that is abstracted from implementation, but more comprehensive than the [TFL Book](https://electric-coin-company.github.io/tfl-book/) design because it encompasses all of a specific concrete integrated hybrid protocol with full PoW, BFT, and PoS scope.

- The keystone document for this is an in-progress [ZIP Draft](https://docs.google.com/document/d/1wSLLReAEe4cM60VMKj0-ze_EHS12DqVoI0QVW8H3X9E/edit?tab=t.0#heading=h.f0ehy0pxr01t) which will enter the formal [ZIP](https://zips.z.cash) process as it matures.
- Additionally, we are tracking _Architecture Design Rationales_ as we prototype the complete design. More detail is provided in the [Implementation Approach](#implementation-approach) section below.

### Protocol and Engineering Documentation

As we develop this specific instance, we are also aiming to produce _generic implementation specification_ in two forms:

- The other is a set of lower-level [Crosslink Implementation Requirements](https://docs.google.com/document/d/1YXalTGoezGH8GS1dknO8aK6eBFRq_Pq8LvDeho1KVZ8/edit?tab=t.0#heading=h.2sfs3jgpkfgx)  that engineers aim to adhere to in our implementation.

## Implementation Approach

The high level roadmap has two main phases: _Prototyping Phase_ and _Productionization Phase_.

Please see the Shielded Labs Crosslink [Roadmap Timeline](https://shieldedlabs.net/roadmap/) for current status, and our [Blogpost](https://shieldedlabs.net/crosslink-roadmap-q1-2025/) for more detailed milestone plans.

During the Prototype Phase we often choose the "simplest" design alternative when faced with choices. Simplicity has multiple axes: easy to explain, easy to implement, easiest implications to understand, etc... We prioritize time to market, then easiness to explain.

We intend to revisit each of these documented decisions, and any new ones we discover, during Milestone 5 where we begin the transition from prototype to production-ready.

### Design Adherence, Safety, & Analysis

Note the scope of these decisions may deviate from the more rigorous yet abstract [TFL Book §3.3 The Crosslink Construction](https://electric-coin-company.github.io/tfl-book/design/crosslink.html)!

This means the prototype _may_ violate some of the security assumptions or other goals of that design, so we make no assurances _the prototype is not safe_. We have three possible types of rationale for these deviations:

- We _intentionally_ prefer a different trade-off to the original design; in which case we should have very explicit rationale documentation about the difference (in an ADR). One example (**TODO**: not-yet-done) is our ADR-0001 selects a different approach to on-chain signatures than (**TODO**: link to TFL section).
- Something about the more abstract design makes assumptions we cannot uphold in practice with all of the other implementation constraints. In this case, we need to determine if the upstream design needs to be improved, or we need to alter our implementation constraints.
- As an expedient, we found it quicker and easier to do something different to get the prototype working, even though we believe the design makes better trade-offs. These are prime candidates for improvement during productionization to match the design, or else require persuasive rationale that a "short cut" is worth the trade-offs and risks.

### Architecture Design Rationales

We use an approach of documenting many design decisions via "Architecture Design Rationale" documents, inspired by the upstream development process. These are especially important for our Prototype-then-Productionize approach, since they record key decisions we should review for production readiness.

Currently ADRs are very rough and tracked on this [zebra-crosslink ADR Google Sheet](https://docs.google.com/spreadsheets/d/1X6dMxrkbWshhy8JwR7WkNC7JuN1J5YKNFzTcR-BolRo/edit?gid=0#gid=0).

Moving forward we will begin to incorporate them into this codebase.
