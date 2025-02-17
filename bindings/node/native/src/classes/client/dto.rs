// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota::{
    AddressBalancePair, Ed25519Signature, IndexationPayload, Input, Output, OutputMetadata, Payload, ReferenceUnlock,
    SignatureLockedSingleOutput, SignatureUnlock, TransactionPayload, TransactionPayloadEssence, UTXOInput,
    UnlockBlock,
};
use serde::{Deserialize, Serialize};

use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct OutputDto {
    address: String,
    amount: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageTransactionPayloadEssenceDto {
    inputs: Box<[String]>,
    outputs: Box<[OutputDto]>,
    payload: Option<Box<MessagePayloadDto>>,
}

impl TryFrom<MessageTransactionPayloadEssenceDto> for TransactionPayloadEssence {
    type Error = crate::Error;
    fn try_from(value: MessageTransactionPayloadEssenceDto) -> crate::Result<Self> {
        let mut builder = TransactionPayloadEssence::builder();

        let inputs: Vec<Input> = value
            .inputs
            .into_vec()
            .into_iter()
            .map(|input| {
                UTXOInput::from_str(&input)
                    .unwrap_or_else(|_| panic!("invalid input: {}", input))
                    .into()
            })
            .collect();
        for input in inputs {
            builder = builder.add_input(input);
        }

        let outputs: Vec<Output> = value
            .outputs
            .into_vec()
            .into_iter()
            .map(|output| {
                SignatureLockedSingleOutput::new(
                    super::parse_address(output.address.clone())
                        .unwrap_or_else(|_| panic!("invalid output address: {}", output.address)),
                    output.amount,
                )
                .unwrap()
                .into()
            })
            .collect();
        for output in outputs {
            builder = builder.add_output(output);
        }

        builder = match value.payload {
            Some(indexation) => builder.with_payload(
                (*indexation)
                    .try_into()
                    .expect("Invalid indexation in TransactionPayloadEssenceJson"),
            ),
            _ => builder,
        };

        Ok(builder.finish()?)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageSignatureUnlockDto {
    #[serde(rename = "publicKey")]
    public_key: String,
    signature: String,
}

impl TryFrom<MessageSignatureUnlockDto> for SignatureUnlock {
    type Error = crate::Error;

    fn try_from(value: MessageSignatureUnlockDto) -> crate::Result<Self> {
        let mut public_key = [0u8; 32];
        hex::decode_to_slice(value.public_key, &mut public_key)?;
        let signature = hex::decode(value.signature)?.into_boxed_slice();
        Ok(Ed25519Signature::new(public_key, signature).into())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageUnlockBlockJsonDto {
    signature: Option<MessageSignatureUnlockDto>,
    reference: Option<u16>,
}

impl TryFrom<MessageUnlockBlockJsonDto> for UnlockBlock {
    type Error = crate::Error;

    fn try_from(value: MessageUnlockBlockJsonDto) -> crate::Result<Self> {
        let type_ = if value.signature.is_some() { 0 } else { 1 };
        match type_ {
            0 => {
                let sig: SignatureUnlock = value.signature.expect("Must contain signature.").try_into()?;
                Ok(sig.into())
            }
            1 => {
                let reference: ReferenceUnlock = value.reference.expect("Must contain reference.").try_into()?;
                Ok(reference.into())
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageTransactionPayloadDto {
    essence: MessageTransactionPayloadEssenceDto,
    #[serde(rename = "unlockBlocks")]
    unlock_blocks: Box<[MessageUnlockBlockJsonDto]>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageIndexationPayloadDto {
    index: String,
    data: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessagePayloadDto {
    /// The transaction payload.
    Transaction(MessageTransactionPayloadDto),
    /// The indexation payload.
    Indexation(MessageIndexationPayloadDto),
}

#[derive(Serialize, Deserialize)]
pub struct MessageDto {
    pub parent1: Option<String>,
    pub parent2: Option<String>,
    pub payload: MessagePayloadDto,
}

impl TryFrom<MessagePayloadDto> for Payload {
    type Error = crate::Error;
    fn try_from(payload: MessagePayloadDto) -> crate::Result<Self> {
        match payload {
            MessagePayloadDto::Transaction(transaction_payload) => {
                let mut transaction = TransactionPayload::builder();
                transaction = transaction.with_essence(transaction_payload.essence.try_into()?);

                let unlock_blocks = transaction_payload.unlock_blocks.into_vec();
                for unlock_block in unlock_blocks {
                    transaction = transaction.add_unlock_block(unlock_block.try_into()?);
                }

                Ok(Payload::Transaction(Box::new(transaction.finish()?)))
            }
            MessagePayloadDto::Indexation(indexation_payload) => {
                let indexation = IndexationPayload::new(indexation_payload.index, &indexation_payload.data).unwrap();
                Ok(Payload::Indexation(Box::new(indexation)))
            }
        }
    }
}

#[derive(Serialize)]
pub(super) struct OutputMetadataDto {
    /// Message ID of the output
    #[serde(rename = "messageId")]
    message_id: String,
    /// Transaction ID of the output
    #[serde(rename = "transactionId")]
    transaction_id: String,
    /// Output index.
    #[serde(rename = "outputIndex")]
    output_index: u16,
    /// Spend status of the output
    #[serde(rename = "isSpent")]
    is_spent: bool,
    /// Corresponding address
    address: String,
    /// Balance amount
    amount: u64,
}

impl From<OutputMetadata> for OutputMetadataDto {
    fn from(value: OutputMetadata) -> Self {
        Self {
            message_id: hex::encode(value.message_id),
            transaction_id: hex::encode(value.transaction_id),
            output_index: value.output_index,
            is_spent: value.is_spent,
            address: value.address.to_bech32(),
            amount: value.amount,
        }
    }
}

#[derive(Serialize)]
pub(super) struct AddressBalanceDto {
    address: String,
    balance: u64,
}

impl From<AddressBalancePair> for AddressBalanceDto {
    fn from(value: AddressBalancePair) -> Self {
        Self {
            address: value.address.to_string(),
            balance: value.balance,
        }
    }
}
