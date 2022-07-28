use async_trait::async_trait;

use crate::std_prelude::*;
use crate::traits::ibc_event_context::IbcEventContext;
use crate::traits::packet_relayer::PacketRelayer;
use crate::traits::packet_relayers::ack_packet::AckPacketRelayer;
use crate::traits::packet_relayers::receive_packet::ReceivePacketRelayer;
use crate::traits::queries::status::{ChainStatus, ChainStatusQuerier};
use crate::traits::relay_context::RelayContext;
use crate::types::aliases::Packet;

pub struct FullRelayer<ReceiveRelay, AckRelay> {
    pub receive_relayer: ReceiveRelay,
    pub ack_relayer: AckRelay,
}

#[async_trait]
impl<Context, ReceiveRelay, AckRelay> PacketRelayer<Context> for FullRelayer<ReceiveRelay, AckRelay>
where
    Context: RelayContext,
    ReceiveRelay: ReceivePacketRelayer<Context>,
    AckRelay: AckPacketRelayer<Context>,
    Context::DstChain: IbcEventContext<Context::SrcChain>,
    Context::SrcChain: ChainStatusQuerier,
    Context::DstChain: ChainStatusQuerier,
{
    async fn relay_packet(
        &self,
        context: &Context,
        packet: &Packet<Context>,
    ) -> Result<(), Context::Error> {
        let source_height = context.source_chain().query_chain_status().await?.height();

        let write_ack = self
            .receive_relayer
            .relay_receive_packet(context, &source_height, packet)
            .await?;

        if let Some(ack) = write_ack {
            let destination_height = context
                .destination_chain()
                .query_chain_status()
                .await?
                .height();

            self.ack_relayer
                .relay_ack_packet(context, &destination_height, packet, &ack)
                .await?;
        }

        Ok(())
    }
}