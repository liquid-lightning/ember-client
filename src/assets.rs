use crate::client::EmberClient;
use crate::error::Error;
use crate::pb::ember::assets::v1::{
    Asset, AssetInvoiceRequest, AssetInvoiceResponse, GetAssetBalancesRequest,
    AssetUtxo, AssetWalletTx, GetAssetBalancesResponse, ListAssetTransactionsRequest, GetSwapServiceRequest, ListAssetUtxosRequest, ListAssetsRequest, NewAssetAddressRequest,
    OpenAssetChannelRequest, OpenAssetChannelResponse, PayAssetInvoiceRequest,
    PayAssetInvoiceResponse, QuoteAssetInvoiceRequest, QuoteAssetInvoiceResponse,
    SendAssetRequest,
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

    /// This node's balance per asset: in its channels (ordinary L-BTC
    /// channels grouped under `"lbtc"`) and on-chain in its asset wallet,
    /// plus that wallet's L-BTC for fees (`asset_wallet_lbtc_msat`) -
    /// proxies to the caller's own node, unlike [`Self::list_assets`].
    pub async fn balances(&self) -> Result<GetAssetBalancesResponse, Error> {
        Ok(self
            .ember
            .assets_client()
            .get_asset_balances(GetAssetBalancesRequest {})
            .await?
            .into_inner())
    }

    /// Transaction history of this node's on-chain asset wallet: per tx
    /// the net change per asset - proxies to the caller's own node.
    pub async fn transactions(&self) -> Result<Vec<AssetWalletTx>, Error> {
        Ok(self
            .ember
            .assets_client()
            .list_asset_transactions(ListAssetTransactionsRequest {})
            .await?
            .into_inner()
            .transactions)
    }

    /// Unspent outputs of this node's on-chain asset wallet (its own
    /// elementsd wallet), unconfirmed ones included - proxies to the
    /// caller's own node, like [`Self::balances`].
    pub async fn utxos(&self) -> Result<Vec<AssetUtxo>, Error> {
        Ok(self
            .ember
            .assets_client()
            .list_asset_utxos(ListAssetUtxosRequest {})
            .await?
            .into_inner()
            .utxos)
    }

    /// A fresh address of this node's on-chain asset wallet (its own
    /// elementsd wallet). Receives any asset, and L-BTC sent here tops up
    /// the wallet's fee balance.
    pub async fn new_address(&self) -> Result<String, Error> {
        Ok(self
            .ember
            .assets_client()
            .new_asset_address(NewAssetAddressRequest {})
            .await?
            .into_inner()
            .address)
    }

    /// Sends `amount_msat` (1000 msat per base unit, whole units only) of
    /// `asset` from the asset wallet to `address`; fees are paid from the
    /// wallet's own L-BTC. Returns the txid.
    pub async fn send(
        &self,
        address: impl Into<String>,
        asset: impl Into<String>,
        amount_msat: u64,
    ) -> Result<String, Error> {
        Ok(self
            .ember
            .assets_client()
            .send_asset(SendAssetRequest {
                address: address.into(),
                asset: asset.into(),
                amount_msat,
            })
            .await?
            .into_inner()
            .txid)
    }

    /// Opens a channel in `asset` to `peer_id` (already connected), funded
    /// from the asset wallet, with `fee_reserve_sat` L-BTC beside it for
    /// its on-chain fees (`None`: the node's default).
    pub async fn open_channel(
        &self,
        peer_id: impl Into<String>,
        asset: impl Into<String>,
        amount_msat: u64,
        fee_reserve_sat: Option<u64>,
        announce: bool,
    ) -> Result<OpenAssetChannelResponse, Error> {
        Ok(self
            .ember
            .assets_client()
            .open_asset_channel(OpenAssetChannelRequest {
                peer_id: peer_id.into(),
                asset: asset.into(),
                amount_msat,
                fee_reserve_sat,
                announce,
            })
            .await?
            .into_inner())
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
