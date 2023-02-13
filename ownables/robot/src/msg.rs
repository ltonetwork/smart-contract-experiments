use std::collections::HashMap;
use cosmwasm_std::{Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use crate::state::NFT;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub ownable_id: String,
    // pub nft: NFT,
    // pub network_id: u8, // T/L in ascii: 76/84
    // pub image: Option<String>,
    // pub image_data: Option<String>,
    // pub external_url: Option<String>,
    // pub description: Option<String>,
    // pub name: Option<String>,
    // pub background_color: Option<String>,
    // pub animation_url: Option<String>,
    // pub youtube_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // transfers ownership
    Transfer { to: Addr },
    // locks the ownable
    Lock {},
}



#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ExternalEvent {
    // CAIP-2 format: <namespace + ":" + reference>
    // e.g. ethereum: eip155:1
    pub chain_id: String,
    pub event_type: String,
    pub args: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetOwnableConfig {},
    GetOwnableMetadata {},
    GetOwnership {},
    IsLocked {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnableStateResponse {
    pub consumed_ownable_ids: Vec<Addr>,
    pub color: String,
    pub has_antenna: bool,
    pub has_speaker: bool,
    pub has_armor: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnershipResponse {
    pub owner: String,
    pub issuer: String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct IdbStateDump {
    // map of the indexed db key value pairs of the state object store
    #[serde_as(as = "Vec<(_, _)>")]
    pub state_dump: HashMap<Vec<u8>, Vec<u8>>,
}

// from github.com/CosmWasm/cw-nfts/blob/main/contracts/cw721-metadata-onchain
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    // pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}