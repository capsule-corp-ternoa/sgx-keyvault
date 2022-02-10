use itp_storage::{storage_map_key, StorageHasher};
use itp_types::{AccountId, NFTData};
use sp_core::Pair;
use sp_runtime::MultiSignature;
use substrate_api_client::{Api, RpcClient, StorageKey};

use crate::ApiResult;

pub const NFTS: &str = "Nfts";

/// ApiClient extension that enables communication with the `nfts` pallet.
pub trait PalletNftsApi {
	fn data(&self, nft_id: u32) -> ApiResult<Option<NFTData>>;
	fn owner(&self, nft_id: u32) -> ApiResult<Option<AccountId>>;
	fn is_owner(&self, nft_id: u32, account: AccountId) -> ApiResult<Option<bool>>;
}

impl<P: Pair, Client: RpcClient> PalletNftsApi for Api<P, Client>
where
	MultiSignature: From<P::Signature>,
{
	fn data(&self, nft_id: u32) -> ApiResult<Option<NFTData>> {
		let key = storage_map_key(NFTS, "Data", &nft_id, &StorageHasher::Blake2_128Concat);
		self.get_storage_by_key_hash(StorageKey(key), None)
	}

	fn owner(&self, nft_id: u32) -> ApiResult<Option<AccountId>> {
		Ok(self.data(nft_id)?.map(|d| d.owner.into()))
	}

	fn is_owner(&self, nft_id: u32, account: AccountId) -> ApiResult<Option<bool>> {
		Ok(self.owner(nft_id)?.map(|o| o == account))
	}
}
