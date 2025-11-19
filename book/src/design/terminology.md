# Terminology

Here we introduce terminology specific to this design, especially when it may deviate from other consensus systems or common terminology used in the consensus protocol field.

## Crosslink Finality

Crosslink provides a kind of finality which we call _crosslink finality_. Crosslink finality has these properties:

- It is [accountable](#FIXME) because the chain contains explicit voting records which indicate each participating [finalizer](#FIXME)'s on-chain behavior.
- It is [irreversible](#FIXME) within the protocol scope. The only way to "undo" a final block is to modify the software to do so, which requires intervention by the user. Note that because a [finality safety violation](#FIXME) is always theoretically possible, this implies the only recovery from such a violation is out-of-band to the protocol and requires human intervention.
- It is [objectively verifiable](#FIXME) because it is possible to determine finality from a local block history without any out-of-band information.
- It is [globally consistent](#FIXME) meaning that all nodes which have received a sufficient number of valid blocks agree on the finality status, and also can rely on all other nodes in this set arriving at the same conclusion.
- It provides [asymmetric cost-of-attack defense](#FIXME) (TODO: Describe this better.)
