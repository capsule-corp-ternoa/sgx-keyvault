extern crate alloc;

use crate::DirectRequestStatus;
use alloc::{borrow::ToOwned, string::String, vec::Vec};
use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode)]
pub struct RpcReturnValue {
	pub value: Vec<u8>,
	pub do_watch: bool,
	pub status: DirectRequestStatus,
	//pub signature: Signature,
}
impl RpcReturnValue {
	pub fn new(val: Vec<u8>, watch: bool, status: DirectRequestStatus) -> Self {
		Self {
			value: val,
			do_watch: watch,
			status,
			//signature: sign,
		}
	}

	pub fn from_error_message(error_msg: &str) -> Self {
		RpcReturnValue {
			value: error_msg.encode(),
			do_watch: false,
			status: DirectRequestStatus::Error,
		}
	}
}

#[derive(Clone, Encode, Decode, Serialize, Deserialize, Debug)]
pub struct RpcError {
	pub code: i64,
	pub message: Option<String>,
}

#[derive(Clone, Encode, Decode, Serialize, Deserialize, Debug)]
pub struct RpcResponse<T>
where
	T: Serialize,
{
	pub jsonrpc: String,
	pub result: T, // encoded RpcReturnValue
	pub id: u32,
	pub error: Option<RpcError>,
}

#[derive(Clone, Encode, Decode, Serialize, Deserialize)]
pub struct RpcRequest<T>
where
	T: Serialize,
{
	pub jsonrpc: String,
	pub method: String,
	pub params: T,
	pub id: i32,
}

impl<T: Serialize> RpcRequest<T> {
	pub fn compose_jsonrpc_call(method: String, data: T) -> String {
		let direct_invocation_call =
			RpcRequest { jsonrpc: "2.0".to_owned(), method, params: data, id: 1 };
		serde_json::to_string(&direct_invocation_call).unwrap()
	}
}
