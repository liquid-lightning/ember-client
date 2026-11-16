use crate::client::EmberClient;
use crate::error::Error;
use crate::pb::ember::assets::v1::{
    Asset, AssetBalance, AssetInvoiceRequest, AssetInvoiceResponse, GetAssetBalancesRequest,
    GetSwapServiceRequest, ListAssetsRequest, PayAssetInvoiceRequest, PayAssetInvoiceResponse,
    QuoteAssetInvoiceRequest, QuoteAssetInvoiceResponse,
};

/// Ergonomic wrapper over ember's own Assets service
/// (`ember.assets.v1.Assets`) - cross-asset payments via the swap service,
/// not a CLN proxy (that's [`crate::node::ClnClient`]). Cheap to clone -
/// it's just a handle on the shared connection, same as [`crate::Scheduler`].
#[derive(Clone)]
pub struct Assets {
    ember: EmberClient,
}

impl Assets {
    pub fn new(ember: EmberClient) -> Self {
        Self { ember }
    }

    /// The assets ember currently supports cross-asset payments for -
    /// server-side config (`SUPPORTED_ASSETS`), not per-node state.
    pub async fn list_assets(&self) -> Result<Vec<Asset>, Error> {
        Ok(self
            .ember
            .assets_client()
            .list_assets(ListAssetsRequest {})
            .await?
            .into_inner()
            .assets)
    }

    /// Creates an invoice for `amount_msat` requiring payment in `asset`
    /// (the opaque asset id `list_assets` returned, or "lbtc" for this
    /// node's own native coin), wrapped into one self-contained string -
    /// share/scan this, not `bolt11` alone, since BOLT11 has no asset
    /// field at all.
    pub async fn create_invoice(
        &self,
        asset: impl Into<String>,
        amount_msat: u64,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<AssetInvoiceResponse, Error> {
        Ok(self
            .ember
            .assets_client()
            .asset_invoice(AssetInvoiceRequest {
                asset: asset.into(),
                amount_msat,
                label: label.into(),
                description: description.into(),
            })
            .await?
            .into_inner())
    }

    /// Estimates what paying `wrapped` (a string from [`Self::create_invoice`]
    /// or a peer) would cost, without paying it - only returns real numbers
    /// if this node happens to carry matching pricing config of its own
    /// (see the underlying plugin's own doc comment); otherwise fails the
    /// same way an unsupported pair would.
    pub async fn quote_invoice(
        &self,
        wrapped: impl Into<String>,
    ) -> Result<QuoteAssetInvoiceResponse, Error> {
        Ok(self
            .ember
            .assets_client()
            .quote_asset_invoice(QuoteAssetInvoiceRequest {
                wrapped: wrapped.into(),
            })
            .await?
            .into_inner())
    }

    /// Pays `wrapped`, even in an asset this node doesn't directly hold
    /// (via a swap through the configured provider). `tier` picks a
    /// pricing tier (default: "medium"); `from_asset`/`from_amount_msat`
    /// let the caller choose which asset to pay with and override the
    /// local price estimate - see the underlying plugin's own
    /// `payassetinvoice` doc comment for the full semantics of each.
    pub async fn pay_invoice(
        &self,
        wrapped: impl Into<String>,
        tier: Option<String>,
        from_asset: Option<String>,
        from_amount_msat: Option<u64>,
    ) -> Result<PayAssetInvoiceResponse, Error> {
        Ok(self
            .ember
            .assets_client()
            .pay_asset_invoice(PayAssetInvoiceRequest {
                wrapped: wrapped.into(),
                tier,
                from_asset,
                from_amount_msat,
            })
            .await?
            .into_inner())
    }

    /// This node's own channel balance per asset (ordinary L-BTC channels
    /// grouped under `"lbtc"`) - proxies to the caller's own node, unlike
    /// [`Self::list_assets`].
    pub async fn balances(&self) -> Result<Vec<AssetBalance>, Error> {
        Ok(self
            .ember
            .assets_client()
            .get_asset_balances(GetAssetBalancesRequest {})
            .await?
            .into_inner()
            .balances)
    }

    /// The shared swap-service node every freshly-provisioned node gets
    /// peer-connected to by default (no channel) - `None` if `ember-cli
    /// set-swap-service` hasn't configured one. Server-side config, like
    /// [`Self::list_assets`], not a per-node call.
    pub async fn swap_service(&self) -> Result<Option<(String, String)>, Error> {
        let resp = self
            .ember
            .assets_client()
            .get_swap_service(GetSwapServiceRequest {})
            .await?
            .into_inner();
        Ok(resp.node_id.zip(resp.addr))
    }
}
