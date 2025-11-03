//! Types & commands for crosslink

use std::fmt;

use tokio::sync::broadcast;

use zebra_chain::block::{CommandBuf, Hash as BlockHash, Height as BlockHeight};

/// The finality status of a block
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum TFLBlockFinality {
    // TODO: rename?
    /// The block height is above the finalized height, so it's not yet determined
    /// whether or not it will be finalized.
    NotYetFinalized,

    /// The block is finalized: it's height is below the finalized height and
    /// it is in the best chain.
    Finalized,

    /// The block cannot be finalized: it's height is below the finalized height and
    /// it is not in the best chain.
    CantBeFinalized,
}

/// Placeholder representation for entity staking on PoS chain.
// TODO: do we want to unify or separate staker/finalizer/delegator
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct TFLStaker {
    /// Placeholder identity
    pub id: u64, // TODO: IP/malachite identifier/...
    /// Placeholder stake weight
    pub stake: u64, // TODO: do we want to store flat/tree delegators
                    // ALT: delegate_stake_to_id
                    // ...
}

/// Placeholder representation for group of stakers that are to be treated as finalizers.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct TFLRoster {
    /// The list of stakers whose votes(?) will count. Sorted by weight(?)
    pub finalizers: Vec<TFLStaker>,
}

/// Types of requests that can be made to the TFLService.
///
/// These map one to one to the variants of the same name in [`TFLServiceResponse`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TFLServiceRequest {
    /// Is the TFL service activated yet?
    IsTFLActivated,
    /// Get the final block hash
    FinalBlockHeightHash,
    /// Get a receiver for the final block hash
    FinalBlockRx,
    /// Set final block hash
    SetFinalBlockHash(BlockHash),
    /// Get the finality status of a block
    BlockFinalityStatus(BlockHeight, BlockHash),
    /// Get the finality status of a transaction
    TxFinalityStatus(zebra_chain::transaction::Hash),
    /// Get the finalizer roster
    Roster,
    /// Update the list of stakers
    UpdateStaker(TFLStaker),
    /// Get the fat pointer to the BFT chain tip
    FatPointerToBFTChainTip,
    /// Set the command buffer
    SetCommandBuf(CommandBuf),
}

/// Types of responses that can be returned by the TFLService.
///
/// These map one to one to the variants of the same name in [`TFLServiceRequest`].
#[derive(Debug)]
pub enum TFLServiceResponse {
    /// Is the TFL service activated yet?
    IsTFLActivated(bool),
    /// Final block hash
    FinalBlockHeightHash(Option<(BlockHeight, BlockHash)>),
    /// Receiver for the final block hash
    FinalBlockRx(broadcast::Receiver<(BlockHeight, BlockHash)>),
    /// Set final block hash
    SetFinalBlockHash(Option<BlockHeight>),
    /// Finality status of a block
    BlockFinalityStatus(Option<TFLBlockFinality>),
    /// Finality status of a transaction
    TxFinalityStatus(Option<TFLBlockFinality>),
    /// Finalizer roster
    Roster(TFLRoster),
    /// Update the list of stakers
    UpdateStaker, // TODO: batch?
    /// Fat pointer to the BFT chain tip
    FatPointerToBFTChainTip(zebra_chain::block::FatPointerToBftBlock),
    /// Set command buf
    SetCommandBuf,
}

/// Errors that can occur when interacting with the TFLService.
#[derive(Debug)]
pub enum TFLServiceError {
    /// Not implemented error
    NotImplemented,
    /// Arbitrary error
    Misc(String),
}

impl fmt::Display for TFLServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TFLServiceError: {:?}", self)
    }
}

use std::error::Error;
impl Error for TFLServiceError {}
