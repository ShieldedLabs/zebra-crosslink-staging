# Proposal: Mechanism Design Audit of Crosslink Zebra

Prepared for: Shielded Labs
Duration: 1 week core analysis + light follow‑up outreach

## Objective

Shielded Labs is designing and stewarding Crosslink v2 (Crosslink 2 consensus plus updated staking economics). This proposal covers a mechanism design–focused audit that:

- Evaluates the incentive structure faced by miners, finalizers, stakers, and the wider community.
- Assesses the Penalty for Failure to Defend (PFD) for key properties (safety, liveness, censorship resistance).
- Identifies likely equilibria and failure modes under realistic behavior.
- Produces clear recommendations to harden Crosslink v2 before or shortly after deployment.
- The work will be completed over one week, followed by targeted outreach so results can inform the broader Zcash ecosystem.

## Scope

In scope: Crosslink Zebra as defined by Shielded Labs, specifically defined in this book in the [Design](../../../../design.md) chapter.

## Questions to answer:

- What behaviors are rational for miners, finalizers, and stakers?
- What are the “good” and “bad” equilibria?
- How large is the effective PFD for safety, liveness, and censorship resistance?
- Where does the design rely on social courage instead of hard incentives?
- In general, what conditions are necessary to violate the security properties listed [Security Properties](../../../security-properties.md)

### Out of scope

- Cryptographic soundness of Zcash primitives.
- Implementation bugs in node software.
- Detailed macro‑economic modeling of ZEC price or off‑chain markets.
- Privacy impacts on stakers or finalizers. Privacy protection is a top priority for Zcash generally, and a priority for Crosslink analyses in the future. However, for this initial audit we first want to focus on the general safety and reliability of the base consensus mechanism. Note that there are a set of constraints on the mechanism design which are present in order to provide privacy, and these must be satisfied for any alternatives or recommendations. These are described in our blog post Crosslink: Updated Staking Design.

## Deliverables

By the end of the one‑week analysis:

1. Mechanism Design Audit Report (10–15 pages)
  - Role‑by‑role incentive analysis (miners, finalizers, stakers, community).
  - PFD analysis for core properties under representative attack scenarios.
  - Identified “bad equilibria” (e.g., PoS cartelization).
  - Prioritized recommendations with concrete levers (e.g., minimal penalties, tweaks to incentives to bolster against bad outcomes, delegation tweaks, governance formalization).
2. Executive Summary (2–3 pages)
  - Non‑technical overview of risks, trade‑offs, and recommended changes, suitable for Shielded Labs leadership and partner orgs (ECC, ZF).
3. Slide Deck (15–20 slides)
  - Visual summary of findings for internal review and community calls.
4. Outreach Draft
  - Draft blog / forum post summarizing the results for the public Zcash community.
  - Suggested agenda + talking points for a community call presenting the audit.

## Work Plan & Timeline

**Day 1 – Kickoff & Alignment**

- 60–90 minute kickoff with Shielded Labs.
- Confirm canonical specs and design assumptions.
- Clarify constraints (e.g., “no protocol‑level slashing in V1”) and any specific questions.
- Finalize success criteria and deliverable format.

**Days 2–4 – Mechanism Design Analysis**

- Map incentives for each role (miners, finalizers, stakers, community), given:
  - 40/40 issuance split;
  - 5‑day epochs, 5–10 day unbonding;
  - Emergency patch + hard‑fork constitution.
- Identify likely equilibria and failure modes (cartels, PoS capture, unbounded divergence between PoW progress and a stalled BFT subprotocol, etc.).
- Qualitative PFD modeling for safety, liveness, and censorship resistance; comparison to Zcash PoW‑only at the same issuance rate.

**Day 5 – Recommendations & Drafts**

- Translate analysis into concrete, ranked recommendations (high / medium / low impact & complexity).
- Produce draft audit report and executive summary.

**Day 6 – Review & Finalization**

- Share drafts with a small Shielded Labs review group.
- Incorporate feedback and finalize:
  - Audit report
  - Executive summary
  - Slide deck

**Day 7 – Outreach Prep**

- Draft blog / forum post and call agenda.
- Finalize outreach materials in collaboration with Shielded Labs communications.

## Outreach & Follow‑up

Within the same budget:

- **One live presentation** of findings (e.g., Zcash community call or internal Shielded Labs session).
- **Asynchronous Q&A** on the report (e.g., GitHub, forum thread) for 1–2 weeks after publication.
- Minor updates to slides and outreach text in response to initial feedback (no new analysis).
