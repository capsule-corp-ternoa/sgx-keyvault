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

use my_node_primitives::NFTId;

/// Prints all registered keyvaults and stores all url within a file (one url per line)
pub fn check(_nft_id: NFTId, _owner: AccountId, _url: &str) -> Result<(), String> {
    // TODO: Task #6, create trusted call
    Ok(())
}
