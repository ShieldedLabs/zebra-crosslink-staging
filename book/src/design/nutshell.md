# Crosslink Design in a Nutshell

This chapter covers the major components of the design. This presents the design positively, without rationales, trade-offs, safety anlyses, or other supporting information. This will eventually be entirely redundant with, or simply link to, a canonical [Zcash Improvement Proposal](https://zips.z.cash) submission. Treat this chapter as a provisional sketch.

It assumes and leverages familiarity with Zcash PoW and only describes new differences from today's mainnet.

**CAVEATS:** A lot of the terminology and descriptions will be adjusted as we go to follow conventions and specifications in the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf) and the [Zcash Improvement Proposals](https://zips.z.cash).

## Changes to Transactions

A new transaction format is introduced which contains a new optional field for _staking actions_, which enable operations such as staking `ZEC` to a finalizer, beginning or completing an _unstaking_ action, or _redelegating_ an existing staking position to a different delegator.

The `ZEC` flowing into staking actions, or flowing out of completing unstaking transactions must balance as part of the transactions chain value pool balancing. See [The Zcash Protocol §4.17 Chain Value Pool Balances](https://zips.z.cash/protocol/protocol.pdf).

Additionally, we introduce a new restricting consensus rule on context-free transaction validity[^ctx-free-validity]:

[^ctx-free-validity]: A context-free transaction validity check may be performed on the bytes of the transaction itself without the need to access any chain state or index.

```
Crosslink-Staking-Orchard-Restriction:

  A transaction which contains any _staking actions_ must not contain any other fields contributing to the Chain Value Pool Balances _except_ Orchard actions, explicit transaction fees, and/or explicit ZEC burning fields.
```

## Changes to Ledger State

The _Ledger State_ is a conceptual and practical data structure which can be computed by fully verifying nodes solely from the sequence of blocks starting from the genesis block.[^lazy-ledger-state] To this Ledger State, Crosslink introduces the _roster_ which is a table containing all of the information about all ZEC staked to _finalizers_.

[^lazy-ledger-state]: Theoretically all of the Ledger State could be purely computed on demand from prior blocks, so literal Ledger State representations in memory can be thought of as chain indices or optimizing caches for verifying consensus rules. In some cases components of the Ledger State must be committed to within the blockchain state itself (for example, commitment anchors) which constrains how lazy a theoretical implementation can be. Real implementations trade off caching vs lazy-computation in a fine-grained way to achieve practical goals.

Each _staking position_ is composed of at least a single specific target _finalizer verification key_, a _staked ZEC amount_, and a _unstaking/redelegation verification key_. The _finalizer verification key_ designates a cryptographic signature verification key which serves as the sole on-chain reference to a specific _finalizer_[^multi-key-finalizers]. People may mean a specific person, organization, entity, computer server, etc... when they say "finalizer", but for the rest of this document we will use the term _finalizer_ as a shorthand for "whichever entity controls the verification key for a given _finalizer verification key_.

[^multi-key-finalizers]: Any particular entity may produce any number of _finalizer signing keys_ of course, although for economic or social reasons, we expect many finalizers to be motivated to hang their reputation on 1 or just a few well-known finalizer signing keys.

 The _unstaking/redelegation signing key_ enables unstaking or redelegating that position from the participant that created that position (or more precisely anyone who controls the associated signing key, which should be managed by a good wallet on behalf of users without their need to directly know or care about these keys for safe operation). These are unique to the position itself (and not linked on-chain to a user's other potential staking positions or other activity).

The sum of outstanding staking positions designating a specific finalizer verification key at a given block height is called the _stake weight_ of that finalizer. At a given block height the top `K` (currently 100) finalizers by stake weight are called the _active set_ of finalizers.

### Finality, Transaction Semantics, Ledger State, and the Roster

It bears pointing out a nuance: the Ledger State, including the Roster and the Active Set are all determined by valid transactions within PoW blocks. Every PoW block height thus has a specific unambiguous Ledger State. This comes into play later when we consider _finality_ which is a property of a given block height which is only verifiable at a later block height.

# MORE TO COME

...
