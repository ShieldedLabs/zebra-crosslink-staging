//! Types for the `z_getstandardfees` RPC.

/// A response to a `z_getstandardfees` RPC request.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ZGetStandardFeesResponse {
    pub standard_fee: u64,
    pub priority_fee: u64,
}
