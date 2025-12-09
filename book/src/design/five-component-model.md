# The Five Component Model

There are five components of the system. Each component is responsible for preventing certain attacks, or for holding other components accountable.

## Wallets

The job of the wallet is to protect a user’s funds and to enable sending or receiving funds safely.

When receiving funds, the wallet verifies that the incoming funds total to a specific amount, safe against rollbacks, and available to be respent in the future. Additionally, wallets that prioritize user privacy often ensure all incoming funds become shielded to protect the user against unintentional on-chain privacy leaks.

When sending funds, the wallet minimizes all information leakage except those details a user explicitly requests to disclose to certain parties, or those details which must be disclosed. Zcash relies on strong encryption for all information where possible given a transaction type. Privacy is also a protection against censorship, since a censor must either identify specific transactions to censor, or perform blanket or probabilistic censorship in a less effective manner. However, some information disclosures or leakages are inevitable:

- Some information disclosures are always necessary for certain transactions:
- The amount, timing, and memo always needs to be disclosed to the recipient.
- The recipient’s receiving address is always known to a sender.
- Some other kinds of transactions require that some of the information is disclosed to specific parties or to the public in order to achieve certain kinds of desirable transparency. Examples include coin-voting, bridges/DEXes/L2’s, CEXes, turnstile migrations, t-addresses for backward compatibility, staking, unstaking, and redelegating.
- Some information “leakage” occurs even when not theoretically necessary for a given kind of transaction, especially by correlation with off-chain activity, such as:
- The recipient will often know something about the sender’s relationship with them, may be able to learn the sender’s IP address, etc. Therefore if the recipient colludes with the attacker, encryption cannot entirely protect against censorship by itself.
- 
- It is also the responsibility of the wallet to protect the user against censorship attacks that leverage network-level information leakage. Wallets can and should use network-level protections against information leakage such as Tor or Nym.

## Miners

The job of the miners is to include transactions and advance the blockchain, which provides an additional kind of censorship-resistance, in addition to the kind of censorship-resistance provided by wallets encrypting the transaction contents when possible. This function of miners also provides liveness. However the miners cannot unilaterally violate finality, even if an attacker controls >½ of the hashpower, because the finalizers ensure finality regardless of the behavior of the miners. A majority of the miners (>½ of the hashpower) can violate censorship-resistance (except when it can be provided by the wallet with encryption).

## Finalizers

The job of the finalisers is to ensure finality (i.e. to prevent rollback attacks) but finalisers cannot unilaterally compromise liveness or censorship-resistance (even if an attacker controls >⅔ of the stake-weighted votes) because miners ensure those properties. Also the finalisers cannot unilaterally execute rollback attacks even though they can prevent rollback attacks. A plurality of the finalisers (controlling >⅓ of the stake-weighted votes) could execute stall attacks (i.e. violate liveness).

## Stakers

It is the job of the stakers to hold the finalisers accountable by detecting and punishing misbehavior on the part of the finalizers by redelegating stake away from ill-behaved finalizers to well-behaved finalizers.

## Users

It is the duty of the users, collectively, to hold the stakers accountable for doing their job. If the stakers fail in their duty to hold the finalizers accountable, they can be disempowered and punished for this failure by the users with an emergency user-led hard fork (which is what happened in the STEEM/HIVE hard fork).

# Attacker Resource Scenarios

When considering these components, it is useful to consider scenarios where an attacker has compromised some portion of each kind of component, as a coarse means of assessing the robustness of Crosslink in maintaining the security properties.

* A collusion that controlled both >½ of the hashpower and >⅔ of the stake.
 + ...could violate finality.

* A collusion that included both a majority of the miners (controlling >½ of the hashpower) and a supermajority of the finalizers (controlling >⅔ of the stake-weighted votes)
 + ...could execute rollback and unavailability attacks.

Censorship-resistance is necessary for stakers to stake, unstake, and redelegate, which means censorship-resistance is necessary for stakers to be able to perform their duty of holding the finalizers accountable. Therefore, if an attacker controls >½ of the hashpower (and is thereby able to censor), then the attacker can prevent the stakers from holding the finalizers accountable.

An important and under-investigated consideration is what security properties hold when an attacker controls a minority of the hashpower. In a pure-PoW system, an attacker that controls some hashpower but ≤½ of it can not completely censor transactions, but can force them to be delayed.

* A collusion that controls >⅔ of the stake and <½ of the hashpower.
