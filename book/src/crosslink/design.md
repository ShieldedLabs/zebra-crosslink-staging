# Crosslink Design

As we proceed we intend to produce design documentation that is abstracted from implementation, but more comprehensive than the [TFL Book](https://electric-coin-company.github.io/tfl-book/) design because it encompasses all of a specific concrete integrated hybrid protocol with full PoW, BFT, and PoS scope.

- The keystone document for this is an in-progress [ZIP Draft](https://docs.google.com/document/d/1wSLLReAEe4cM60VMKj0-ze_EHS12DqVoI0QVW8H3X9E/edit?tab=t.0#heading=h.f0ehy0pxr01t) which will enter the formal [ZIP](https://zips.z.cash) process as it matures.
- Additionally, we are tracking _Architecture Design Rationales_ as we prototype the complete design. More detail is provided in the [Implementation Approach](#implementation-approach) section below.

## Design Adherence, Safety, & Analysis

Note the scope of these decisions may deviate from the more rigorous yet abstract [TFL Book §3.3 The Crosslink Construction](https://electric-coin-company.github.io/tfl-book/design/crosslink.html)!

This means the prototype _may_ violate some of the security assumptions or other goals of that design, so we make no assurances _the prototype is not safe_. We have three possible types of rationale for these deviations:

- We _intentionally_ prefer a different trade-off to the original design; in which case we should have very explicit rationale documentation about the difference (in an ADR). One example (**TODO**: not-yet-done) is our ADR-0001 selects a different approach to on-chain signatures than (**TODO**: link to TFL section).
- Something about the more abstract design makes assumptions we cannot uphold in practice with all of the other implementation constraints. In this case, we need to determine if the upstream design needs to be improved, or we need to alter our implementation constraints.
- As an expedient, we found it quicker and easier to do something different to get the prototype working, even though we believe the design makes better trade-offs. These are prime candidates for improvement during productionization to match the design, or else require persuasive rationale that a "short cut" is worth the trade-offs and risks.

## Architecture Design Rationales

We use an approach of documenting many design decisions via "Architecture Design Rationale" documents, inspired by the upstream development process. These are especially important for our Prototype-then-Productionize approach, since they record key decisions we should review for production readiness.

Currently ADRs are very rough and tracked on this [zebra-crosslink ADR Google Sheet](https://docs.google.com/spreadsheets/d/1X6dMxrkbWshhy8JwR7WkNC7JuN1J5YKNFzTcR-BolRo/edit?gid=0#gid=0).

Moving forward we will begin to incorporate them into this codebase.
