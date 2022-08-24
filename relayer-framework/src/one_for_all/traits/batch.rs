use async_trait::async_trait;
use core::marker::PhantomData;

use crate::extras::batch::context::{BatchContext, HasBatchContext};
use crate::one_for_all::impls::message::OfaMessage;
use crate::one_for_all::traits::chain::OfaChain;
use crate::one_for_all::traits::error::OfaErrorContext;
use crate::one_for_all::traits::relay::{OfaRelay, OfaRelayContext};
use crate::std_prelude::*;
use crate::traits::core::Async;
use crate::traits::target::{DestinationTarget, SourceTarget};

#[derive(Clone)]
pub struct OfaBatchContext<Chain, Batch> {
    pub batch_context: Batch,
    pub phantom: PhantomData<Chain>,
}

impl<Chain, Batch> OfaBatchContext<Chain, Batch> {
    pub fn new(batch_context: Batch) -> Self {
        Self {
            batch_context,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
pub trait OfaBatch<Chain: OfaChain>: Async {
    type MessagesSender: Async;
    type MessagesReceiver: Async;

    type ResultSender: Async;
    type ResultReceiver: Async;

    fn new_messages_channel(&self) -> (Self::MessagesSender, Self::MessagesReceiver);

    fn new_result_channel(&self) -> (Self::ResultSender, Self::ResultReceiver);

    async fn send_messages(
        sender: &Self::MessagesSender,
        messages: Vec<Chain::Message>,
        result_sender: Self::ResultSender,
    ) -> Result<(), Chain::Error>;

    async fn try_receive_messages(
        receiver: &mut Self::MessagesReceiver,
    ) -> Result<Option<(Vec<Chain::Message>, Self::ResultSender)>, Chain::Error>;

    async fn receive_result(
        result_receiver: Self::ResultReceiver,
    ) -> Result<Result<Vec<Vec<Chain::Event>>, Chain::Error>, Chain::Error>;

    fn send_result(
        result_sender: Self::ResultSender,
        events: Result<Vec<Vec<Chain::Event>>, Chain::Error>,
    ) -> Result<(), Chain::Error>;
}

pub trait OfaChainWithBatch: OfaChain {
    type BatchContext: OfaBatch<Self>;

    fn batch_context(&self) -> &OfaBatchContext<Self, Self::BatchContext>;

    fn batch_sender(&self) -> &<Self::BatchContext as OfaBatch<Self>>::MessagesSender;
}

#[async_trait]
impl<Chain, Batch> BatchContext for OfaBatchContext<Chain, Batch>
where
    Chain: OfaChain,
    Batch: OfaBatch<Chain>,
{
    type Error = OfaErrorContext<Chain::Error>;

    type Message = OfaMessage<Chain>;

    type Event = Chain::Event;

    type MessagesSender = Batch::MessagesSender;

    type MessagesReceiver = Batch::MessagesReceiver;

    type ResultSender = Batch::ResultSender;

    type ResultReceiver = Batch::ResultReceiver;

    fn new_messages_channel(&self) -> (Self::MessagesSender, Self::MessagesReceiver) {
        self.batch_context.new_messages_channel()
    }

    fn new_result_channel(&self) -> (Self::ResultSender, Self::ResultReceiver) {
        self.batch_context.new_result_channel()
    }

    async fn send_messages(
        sender: &Self::MessagesSender,
        messages: Vec<Self::Message>,
        result_sender: Self::ResultSender,
    ) -> Result<(), Self::Error> {
        let in_messages = messages
            .into_iter()
            .map(|message| message.message)
            .collect();
        Batch::send_messages(sender, in_messages, result_sender)
            .await
            .map_err(OfaErrorContext::new)
    }

    async fn try_receive_messages(
        receiver: &mut Self::MessagesReceiver,
    ) -> Result<Option<(Vec<Self::Message>, Self::ResultSender)>, Self::Error> {
        let result = Batch::try_receive_messages(receiver)
            .await
            .map_err(OfaErrorContext::new)?;

        match result {
            Some((messages, result_sender)) => Ok(Some((
                messages.into_iter().map(OfaMessage::new).collect(),
                result_sender,
            ))),
            None => Ok(None),
        }
    }

    async fn receive_result(
        result_receiver: Self::ResultReceiver,
    ) -> Result<Result<Vec<Vec<Self::Event>>, Self::Error>, Self::Error> {
        let result = Batch::receive_result(result_receiver).await;

        match result {
            Ok(Ok(events)) => Ok(Ok(events)),
            Ok(Err(e)) => Ok(Err(OfaErrorContext::new(e))),
            Err(e) => Err(OfaErrorContext::new(e)),
        }
    }

    fn send_result(
        result_sender: Self::ResultSender,
        result: Result<Vec<Vec<Chain::Event>>, Self::Error>,
    ) -> Result<(), Self::Error> {
        let in_result = result.map_err(|e| e.error);

        Batch::send_result(result_sender, in_result).map_err(OfaErrorContext::new)
    }
}

impl<Relay> HasBatchContext<SourceTarget> for OfaRelayContext<Relay>
where
    Relay: OfaRelay,
    Relay::SrcChain: OfaChainWithBatch,
{
    type BatchContext =
        OfaBatchContext<Relay::SrcChain, <Relay::SrcChain as OfaChainWithBatch>::BatchContext>;

    fn batch_context(&self) -> &Self::BatchContext {
        self.relay.src_chain().chain.batch_context()
    }

    fn messages_sender(&self) -> &<Self::BatchContext as BatchContext>::MessagesSender {
        self.relay.src_chain().chain.batch_sender()
    }
}

impl<Relay> HasBatchContext<DestinationTarget> for OfaRelayContext<Relay>
where
    Relay: OfaRelay,
    Relay::DstChain: OfaChainWithBatch,
{
    type BatchContext =
        OfaBatchContext<Relay::DstChain, <Relay::DstChain as OfaChainWithBatch>::BatchContext>;

    fn batch_context(&self) -> &Self::BatchContext {
        self.relay.dst_chain().chain.batch_context()
    }

    fn messages_sender(&self) -> &<Self::BatchContext as BatchContext>::MessagesSender {
        self.relay.dst_chain().chain.batch_sender()
    }
}