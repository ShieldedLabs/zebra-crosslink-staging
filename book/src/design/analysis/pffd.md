# Penalty for Failure to Defend

**TODO:** This is a paste of a Github ticket [Add "penalty-for-failure-to-defend" concept to book #264](https://github.com/ShieldedLabs/zebra-crosslink/issues/264) in order to ensure the text is committed into the revision.

Our primary conceptual framework for _quantitative_ comparison between Crosslink security properties versus pure PoW or Tendermint-like BFT protocols is called "penalty for failure to defend" (aka $PFD$).

## Key Top-Level Details

- PFDs can be specific to compromising specific properties; example: $PFD_{liveness}$ and $PFD_{finality}$ are different (so there's no simple top-line number for a given protocol).
- $PFD$ is distinct from the confusing misnomer of "cost-to-attack" (because the penalty may accrue to non-attackers, and the nature of successful attacks is that they cost less than designers or security defense analyzers fail to anticipate).
- Neither $PFD$ or the ill-conceived "cost-to-attack" metrics are conclusive about financial feasibility, since they don't capture attacker _revenue_/_profit_.
- Some $PFD$ may be a "financial value gauge" such as in Tendermint protocols where a $PFD$ could slash a large bond at stake. Meanwhile others may be a "rate" such as in PoW where the penalty to miners is loss of a future revenue rate over time.
- The two different types of $PFD$ (gauge vs rate) cannot be directly compared for two reasons:
  - The obvious reason that rates and gauges are different unit types.
  - The less obvious reason that the trade-offs in compromise, response, and recovery dynamics are qualitatively incommensurate. (TODO: flesh out the nuance here.) 

