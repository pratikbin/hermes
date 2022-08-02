use core::ops::Deref;

use dyn_clone::DynClone;
use erased_serde::Serialize as ErasedSerialize;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::tendermint::v1::Header as RawHeader;
#[cfg(any(test, feature = "mocks"))]
use ibc_proto::ibc::mock::Header as RawMockHeader;
use ibc_proto::protobuf::Protobuf as ErasedProtobuf;
use serde_derive::{Deserialize, Serialize};
use subtle_encoding::hex;

use crate::clients::ics07_tendermint::header::{decode_header, Header as TendermintHeader};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::Error;
#[cfg(any(test, feature = "mocks"))]
use crate::mock::header::MockHeader;
use crate::prelude::*;
use crate::timestamp::Timestamp;
use crate::Height;

use super::client_consensus::AsAny;

pub const TENDERMINT_HEADER_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.Header";
pub const MOCK_HEADER_TYPE_URL: &str = "/ibc.mock.Header";

/// Abstract of consensus state update information
pub trait Header:
    AsAny
    + DynClone
    + ErasedSerialize
    + ErasedProtobuf<Any, Error = Error>
    + core::fmt::Debug
    + Send
    + Sync
{
    /// The type of client (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// The height of the consensus state
    fn height(&self) -> Height;

    /// The timestamp of the consensus state
    fn timestamp(&self) -> Timestamp;
}

// Implements `Clone` for `Box<dyn Header>`
dyn_clone::clone_trait_object!(Header);

// Implements `serde::Serialize` for all types that have Header as supertrait
erased_serde::serialize_trait_object!(Header);

pub fn downcast_header<H: Header>(h: &dyn Header) -> Option<&H> {
    h.as_any().downcast_ref::<H>()
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum AnyHeader {
    Tendermint(TendermintHeader),

    #[cfg(any(test, feature = "mocks"))]
    Mock(MockHeader),
}

impl Header for AnyHeader {
    fn client_type(&self) -> ClientType {
        match self {
            Self::Tendermint(header) => header.client_type(),

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(header) => header.client_type(),
        }
    }

    fn height(&self) -> Height {
        match self {
            Self::Tendermint(header) => header.height(),

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(header) => header.height(),
        }
    }

    fn timestamp(&self) -> Timestamp {
        match self {
            Self::Tendermint(header) => header.timestamp(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(header) => header.timestamp(),
        }
    }
}

impl AnyHeader {
    pub fn encode_to_string(&self) -> String {
        let buf = ErasedProtobuf::encode_vec(self).expect("encoding shouldn't fail");
        let encoded = hex::encode(buf);
        String::from_utf8(encoded).expect("hex-encoded string should always be valid UTF-8")
    }

    pub fn decode_from_string(s: &str) -> Result<Self, Error> {
        let header_bytes = hex::decode(s).unwrap();
        ErasedProtobuf::decode(header_bytes.as_ref()).map_err(Error::invalid_raw_header)
    }
}

impl ErasedProtobuf<Any> for AnyHeader {}

impl TryFrom<Any> for AnyHeader {
    type Error = Error;

    fn try_from(raw: Any) -> Result<Self, Error> {
        match raw.type_url.as_str() {
            TENDERMINT_HEADER_TYPE_URL => {
                let val = decode_header(raw.value.deref())?;

                Ok(AnyHeader::Tendermint(val))
            }

            #[cfg(any(test, feature = "mocks"))]
            MOCK_HEADER_TYPE_URL => Ok(AnyHeader::Mock(
                ErasedProtobuf::<RawMockHeader>::decode_vec(&raw.value)
                    .map_err(Error::invalid_raw_header)?,
            )),

            _ => Err(Error::unknown_header_type(raw.type_url)),
        }
    }
}

impl From<AnyHeader> for Any {
    fn from(value: AnyHeader) -> Self {
        match value {
            AnyHeader::Tendermint(header) => Any {
                type_url: TENDERMINT_HEADER_TYPE_URL.to_string(),
                value: ErasedProtobuf::<RawHeader>::encode_vec(&header)
                    .expect("encoding to `Any` from `AnyHeader::Tendermint`"),
            },
            #[cfg(any(test, feature = "mocks"))]
            AnyHeader::Mock(header) => Any {
                type_url: MOCK_HEADER_TYPE_URL.to_string(),
                value: ErasedProtobuf::<RawMockHeader>::encode_vec(&header)
                    .expect("encoding to `Any` from `AnyHeader::Mock`"),
            },
        }
    }
}

#[cfg(any(test, feature = "mocks"))]
impl From<MockHeader> for AnyHeader {
    fn from(header: MockHeader) -> Self {
        Self::Mock(header)
    }
}

impl From<TendermintHeader> for AnyHeader {
    fn from(header: TendermintHeader) -> Self {
        Self::Tendermint(header)
    }
}
