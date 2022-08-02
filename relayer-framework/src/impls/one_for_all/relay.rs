use async_trait::async_trait;

use crate::impls::one_for_all::chain::OfaChainContext;
use crate::impls::one_for_all::error::OfaErrorContext;
use crate::impls::one_for_all::message::OfaMessage;
use crate::impls::one_for_all::runtime::OfaRuntimeContext;
use crate::std_prelude::*;
use crate::traits::core::ErrorContext;
use crate::traits::messages::ack_packet::AckPacketMessageBuilder;
use crate::traits::messages::receive_packet::ReceivePacketMessageBuilder;
use crate::traits::messages::update_client::UpdateClientMessageBuilder;
use crate::traits::one_for_all::chain::OfaChain;
use crate::traits::one_for_all::relay::OfaRelay;
use crate::traits::packet::IbcPacket;
use crate::traits::relay_context::RelayContext;
use crate::traits::runtime::context::RuntimeContext;
use crate::traits::target::{DestinationTarget, SourceTarget};

pub struct OfaRelayContext<Relay: OfaRelay> {
    pub relay: Relay,

    pub src_chain: OfaChainContext<Relay::SrcChain>,
    pub dst_chain: OfaChainContext<Relay::DstChain>,

    pub src_client_id: <Relay::SrcChain as OfaChain>::ClientId,
    pub dst_client_id: <Relay::DstChain as OfaChain>::ClientId,

    pub runtime: OfaRuntimeContext<Relay::Runtime>,
}

pub struct OfaPacket<Relay: OfaRelay> {
    pub packet: Relay::Packet,
}

impl<Relay: OfaRelay> ErrorContext for OfaRelayContext<Relay> {
    type Error = OfaErrorContext<Relay::Error>;
}

impl<Relay: OfaRelay> RuntimeContext for OfaRelayContext<Relay> {
    type Runtime = OfaRuntimeContext<Relay::Runtime>;

    fn runtime(&self) -> &Self::Runtime {
        &self.runtime
    }
}

impl<Relay: OfaRelay> IbcPacket<OfaChainContext<Relay::SrcChain>, OfaChainContext<Relay::DstChain>>
    for OfaPacket<Relay>
{
    fn source_port(&self) -> &<Relay::SrcChain as OfaChain>::PortId {
        Relay::packet_src_port(&self.packet)
    }

    fn source_channel_id(&self) -> &<Relay::SrcChain as OfaChain>::ChannelId {
        Relay::packet_src_channel_id(&self.packet)
    }

    fn destination_port(&self) -> &<Relay::DstChain as OfaChain>::PortId {
        Relay::packet_dst_port(&self.packet)
    }

    fn destination_channel_id(&self) -> &<Relay::DstChain as OfaChain>::ChannelId {
        Relay::packet_dst_channel_id(&self.packet)
    }

    fn sequence(&self) -> &<Relay::SrcChain as OfaChain>::Sequence {
        Relay::packet_sequence(&self.packet)
    }

    fn timeout_height(&self) -> Option<&<Relay::DstChain as OfaChain>::Height> {
        Relay::packet_timeout_height(&self.packet)
    }

    fn timeout_timestamp(&self) -> &<Relay::DstChain as OfaChain>::Timestamp {
        Relay::packet_timeout_timestamp(&self.packet)
    }
}

impl<Relay: OfaRelay> RelayContext for OfaRelayContext<Relay> {
    type SrcChain = OfaChainContext<Relay::SrcChain>;

    type DstChain = OfaChainContext<Relay::DstChain>;

    type Packet = OfaPacket<Relay>;

    fn source_chain(&self) -> &Self::SrcChain {
        &self.src_chain
    }

    fn destination_chain(&self) -> &Self::DstChain {
        &self.dst_chain
    }

    fn source_client_id(&self) -> &<Relay::SrcChain as OfaChain>::ClientId {
        &self.src_client_id
    }

    fn destination_client_id(&self) -> &<Relay::DstChain as OfaChain>::ClientId {
        &self.dst_client_id
    }
}

pub struct OfaUpdateClientMessageBuilder;

#[async_trait]
impl<Relay: OfaRelay> UpdateClientMessageBuilder<OfaRelayContext<Relay>, SourceTarget>
    for OfaUpdateClientMessageBuilder
{
    async fn build_update_client_messages(
        context: &OfaRelayContext<Relay>,
        height: &<Relay::DstChain as OfaChain>::Height,
    ) -> Result<Vec<OfaMessage<Relay::SrcChain>>, OfaErrorContext<Relay::Error>> {
        let messages = context
            .relay
            .build_src_update_client_messages(height)
            .await?;

        let out_messages = messages.into_iter().map(OfaMessage::new).collect();

        Ok(out_messages)
    }
}

#[async_trait]
impl<Relay: OfaRelay> UpdateClientMessageBuilder<OfaRelayContext<Relay>, DestinationTarget>
    for OfaUpdateClientMessageBuilder
{
    async fn build_update_client_messages(
        context: &OfaRelayContext<Relay>,
        height: &<Relay::SrcChain as OfaChain>::Height,
    ) -> Result<Vec<OfaMessage<Relay::DstChain>>, OfaErrorContext<Relay::Error>> {
        let messages = context
            .relay
            .build_dst_update_client_messages(height)
            .await?;

        let out_messages = messages.into_iter().map(OfaMessage::new).collect();

        Ok(out_messages)
    }
}

#[async_trait]
impl<Relay: OfaRelay> ReceivePacketMessageBuilder<OfaRelayContext<Relay>>
    for OfaRelayContext<Relay>
{
    async fn build_receive_packet_message(
        &self,
        height: &<Relay::SrcChain as OfaChain>::Height,
        packet: &OfaPacket<Relay>,
    ) -> Result<OfaMessage<Relay::DstChain>, OfaErrorContext<Relay::Error>> {
        let message = self
            .relay
            .build_receive_packet_message(height, &packet.packet)
            .await?;

        Ok(OfaMessage::new(message))
    }
}

#[async_trait]
impl<Relay: OfaRelay> AckPacketMessageBuilder<OfaRelayContext<Relay>> for OfaRelayContext<Relay> {
    async fn build_ack_packet_message(
        &self,
        destination_height: &<Relay::DstChain as OfaChain>::Height,
        packet: &OfaPacket<Relay>,
        ack: &<Relay::DstChain as OfaChain>::WriteAcknowledgementEvent,
    ) -> Result<OfaMessage<Relay::SrcChain>, OfaErrorContext<Relay::Error>> {
        let message = self
            .relay
            .build_ack_packet_message(destination_height, &packet.packet, ack)
            .await?;

        Ok(OfaMessage::new(message))
    }
}