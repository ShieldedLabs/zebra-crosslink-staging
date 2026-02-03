//! Types for the `z_getstandardfees` RPC.

use derive_getters::Getters;
use derive_new::new;

/// A response to a `z_getstandardfees` RPC request.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, Getters, new)]
pub struct ZGetStandardFeesResponse {
    #[getter(copy)]
    pub(crate) standard_fee: u64,
    #[getter(copy)]
    pub(crate) priority_fee: u64,
}
