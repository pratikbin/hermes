use ibc_proto::google::protobuf::Any;
use ibc_relayer::chain::cosmos::types::account;
use ibc_relayer::config;
use ibc_relayer::config::types;
use ibc_relayer::keyring;
use ibc_relayer_types::core::ics24_host::identifier;
use tendermint_rpc;

use crate::transaction::traits::field::FieldKey;

pub struct KeyEntry;

impl FieldKey for KeyEntry {
    type Value = keyring::KeyEntry;
}

pub struct AccountSequence;

impl FieldKey for AccountSequence {
    type Value = account::AccountSequence;
}

pub struct AccountNumber;

impl FieldKey for AccountNumber {
    type Value = account::AccountNumber;
}

pub struct AddressType;

impl FieldKey for AddressType {
    type Value = config::AddressType;
}

pub struct Memo;

impl FieldKey for Memo {
    type Value = types::Memo;
}

pub struct ExtensionOptions;

impl FieldKey for ExtensionOptions {
    type Value = Vec<Any>;
}

pub struct ChainId;

impl FieldKey for ChainId {
    type Value = identifier::ChainId;
}

pub struct RpcClient;

impl FieldKey for RpcClient {
    type Value = tendermint_rpc::HttpClient;
}

pub struct RpcAddress;

impl FieldKey for RpcAddress {
    type Value = tendermint_rpc::Url;
}