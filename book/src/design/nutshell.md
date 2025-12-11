# Crosslink Design in a Nutshell

This chapter covers the major components of the design. This presents the design positively, without rationales, trade-offs, safety anlyses, or other supporting information. This will eventually be entirely redundant with, or simply link to, a canonical [Zcash Improvement Proposal](https://zips.z.cash) submission. Treat this chapter as a provisional sketch.

It assumes and leverages familiarity with Zcash PoW and only describes new differences from today's mainnet.

**CAVEATS:** A lot of the terminology and descriptions will be adjusted as we go to follow conventions and specifications in the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf) and the [Zcash Improvement Proposals](https://zips.z.cash).

## Roles and Capabilities

We rely on several roles described in [the Five Component Model](five-component-model.md) in this nutshell section along with their intended capabilities. The absence of an explicit capability description for a role implies that role should not be able to perform that capability

## Changes to Transactions

### Changes to the Coinbase Transaction

The _coinbase transaction_ is an exceptional transaction Zcash inherited from Bitcoin which follows exceptional rules, including the distribution of newly issued tokens. This transaction is created by a block's miner.

The consensus rules are updated to require this transaction to make transfers of a portion of newly issued ZEC dedicated to Crosslink rewards _only when_ this block updates finality to a greater height, and then as the sum of all _pending finalization rewards_ since the previous finalization.

For each block produced which does not update finality, the per-block Crosslink rewards are collected into a running total, and a block which updates finality must then distributed this entire pending amount to the stakers and finalizers determined from the active set used to produce the finality update.

The _pending finalizaiton rewards_, $R$, in a block which updates finality must be distributed by the coinbase transaction to finalizers and stakers proportionally to their _reward slice_, $S_{participant}$, where the _active set_ (defined [below](#the-roster)) refers to the state as of the most recent final block, prior to this update.

This is calculated as follows where we let $W_{participant}$ be the total stake weight for that participant:

- For finalizers in the _active set_: $S_{finalizer} = COMMISSION_FEE_RATE * \frac{W_{finalizer}}{W_total}$

  The constant $COMMISSION_FEE_RATE = 0.1$ is a fixed protocol parameter called the _commission fee rate_.

- For all stakers: $S_{staker} = (1.0 - COMMISSION_FEE_RATE) \frac{W_{staker}}{W_total}$

  The constant $1.0 - COMMISSION_FEE_RATE = 0.9$ is called the _staker reward rate_.

  Note that $W_{staker}$ is the total weight of all staking positions of the given staker _regardless_ of whether or not it is assigned to an finalizer in the _active set_.

Another nuance: the sum of all commission fees and staker rewards adds up to $S_total$ *or less*, with the difference coming from stake assigned to finalizers outside the active set. 

### New Transaction Format & Staking Actions

A new transaction format is introduced which contains a new optional field for _staking actions_, which enable operations such as staking `ZEC` to a finalizer, beginning or completing an _unstaking_ action, or _redelegating_ an existing staking position to a different delegator.

The `ZEC` flowing into staking actions, or flowing out of completing unstaking transactions must balance as part of the transactions chain value pool balancing. See [The Zcash Protocol §4.17 Chain Value Pool Balances](https://zips.z.cash/protocol/protocol.pdf).

Additionally, we introduce a new restricting consensus rule on context-free transaction validity[^ctx-free-validity]:

[^ctx-free-validity]: A context-free transaction validity check may be performed on the bytes of the transaction itself without the need to access any chain state or index.

> **Crosslink Staking Orchard Restriction:**
>
> A transaction which contains any _staking actions_ must not contain any other fields contributing to or withdrawing from the Chain Value Pool Balances _except_ Orchard actions, explicit transaction fees, and/or explicit ZEC burning fields.

Abstractly, the staking actions include:

- _stake - This creates a new _staking position_ assigning a transparent ZEC amount, which must be a power of 10, to a finalizer and including a cryptographic signature verification key intended for only the signing-key holder of the position to redelegate or unstake.
- _redelegate_ - This identifies a specific staking position, a new finalizer to reassign to, and a signature valid with the position's published verification key to authorize the transition.
- _unbond_ - This initiates an "unbonding process" which puts the staking position into an _unbonding state_ and includes the ZEC amount, the current block height, and the signature verification key to authorize a later claim. Once in this state, redelegations are not valid and the ZEC amount does not contribute to staking weight for finalizers or the staker.
- _claim_ - This action removes a position in the _unbonding state_ and contributes the bonded ZEC into the transaction's Chain Value Pool Balance, where because of the "Crosslink Staking Orchard Restriction, it may only be distributed into an Orchard destination and/or transaction fees.

#### Further Restrictions on Staking Actions

We furthermore introduce the following consensus rule restrictions on transactions involving staking actions which rely on a _staking epoch_ design:

> **Staking Epoch Definition:**
>
> Once Crosslink activates at block height $H_{crosslink_activation}$, _staking epochs_ begin, which are continguous spans of block heights with two phases: _staking day_ and the _locked phase_. The number of blocks for _staking day_ is a protocol parameter constant roughly equivalent to 24 hours given assumptions about the Difficulty Adjustment Algorithm. The number of blocks for the _locked phase_ is also a constant roughly equivalent to 6*24 hour periods, so that _staking day_ is approximately one day per week.

Given the staking epoch definition, we introduce the following restrictions in the consensus rules:

> **Locked Staking Actions Restriction:**
>
> If a transaction includes any staking actions *except for* redelegate, and that transaction is in a block height in a _locked phase_ of the _staking epoch_, then that block is invalid.

Also:

> **Unbonding Delay:**
>
> If a transaction is block height $H$ includes any _claim_ staking actions which refer to _unbonding state_ positions which have not existed in the unbonding state for at least one full staking epoch, that block is invalid.

Note: An implication of the Unbonding Delay is that the same staking day cannot include both an _unbond_ and _claim_ for the same position.

A final restriction (already mentioned above in introducing the _staking_ action is:

> **Stake Action Amount Quantization:**
>
> If a transaction includes any staking actions with an amount which is not a power of 10 ZEC (or ZAT), that transaction is invalid.

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
