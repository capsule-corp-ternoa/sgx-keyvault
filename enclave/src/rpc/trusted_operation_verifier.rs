// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

pub extern crate alloc;
use alloc::{string::String, string::ToString, vec::Vec};

use crate::attestation;
use crate::rpc::rpc_info::RpcCallStatus;
use crate::rsa3072;
use base58::ToBase58;
use codec::Decode;
use log::*;
use sgx_types::sgx_measurement_t;
use substratee_node_primitives::Request;
use substratee_stf::{Getter, TrustedCallSigned, TrustedOperation};
use substratee_worker_primitives::ShardIdentifier;

pub trait TrustedOperationExtractor: Send + Sync {
    fn decrypt_and_verify_trusted_operation(
        &self,
        request: Request,
    ) -> Result<TrustedOperation, String>;
}

pub struct TrustedOperationVerifier {}

impl TrustedOperationExtractor for TrustedOperationVerifier {
    fn decrypt_and_verify_trusted_operation(
        &self,
        request: Request,
    ) -> Result<TrustedOperation, String> {
        decrypt_and_verify_trusted_operation(request)
    }
}

pub fn decrypt_and_verify_trusted_operation(request: Request) -> Result<TrustedOperation, String> {
    // decrypt call
    let shard_id = request.shard;

    let trusted_operation = decrypt_cyphertext(request.cyphertext).map_err(|e| e.to_string())?;

    match verify_signature(&trusted_operation, &shard_id) {
        Ok(()) => {
            debug!("successfully verified signature")
        }
        Err(e) => return Err(e.to_string()),
    }

    Ok(trusted_operation)
}

pub fn decrypt_cyphertext(cyphertext: Vec<u8>) -> Result<TrustedOperation, RpcCallStatus> {
    debug!("decrypt Request -> TrustedOperation");
    // decrypt call
    let rsa_keypair = rsa3072::unseal_pair().unwrap();
    let encoded_operation: Vec<u8> = rsa3072::decrypt(&cyphertext.as_slice(), &rsa_keypair)
        .map_err(|_| RpcCallStatus::decryption_failure)?;
    // decode call
    TrustedOperation::decode(&mut encoded_operation.as_slice())
        .map_err(|_| RpcCallStatus::decoding_failure)
}

fn verify_signature(
    top: &TrustedOperation,
    shard_id: &ShardIdentifier,
) -> Result<(), RpcCallStatus> {
    debug!("verify signature of TrustedOperation");
    debug!("query mrenclave of self");
    let mrenclave = match attestation::get_mrenclave_of_self() {
        Ok(m) => m,
        Err(_) => return Err(RpcCallStatus::mrenclave_failure),
    };

    debug!("MRENCLAVE of self is {}", mrenclave.m.to_base58());

    match top {
        TrustedOperation::direct_call(tcs) => {
            verify_signature_of_signed_call(tcs, &mrenclave, shard_id)
        }
        TrustedOperation::indirect_call(tcs) => {
            verify_signature_of_signed_call(tcs, &mrenclave, shard_id)
        }
        TrustedOperation::get(getter) => {
            match getter {
                Getter::public(_) => Ok(()), // no need to verify signature on public getter
                Getter::trusted(tgs) => {
                    if let true = tgs.verify_signature() {
                        return Ok(());
                    }
                    Err(RpcCallStatus::signature_verification_failure)
                }
            }
        }
    }
}

fn verify_signature_of_signed_call(
    trusted_call: &TrustedCallSigned,
    mrenclave: &sgx_measurement_t,
    shard_id: &ShardIdentifier,
) -> Result<(), RpcCallStatus> {
    if trusted_call.verify_signature(&mrenclave.m, shard_id) {
        return Ok(());
    }

    Err(RpcCallStatus::signature_verification_failure)
}

pub mod tests {

    use super::*;
    use crate::rpc::mocks::dummy_builder::{
        create_dummy_account, create_dummy_request, sign_trusted_call,
    };
    use codec::Encode;
    use my_node_primitives::{nfts::NFTId, AccountId};
    use sp_core::{ed25519 as ed25519_core, Pair, H256};
    use substratee_stf::{ShamirShare, TrustedCall};

    pub fn given_valid_operation_in_request_then_decrypt_succeeds() {
        let nft_id: NFTId = 10;
        let share = vec![10, 20, 1, 0];
        let input_trusted_operation = create_trusted_operation(nft_id, share.clone());
        let request = Request {
            cyphertext: encrypt(input_trusted_operation),
            shard: H256::from([1u8; 32]),
        };

        let decrypted_operation = decrypt_cyphertext(request.cyphertext).unwrap();

        match decrypted_operation {
            TrustedOperation::direct_call(tcs) => match tcs.call {
                TrustedCall::keyvault_provision(_, retrieved_nft_id, retrieved_share) => {
                    assert_eq!(nft_id, retrieved_nft_id);
                    assert_eq!(share, retrieved_share);
                }
                _ => assert!(false, "got unexpected TrustedCall back from decoding"),
            },
            _ => assert!(false, "got unexpected TrustedOperation back from decoding"),
        }
    }

    pub fn given_nonsense_text_in_request_then_decode_fails() {
        let invalid_request = create_dummy_request();

        let top_result = decrypt_cyphertext(invalid_request.cyphertext);

        assert!(top_result.is_err());
    }

    pub fn given_valid_operation_with_invalid_signature_then_return_error() {
        let invalid_top = create_trusted_operation_with_incorrect_signature();
        let request = Request {
            cyphertext: encrypt(invalid_top),
            shard: H256::from([1u8; 32]),
        };

        let top_result = decrypt_and_verify_trusted_operation(request);

        assert!(top_result.is_err());

        match top_result {
            Ok(_) => assert!(false, "did not expect Ok result"),
            Err(e) => {
                assert_eq!(e, RpcCallStatus::signature_verification_failure.to_string())
            }
        }
    }

    fn create_trusted_operation(nft_id: NFTId, share: ShamirShare) -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();

        let trusted_call = TrustedCall::keyvault_provision(account_id, nft_id, share);
        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair);

        TrustedOperation::direct_call(trusted_call_signed)
    }

    fn create_trusted_operation_with_incorrect_signature() -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();

        let malicious_signer = ed25519_core::Pair::from_seed(b"19857777701234567890123456789012");

        let trusted_call = TrustedCall::keyvault_provision(account_id, 25, vec![]);

        let trusted_call_signed = sign_trusted_call(trusted_call, malicious_signer);

        TrustedOperation::direct_call(trusted_call_signed)
    }

    fn encrypt<E: Encode>(to_encrypt: E) -> Vec<u8> {
        let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
        let mut encrypted: Vec<u8> = Vec::new();
        rsa_pubkey
            .encrypt_buffer(&to_encrypt.encode(), &mut encrypted)
            .unwrap();
        encrypted
    }
}
