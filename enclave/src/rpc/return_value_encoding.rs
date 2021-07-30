/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/

pub extern crate alloc;
use alloc::{str, vec::Vec};

use codec::Encode;

use substratee_worker_primitives::DirectRequestStatus;
use substratee_worker_primitives::RpcReturnValue;

pub fn compute_encoded_return_error(error_msg: &str) -> Vec<u8> {
    compute_encoded_return_value(error_msg, false, DirectRequestStatus::Error)
}

pub fn compute_encoded_return_value<T: Encode>(
    value: T,
    do_watch: bool,
    status: DirectRequestStatus,
) -> Vec<u8> {
    let return_value = RpcReturnValue {
        value: value.encode(),
        do_watch,
        status,
    };
    return_value.encode()
}
