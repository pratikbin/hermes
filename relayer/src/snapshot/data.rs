use core::fmt;
use std::collections::HashMap;

use bigdecimal::BigDecimal;
use serde::de::{Deserializer, Error as _};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::postgres::PgRow;
use sqlx::types::Json;

use ibc::core::ics03_connection::connection::IdentifiedConnectionEnd;
use ibc::core::ics04_channel::channel::IdentifiedChannelEnd;
use ibc::core::ics04_channel::packet::{Packet, Sequence};
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortChannelId, PortId};

use crate::chain::endpoint::ChainStatus;
use crate::client_state::IdentifiedAnyClientState;
use crate::consensus_state::AnyConsensusStateWithHeight;

use super::util::bigdecimal_to_u64;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IbcSnapshot {
    pub height: u64,
    pub data: IbcData,
}

impl<'r> sqlx::FromRow<'r, PgRow> for IbcSnapshot {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let height: BigDecimal = row.try_get("height")?;
        let data: Json<IbcData> = row.try_get("data")?;

        Ok(IbcSnapshot {
            height: bigdecimal_to_u64(height),
            data: data.0,
        })
    }
}

// TODO: Consider:
//
// - to help with reducing RPCs from update client
//   (update on NewBlock event, beefed up with block data, probably still the validators RPC is needed)
//
//   pub signed_header: SignedHeader,
//   pub validator_set: ValidatorSet,
//
// - update  clients, their state and consensus states on create and update client events
//
// - to help with packet acknowledgments...this is tricky as we need to pass from
//   the counterparty chain:
//     1. data (seqs for packets with commitments) on start
//     2. Acknowledge and Timeout packet events in order to clear
//
//   pub pending_ack_packets: HashMap<PacketId, Packet>,
//
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IbcData {
    pub app_status: ChainStatus,

    pub connections: HashMap<ConnectionId, IdentifiedConnectionEnd>,
    pub channels: HashMap<Key<PortChannelId>, IdentifiedChannelEnd>,

    pub client_states: HashMap<ClientId, IdentifiedAnyClientState>,
    pub consensus_states: HashMap<ClientId, Vec<AnyConsensusStateWithHeight>>,

    pub pending_sent_packets: HashMap<PacketId, Packet>, // TODO - use IbcEvent val (??)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PacketId {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: Sequence,
}

impl fmt::Display for PacketId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.port_id, self.channel_id, self.sequence)
    }
}

impl Serialize for PacketId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PacketId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = <&str>::deserialize(deserializer)?;
        let parts: [_; 3] = data
            .splitn(3, '/')
            .collect::<Vec<_>>()
            .as_slice()
            .try_into()
            .map_err(D::Error::custom)?;

        let [port_id, channel_id, sequence] = parts;

        let port_id: PortId = port_id.parse().map_err(D::Error::custom)?;
        let channel_id: ChannelId = channel_id.parse().map_err(D::Error::custom)?;
        let sequence: Sequence = sequence.parse().map_err(D::Error::custom)?;

        Ok(Self {
            port_id,
            channel_id,
            sequence,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key<A>(pub A);

impl Serialize for Key<PortChannelId> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{}:{}", self.0.channel_id, self.0.port_id))
    }
}

impl<'de> Deserialize<'de> for Key<PortChannelId> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = <&str>::deserialize(deserializer)?;
        let parts: [_; 2] = data
            .splitn(2, ':')
            .collect::<Vec<_>>()
            .as_slice()
            .try_into()
            .map_err(D::Error::custom)?;

        let [channel_id, port_id] = parts;

        let channel_id: ChannelId = channel_id.parse().map_err(D::Error::custom)?;
        let port_id: PortId = port_id.parse().map_err(D::Error::custom)?;

        Ok(Self(PortChannelId::new(channel_id, port_id)))
    }
}
