use anchor_lang::prelude::borsh;
use anchor_lang::prelude::borsh::maybestd::io;
use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::core::ics02_client::consensus_state::ConsensusState;
use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics23_commitment::commitment::CommitmentRoot;
use ibc::core::timestamp::Timestamp;
#[cfg(any(test, feature = "mocks"))]
use ibc::mock::consensus_state::{
    MockConsensusState, MOCK_CONSENSUS_STATE_TYPE_URL,
};
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::tendermint::v1::ConsensusState as RawTmConsensusState;
#[cfg(any(test, feature = "mocks"))]
use ibc_proto::ibc::mock::ConsensusState as RawMockConsensusState;
use ibc_proto::protobuf::Protobuf;

const TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.tendermint.v1.ConsensusState";

#[derive(Clone, Debug, PartialEq, derive_more::From, derive_more::TryInto)]
pub enum AnyConsensusState {
    Tendermint(TmConsensusState),
    #[cfg(any(test, feature = "mocks"))]
    Mock(MockConsensusState),
}

/// Discriminants used when borsh-encoding [`AnyConsensusState`].
#[derive(Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
enum AnyConsensusStateTag {
    Tendermint = 0,
    #[cfg(any(test, feature = "mocks"))]
    Mock = 255,
}

impl AnyConsensusStateTag {
    /// Returns tag from protobuf type URL.  Returns `None` if the type URL is
    /// not recognised.
    #[allow(dead_code)]
    fn from_type_url(url: &str) -> Option<Self> {
        match url {
            AnyConsensusState::TENDERMINT_TYPE => Some(Self::Tendermint),
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::MOCK_TYPE => Some(Self::Mock),
            _ => None,
        }
    }
}

impl AnyConsensusState {
    /// Protobuf type URL for Tendermint client state used in Any message.
    const TENDERMINT_TYPE: &'static str =
        ibc::clients::ics07_tendermint::consensus_state::TENDERMINT_CONSENSUS_STATE_TYPE_URL;
    #[cfg(any(test, feature = "mocks"))]
    /// Protobuf type URL for Mock client state used in Any message.
    const MOCK_TYPE: &'static str =
        ibc::mock::consensus_state::MOCK_CONSENSUS_STATE_TYPE_URL;

    /// Encodes the payload and returns discriminants that allow decoding the
    /// value later.
    ///
    /// Returns a `(tag, type, value)` triple where `tag` is discriminant
    /// identifying variant of the enum, `type` is protobuf type URL
    /// corresponding to the client state and `value` is the client state
    /// encoded as protobuf.
    ///
    /// `(tag, value)` is used when borsh-encoding and `(type, value)` is used
    /// in Any protobuf message.  To decode value [`Self::from_tagged`] can be
    /// used potentially going through [`AnyConsensusStateTag::from_type_url`] if
    /// necessary.
    fn to_any(&self) -> (AnyConsensusStateTag, &str, Vec<u8>) {
        match self {
            AnyConsensusState::Tendermint(state) => (
                AnyConsensusStateTag::Tendermint,
                Self::TENDERMINT_TYPE,
                Protobuf::<RawTmConsensusState>::encode_vec(state),
            ),
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::Mock(state) => (
                AnyConsensusStateTag::Mock,
                Self::MOCK_TYPE,
                Protobuf::<RawMockConsensusState>::encode_vec(state),
            ),
        }
    }

    /// Decodes protobuf corresponding to specified enum variant.
    fn from_tagged(
        tag: AnyConsensusStateTag,
        value: Vec<u8>,
    ) -> Result<Self, ibc_proto::protobuf::Error> {
        match tag {
            AnyConsensusStateTag::Tendermint => {
                Protobuf::<RawTmConsensusState>::decode_vec(&value)
                    .map(Self::Tendermint)
            }
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusStateTag::Mock => {
                Protobuf::<RawMockConsensusState>::decode_vec(&value)
                    .map(Self::Mock)
            }
        }
    }
}


impl Protobuf<Any> for AnyConsensusState {}

impl TryFrom<Any> for AnyConsensusState {
    type Error = ClientError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            TENDERMINT_CONSENSUS_STATE_TYPE_URL => {
                Ok(AnyConsensusState::Tendermint(
                    Protobuf::<RawTmConsensusState>::decode_vec(&value.value)
                        .map_err(|e| ClientError::ClientSpecific {
                        description: e.to_string(),
                    })?,
                ))
            }
            #[cfg(any(test, feature = "mocks"))]
            MOCK_CONSENSUS_STATE_TYPE_URL => Ok(AnyConsensusState::Mock(
                Protobuf::<RawMockConsensusState>::decode_vec(&value.value)
                    .map_err(|e| ClientError::ClientSpecific {
                        description: e.to_string(),
                    })?,
            )),
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: value.type_url.clone(),
            }),
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(value: AnyConsensusState) -> Self {
        let (_, type_url, value) = value.to_any();
        Any { type_url: type_url.into(), value }
    }
}

impl borsh::BorshSerialize for AnyConsensusState {
    fn serialize<W: io::Write>(&self, wr: &mut W) -> io::Result<()> {
        let (tag, _, value) = self.to_any();
        (tag as u8, value).serialize(wr)
    }
}

impl borsh::BorshDeserialize for AnyConsensusState {
    fn deserialize_reader<R: io::Read>(rd: &mut R) -> io::Result<Self> {
        let (tag, value) = <(u8, Vec<u8>)>::deserialize_reader(rd)?;
        let res = AnyConsensusStateTag::from_repr(tag)
            .map(|tag| Self::from_tagged(tag, value));
        match res {
            None => Err(format!("invalid AnyConsensusState tag: {tag}")),
            Some(Err(err)) => {
                Err(format!("unable to decode AnyConsensusState: {err}"))
            }
            Some(Ok(value)) => Ok(value),
        }
        .map_err(|msg| io::Error::new(io::ErrorKind::InvalidData, msg))
    }
}

impl ConsensusState for AnyConsensusState {
    fn root(&self) -> &CommitmentRoot {
        match self {
            AnyConsensusState::Tendermint(value) => value.root(),
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::Mock(value) => value.root(),
        }
    }

    fn timestamp(&self) -> Timestamp {
        match self {
            AnyConsensusState::Tendermint(value) => value.timestamp(),
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::Mock(value) => value.timestamp(),
        }
    }

    fn encode_vec(&self) -> Vec<u8> {
        match self {
            AnyConsensusState::Tendermint(value) => {
                ibc::core::ics02_client::consensus_state::ConsensusState::encode_vec(value)
            },
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::Mock(value) => {
                ibc::core::ics02_client::consensus_state::ConsensusState::encode_vec(value)
            }
        }
    }
}
