Scope for Crosslink
===

# What is Shielded Labs's first deployment of Crosslink actually required to do?

# Rationale

## Why add Proof-of-Stake?

* Staking ZEC replaces sell pressure from mining pools with buy pressure from finalizers. Reducing supply and increasing demand can potentially put upward pressure on the price of ZEC. This is good because the price of ZEC is the fuel for the mission and attracts users.
* Proof-of-Stake adds assured finality, which protects users from being robbed, improves the safety and efficiency of bridges, and enables centralized exchanges and other services to reduce and unify [deposit times](https://zechub.wiki/using-zcash/custodial-exchanges).
* Proof-of-Stake provides finality more efficiently than Proof-of-Work – at a lower cost in terms of both economics and energy usage. This makes Zcash both more secure (provide better security for the same cost) and more sustainable (provide security long-term even if the rate of issuance shrinks).
* Staking allows a larger number of users to participate actively in the network than mining does, and to become direct recipients of newly created ZEC from the blockchain. This increases the size and decentralization of the network of users and stakeholders.
* About the economic disadvantage of a smaller miner/finalizer/delegator competing with a larger operation:
  * In Proof-of-Work, smaller miners have a substantial economic disadvantage compared to larger mining operations.
  * In Proof-of-Stake, smaller finalizers have an economic disadvantage compared to larger finalizer services, but the disadvantage is not as pronounced as in Proof-of-Work mining.
  * In Proof-of-Stake, smaller delegators compete on a level playing field with larger delegators, earning roughly the same reward with roughly the same risk. (Note that delegators get less reward than finalizers.)

## Why use a Hybrid Proof-of-Stake plus Proof-of-Work system?

* Proof-of-Work provides a different and complementary kind of security that Proof-of-Stake doesn’t: preventing attackers from re-using resources in attacks. Hybrid finality--which leverages both kinds of security--provides a stronger kind of protection for users than pure-Proof-of-Stake finality, and thus lays a secure foundation for future scalability improvements.
* Proof-of-Work allows people to earn ZEC by mining, even if they don’t already own any ZEC and they can’t buy ZEC on an exchange.
* Proof-of-Work facilitates a ZEC->fiat->ZEC economy – rewards from mining have to be mostly spent on purchasing real-world goods and services like electricity and computing hardware (ASICs), which means transaction flows between ZEC and fiat. In contrast, rewards from staking can be reinvested directly into staking, which doesn’t require converting to fiat (although on the other hand it is good because it has those positive effects on supply and demand described above).
* Keeping Proof-of-Work in addition to adding Proof-of-Stake means that in addition to all of the stakers, we also keep miners as active participants in the network and as recipients of ZEC, increasing the total size and diversity of the Zcash network and the ZEC economy.


UX goals
---

This list of [UX Goals](https://github.com/ShieldedLabs/zebra-crosslink/labels/UX%20Goal) is tracked on GitHub.

* [GH #131](https://github.com/ShieldedLabs/zebra-crosslink/issues/131): CEXes are willing to rely on incoming Zcash transaction finality. E.g. Coinbase re-enables market orders and reduces required confirmations to a nice small number like 10. All or most services rely on the same canonical (protocol-provided) finality instead of enforcing [their own additional delays or conditions](https://zechub.wiki/using-zcash/custodial-exchanges), and so the user experience is that transaction finality is predictable and recognizable across services.
* [GH #128](https://github.com/ShieldedLabs/zebra-crosslink/issues/128): Other services that require finality, such as cross-chain bridges, are willing to rely on Zcash’s finality.
* [GH #127](https://github.com/ShieldedLabs/zebra-crosslink/issues/127): Casual users (who understand little about crypto and do not use specialized tools such as a Linux user interface) delegate ZEC and get rewards from their mobile wallet. [GH #124](https://github.com/ShieldedLabs/zebra-crosslink/issues/124) They have to learn a minimal set of new concepts in order to do this.
* [GH #126](https://github.com/ShieldedLabs/zebra-crosslink/issues/126): Users (who have to understand a lot and can use specialized tools) run finalizers and get rewards.
* [GH #158](https://github.com/ShieldedLabs/zebra-crosslink/issues/158): Users can get compounding returns by leaving their stake plus their rewards staked, without them or their wallet having to take action.


_Shielded Labs’s First Deployment of Crosslink is not done until substantial numbers of real users are actually gaining these five benefits._

Deployment Goals
---

This list of [Deployment Goals](https://github.com/ShieldedLabs/zebra-crosslink/labels/Deployment%20Goals) is tracked on GitHub.

* [GH #125](https://github.com/ShieldedLabs/zebra-crosslink/issues/125): Zcash transactions come with a kind of finality which protects the users as much as possible against all possible attacks, and is sufficient for services such as cross-chain bridges and centralized exchanges.
* [GH #124](https://github.com/ShieldedLabs/zebra-crosslink/issues/124): Users can delegate their ZEC and earn rewards, safely and while needing to learn only the minimal number of new concepts.
    * Delegating to a finalizer does not enable the finalizer to steal your funds.
    * Delegating to a finalizer does not leak information that links the user's action to other information about them, such as their IP address, their other ZEC holdings that they are choosing not to stake, or their previous or future transactions.
* [GH #123](https://github.com/ShieldedLabs/zebra-crosslink/issues/123): The time-to-market and the risk of Shielded Labs's First Deployment of Crosslink is minimized: the benefits listed above start accruing to users as soon as safely possible.
* [GH #122](https://github.com/ShieldedLabs/zebra-crosslink/issues/122): Activating Crosslink on Zcash mainnet retains as much as possible of Zcash users' safety, security, privacy, and availability guarantees.

Trade-offs
---

Goals which we currently believe are not safely achievable in this first deployment without losing some of the above goals:
* Improving Zcash's bandwidth (number of transactions per time) or latency (time for a transaction).
* Deploying cross-chain interoperation such as the Inter-Blockchain Communication protocol (IBC) or Cross-Chain Interoperability Protocol (CCIP).
* Supporting a large number of finalizer so that any user who wants to run their own finalizer can do so.
* Supporting casual users, with minimal computer expertise, running finalizers.

Non-Goals
---

* The consensus mechanism is needed only to prevent double-spending/multi-spending/rollback attacks. It is not needed for censorship-resistance, since that is provided by end-to-end encryption, and it is not needed for counterfeiting-resistance, since that is provided by two layers of defense: proofs and turnstiles. So censorship-resistance and counterfeiting-resistance are non-goals for (at least) the first deployment of Crosslink.
* This is neutral with regard to Zcash governance -- it doesn't change anything about Zcash governance.
* This does not change the emissions schedule -- how much total ZEC is in circulation at any given point in time in the future -- and in particular it does not change the eventual 21 million ZEC supply cap.

See also [Zebra Crosslink Implementation Requirements](https://docs.google.com/document/d/1YXalTGoezGH8GS1dknO8aK6eBFRq_Pq8LvDeho1KVZ8/edit?usp=sharing).
