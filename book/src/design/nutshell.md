# Crosslink Design in a Nutshell

This chapter covers the major components of the design. This presents the design positively, without rationales, trade-offs, safety anlyses, or other supporting information. This will eventually be entirely redundant with, or simply link to, a canonical [Zcash Improvement Proposal](https://zips.z.cash) submission. Treat this chapter as a provisional sketch.

It assumes and leverages familiarity with Zcash PoW and only describes new differences from today's mainnet.

**CAVEATS:** A lot of the terminology and descriptions will be adjusted as we go to follow conventions and specifications in the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf) and the [Zcash Improvement Proposals](https://zips.z.cash).

## Roles and Capabilities

We rely on several roles in this description along with their intended capabilities. The absence of an explicit capability description for a role implies that role should not be able to perform that capability.

## Changes to Transactions

A new transaction format is introduced which contains a new optional field for _staking actions_, which enable operations such as staking `ZEC` to a finalizer, beginning or completing an _unstaking_ action, or _redelegating_ an existing staking position to a different delegator.

The `ZEC` flowing into staking actions, or flowing out of completing unstaking transactions must balance as part of the transactions chain value pool balancing. See [The Zcash Protocol §4.17 Chain Value Pool Balances](https://zips.z.cash/protocol/protocol.pdf).

Additionally, we introduce a new restricting consensus rule on context-free transaction validity[^ctx-free-validity]:

[^ctx-free-validity]: A context-free transaction validity check may be performed on the bytes of the transaction itself without the need to access any chain state or index.

> **Crosslink Staking Orchard Restriction:**
>
> A transaction which contains any _staking actions_ must not contain any other fields contributing to or withdrawing from the Chain Value Pool Balances _except_ Orchard actions, explicit transaction fees, and/or explicit ZEC burning fields.

### TODO

We need more explanation of transaction semantics:

- staking (including amount/time quantization) 
- unstaking (including amount/time quantization + yield/reward) 
- redelegation
- finalizer commission fees
- other?

## Changes to Ledger State

The _Ledger State_ is a conceptual and practical data structure which can be computed by fully verifying nodes solely from the sequence of blocks starting from the genesis block.[^lazy-ledger-state] To this Ledger State, Crosslink introduces the _roster_ which is a table containing all of the information about all ZEC staked to _finalizers_.

[^lazy-ledger-state]: Theoretically all of the Ledger State could be purely computed on demand from prior blocks, so literal Ledger State representations in memory can be thought of as chain indices or optimizing caches for verifying consensus rules. In some cases components of the Ledger State must be committed to within the blockchain state itself (for example, commitment anchors) which constrains how lazy a theoretical implementation can be. Real implementations trade off caching vs lazy-computation in a fine-grained way to achieve practical goals.

### The Roster

Crosslink's addition to Ledger State is embodied in the _PoS roster_, aka the _roster_ for short. Which tracks _staking positions_ created by Stakers, and attached to specific Finalizers. A Finalizer has a "voting weight" equal to the sum of all staking positions attached to it (see below in [FIXME](#FIXME)).

Each _staking position_ is composed of at least a single specific target _finalizer verification key_, a _staked ZEC amount_, and a _unstaking/redelegation verification key_. The _finalizer verification key_ designates a cryptographic signature verification key which serves as the sole on-chain reference to a specific _finalizer_[^multi-key-finalizers]. People may mean a specific person, organization, entity, computer server, etc... when they say "finalizer", but for the rest of this document we will use the term _finalizer_ as a shorthand for "whichever entity controls the verification key for a given _finalizer verification key_.

[^multi-key-finalizers]: Any particular entity may produce any number of _finalizer verifying keys_ of course, although for economic or social reasons, we expect many finalizers to be motivated to hang their reputation on 1 or just a few well-known finalizer verifying keys.

 The _unstaking/redelegation verifying key_ enables unstaking or redelegating that position from the participant that created that position (or more precisely anyone who controls the associated verifying key, which should be managed by a good wallet on behalf of users without their need to directly know or care about these keys for safe operation). These are unique to the position itself (and not linked on-chain to a user's other potential staking positions or other activity).

The sum of outstanding staking positions designating a specific finalizer verification key at a given block height is called the _stake weight_ of that finalizer. At a given block height the top `K` (currently 100) finalizers by stake weight are called the _active set_ of finalizers.

### Ledger State Invariants

We adopt a design invariant around the Ledger State changes Crosslink makes against mainnet Zcash:

> **General Ledger State Design Invariant:**
>
> All changes to Ledger State can be computed _entirely_ from (PoW) blocks, transactions, and data transitively referred to by such. This includes all existing Ledger State in mainnet Zcash today, the PoS and BFT roster state (see below), and the _finality status_ of blocks and transactions.

All current mainnet Zcash Ledger State and almost all Crosslink Ledger State is _height-immediate Ledger State_: the value for height $H$ can be computed from height $H$ itself (and implicitly all prior heights and contents). Some examples of height-immediate values are the total ZEC issued, the UTXOs spendable by the same P2PKH t-addr, the block's zero-knowledge commitment anchor values, and so on. In Crosslink examples include all of the roster state (e.g. the active set, finalizer verification keys, staking positions, ...).

#### Height-Eventual Ledger State

Crosslink introduces a novel category of Ledger State not present in Zcash mainnet, which is _height-eventual Ledger State_. This is a property or value about height $H$ which is only computable at a later larger height (of which the implicit block at $H$ must be an ancestor). The sole example of this is _finality status_.

The _finality status_ of a block at height $H$ (and by extension any transactions it contains and all previous block contents) is a property which guarantees[^guarantees] to the verifying node that this block (and all txns and previous content) is _final_ and cannot be reverted or rolled back. Furthermore, the presence of this property also provides a guarantee that a majority of finalizers already agree on the finality status of this block and also that *all verifying nodes* which see a sufficient number of subsequent blocks will also agree on the finality status.

[^guarantees] In this book, whenever we say a protocol "guarantees" some condition, we mean that *so long as the security assumptions truly hold* verifying nodes can and should safely rely on that condition.

The _finality status_ of a block at height $H$ (which we'll call a _finality-candidate block_ here for precision) is computable from a block at a later height $>H$ (which we'll call the _attesting block_). All verifying nodes who have observed the *that* attesting block will agree on the finality of the candidate block _even though_ they may observe the attesting block rollback. In the event of such a rollback, the protocol guarantees that an alternative block will eventually attest to the finality status of the same candidate block.

### Finality, Transaction Semantics, Ledger State, and the Roster

It bears pointing out a nuance: the Ledger State, including the Roster and the Active Set are all determined by valid transactions within PoW blocks. Every PoW block height thus has a specific unambiguous Ledger State. This comes into play later when we consider _finality_ which is a property of a given block height which is only verifiable at a later block height.

## The BFT Sub-Protocol

**TODO:** Flesh out this section



# TODOS from deep dive:

- rewards distribution to stakers and finalizers requires changes to the Coinbase transaction.
- how finality status is calculated
- how BFT operates, votes, network, etc...
- active set selection
- quantization of amounts / time
- the power of 10 restriction on amount is only applied to "stake" (create position)
- how do staking actions pay fees?
