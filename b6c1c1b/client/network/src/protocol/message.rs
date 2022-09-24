// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Network packet message types. These get serialized and put into the lower level protocol
//! payload.

pub use self::generic::{
	AnnounceMessage, BlockAnnounce, AdjustAnnounce, FromBlock, RemoteCallRequest, RemoteChangesRequest, RemoteChangesResponse,
	RemoteHeaderRequest, RemoteHeaderResponse, RemoteReadChildRequest, RemoteReadRequest, Roles,
};
use bitflags::bitflags;
use codec::{Encode, Error, Input, Output};
pub use codec::Decode;
use sc_client_api::StorageProof;
use sp_runtime::{
	traits::{Block as BlockT, Header as HeaderT},
	ConsensusEngineId,
};
pub use sp_arithmetic::traits::{ AtLeast32Bit, AtLeast32BitUnsigned};
/// A unique ID of a request.
pub type RequestId = u64;

/// Type alias for using the message type using block type parameters.
pub type Message<B> = generic::Message<
	<B as BlockT>::Header,
	<B as BlockT>::Hash,
	<<B as BlockT>::Header as HeaderT>::Number,
	<B as BlockT>::Extrinsic,
>;

/// Type alias for using the block request type using block type parameters.
pub type BlockRequest<B> =
	generic::BlockRequest<<B as BlockT>::Hash, <<B as BlockT>::Header as HeaderT>::Number>;

/// Type alias for using the BlockData type using block type parameters.
pub type BlockData<B> =
	generic::BlockData<<B as BlockT>::Header, <B as BlockT>::Hash, <B as BlockT>::Extrinsic>;

/// Type alias for using the BlockResponse type using block type parameters.
pub type BlockResponse<B> =
	generic::BlockResponse<<B as BlockT>::Header, <B as BlockT>::Hash, <B as BlockT>::Extrinsic>;

/// A set of transactions.
pub type Transactions<E> = Vec<E>;

// Bits of block data and associated artifacts to request.
bitflags! {
	/// Node roles bitmask.
	pub struct BlockAttributes: u8 {
		/// Include block header.
		const HEADER = 0b00000001;
		/// Include block body.
		const BODY = 0b00000010;
		/// Include block receipt.
		const RECEIPT = 0b00000100;
		/// Include block message queue.
		const MESSAGE_QUEUE = 0b00001000;
		/// Include a justification for the block.
		const JUSTIFICATION = 0b00010000;
		/// Include indexed transactions for a block.
		const INDEXED_BODY = 0b00100000;
	}
}

impl BlockAttributes {
	/// Encodes attributes as big endian u32, compatible with SCALE-encoding (i.e the
	/// significant byte has zero index).
	pub fn to_be_u32(&self) -> u32 {
		u32::from_be_bytes([self.bits(), 0, 0, 0])
	}

	/// Decodes attributes, encoded with the `encode_to_be_u32()` call.
	pub fn from_be_u32(encoded: u32) -> Result<Self, Error> {
		Self::from_bits(encoded.to_be_bytes()[0])
			.ok_or_else(|| Error::from("Invalid BlockAttributes"))
	}
}

impl Encode for BlockAttributes {
	fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
		dest.push_byte(self.bits())
	}
}

impl codec::EncodeLike for BlockAttributes {}

impl Decode for BlockAttributes {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		Self::from_bits(input.read_byte()?).ok_or_else(|| Error::from("Invalid bytes"))
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Encode, Decode)]
/// Block enumeration direction.
pub enum Direction {
	/// Enumerate in ascending order (from child to parent).
	Ascending = 0,
	/// Enumerate in descending order (from parent to canonical child).
	Descending = 1,
}

/// Block state in the chain.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Encode, Decode)]
pub enum BlockState {
	/// Block is not part of the best chain.
	Normal,
	/// Latest best block.
	Best,
}

/// Remote call response.
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct RemoteCallResponse {
	/// Id of a request this response was made for.
	pub id: RequestId,
	/// Execution proof.
	pub proof: StorageProof,
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
/// Remote read response.
pub struct RemoteReadResponse {
	/// Id of a request this response was made for.
	pub id: RequestId,
	/// Read proof.
	pub proof: StorageProof,
}

/// Announcement summary used for debug logging.
#[derive(Debug)]
pub struct AnnouncementSummary<H: HeaderT> {
	/// Announcement summary used for debug logging. block_hash
	pub block_hash: H::Hash,
	/// Announcement summary used for debug logging. number
	pub number: H::Number,
	/// Announcement summary used for debug logging. parent_hash
	pub parent_hash: H::Hash,
	/// Announcement summary used for debug logging. state
	pub state: Option<BlockState>,
}

impl<H: HeaderT> generic::BlockAnnounce<H> {
	/// Announcement summary used for debug logging.
	pub fn summary(&self) -> AnnouncementSummary<H> {
		AnnouncementSummary {
			block_hash: self.header.hash(),
			number: *self.header.number(),
			parent_hash: self.header.parent_hash().clone(),
			state: self.state,
		}
	}
}
/// Used for wrapping receiving time information of `block` or `adjust`
///Currently only used at NetworkWorker::poll()
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ReceiveTimestamp<B:BlockT>{
	/// Contain Adjust and timestamp
	AdjustTimestamp(AdjustTemplate<B>),
	/// Contain Block and timestamp
	BlockTimestamp(Vec<BlockTemplate<B>>),
}

/// Used for encoding valid BlockTemplates
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct BlockTemplates<B: BlockT>(Vec<BlockTemplate<B>>);

impl<B: BlockT> BlockTemplates<B> {
	/// Create a new BlockTemplates
	pub fn new(inner: Vec<BlockTemplate<B>>) -> Self {
		BlockTemplates(inner)
	}

	/// Get BlockTemplates inner length
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Transform into BlocksSimplified
	pub fn simplify(&self) -> BlocksSimplified<B> {
		let mut inner = Vec::new();

		for block in self.0.clone(){
			inner.push(block.simplify());
		}

		BlocksSimplified(inner)
	}

	/// Get inner blocks
	pub fn blocks(&self) -> Vec<BlockTemplate<B>>{
		self.clone().0
	}
}

///Wrapped information of `block` and it's receiving time.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockTemplate<B: BlockT>{
	/// Header information of the block
	pub block: <B as BlockT>::Header,

	/// Receive time of the block
	pub receive_time: u128,
}
/// `Adjust` will store this information,
/// and later stored on chain
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct BlockSimplified<B: BlockT>{
	/// Current block hash
	pub hash: <B as BlockT>::Hash,

	/// Parent block hash
	pub parent_hash: <B as BlockT>::Hash,

	/// Current block number
	pub number: <<B as BlockT>::Header as HeaderT>::Number,

	/// Block receive time
	pub receive_time: u128,

	/// Slot
	pub slot: Option<u64>
}

/// Used for adjust to store on chain
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct BlocksSimplified<B: BlockT>(Vec<BlockSimplified<B>>);

/// Most functions are used for validation.
impl<B:BlockT> BlocksSimplified<B>{

	/// gets inner block
	pub fn blocks(&self) -> Vec<BlockSimplified<B>>{
		self.clone().0
	}

	/// Check if there are block contain empty slot number.
	pub fn valid(&self) -> bool{
		let mut result = true;
		for block in self.0.clone(){
			if block.slot.is_none(){
				result = false;
				break
			}
		}
		result
	}
}

impl<B: BlockT> BlockTemplate<B>{
	/// Get block number
	pub fn number(&self) -> <<B as BlockT>::Header as HeaderT>::Number{
		*self.block.number()
	}

	/// Transform into simplified block
	pub fn simplify(&self) -> BlockSimplified<B>{
		let hash = self.block.hash();
		let &parent_hash = self.block.parent_hash();
		let &number = self.block.number();
		let receive_time = self.receive_time;
		let &engine_id = b"slot";
		let slot = if let Some(data) = self.block.digest().pre_runtime_id(engine_id){
			if let Ok(slot) = u64::decode(&mut data.as_slice()){
				Some(slot)
			} else{
				None
			}
		} else {
			None
		};
		BlockSimplified{hash, parent_hash, number, receive_time, slot}
	}

	/// Return inner slot
	pub fn slot(&self) -> Option<u64> {
		let &engine_id = b"slot";
		if let Some(data) = self.block.digest().pre_runtime_id(engine_id){
			if let Ok(block_slot) = u64::decode(&mut data.as_slice()){
				Some(block_slot)
			} else{
				None
			}
		} else {
			None
		}
	}

}

impl<B: BlockT> Encode for BlockTemplate<B>{
	fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
		self.block.encode_to(dest);
		self.receive_time.encode_to(dest);
	}
}

impl<B: BlockT> Decode for BlockTemplate<B> {
	fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
		let block =  <B as BlockT>::Header::decode(input)?;
		let receive_time = u128::decode(input)?;
		Ok(Self { block, receive_time })
	}
}


/// Used for encoding valid BlockTemplates
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct AdjustExtracts<B: BlockT>(Vec<Adjust<B>>);

impl<B: BlockT> AdjustExtracts<B> {

	/// Create a new AdjustExtracts from vector of AdjustTemplate.
	pub fn new_from_vec(input: Vec<AdjustTemplate<B>>) -> Self {
		let mut inner = Vec::new();

		for adjust_tmp in input {
			// let header = adjust_tmp.clone().adjust.header;
			let data = adjust_tmp.clone().adjust.data;

			// Check if adjust template contains block data
			if data.is_none() {
				log::warn!("Adjust contains no block");
				continue
			}
			let slot = adjust_tmp.slot();
			// Transform data into Adjust
			if let Ok(blocks) = BlockTemplates::<B>::decode(&mut data.unwrap().as_slice()){
				let simplified_blocks = blocks.simplify();
				let header = adjust_tmp.clone().adjust.header;

				inner.push(
					Adjust{
						hash: header.hash(),
						parent_hash: *header.parent_hash(),
						number: *header.number(),
						slot,
						send_time: adjust_tmp.clone().adjust.timestamp,
						receive_time: adjust_tmp.receive_time,
						blocks: Some(simplified_blocks),
					}
				)
			};

		}

		Self(inner)
	}

	/// Get inner vector length.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Get inner adjusts
	pub fn adjusts(&self) -> Vec<Adjust<B>>{
		self.clone().0
	}

	/// Get biggest slot
	pub fn biggest_slot(&self) -> Option<u64> {
		let mut slots = self.0.iter().map(|x|{
			x.clone().slot.unwrap_or(0)
		}).collect::<Vec<_>>();

		slots.sort();

		if let Some(&res) = slots.last(){
			Some(res)
		} else {
			None
		}

	}

	/// Get smallest slot
	pub fn smallest_slot(&self) -> Option<u64> {
		let mut slots = self.0.iter().map(|x|{
			x.clone().slot.unwrap_or(0)
		}).collect::<Vec<_>>();

		slots.sort();

		if let Some(&res) = slots.first(){
			Some(res)
		} else {
			None
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct Adjust<B: BlockT>{
	/// Current block hash
	pub hash: <B as BlockT>::Hash,

	/// Parent block hash
	pub parent_hash: <B as BlockT>::Hash,

	/// Current block number
	pub number: <<B as BlockT>::Header as HeaderT>::Number,

	/// Created slot number
	pub slot: Option<u64>,

	/// Create time or send time, generated creator.
	pub send_time: u128,

	/// Adjust receive time, generated locally
	pub receive_time: u128,

	pub blocks: Option<BlocksSimplified<B>>
}


///Wrapped information of `adjust` and it's receiving time.
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct AdjustTemplate<B: BlockT>{
	/// Adjust information
	pub adjust: AdjustAnnounce<<B as BlockT>::Header>,

	/// Adjust receive time, recorded locally
	pub receive_time: u128,
}

/// Only used for adjust announce validation process
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct AdjustAnnounceValidation<B: BlockT>{
	/// Create time. Or Send time.
	pub create_time: u128,
	/// Adjust receive time, recorded locally
	pub receive_time: u128,
	/// Data associated with this block announcement, e.g. a candidate message.
	pub block_templates: Option<Vec<u8>>,
	/// Proof of sender is leader
	pub sender_header: <B as BlockT>::Header,
}

impl<B: BlockT> AdjustAnnounceValidation<B> {

	/// Transformation
	pub fn as_template(&self) -> AdjustTemplate<B> {

		let adjust = AdjustAnnounce{
			header: self.clone().sender_header.clone(),
			timestamp: self.clone().create_time,
			state: None,
			data: self.clone().block_templates.clone(),
		};

		AdjustTemplate{
			adjust,
			receive_time: self.clone().receive_time
		}
	}

	/// Transformation
	pub fn from_template(template: AdjustTemplate<B>) -> Self {
		AdjustAnnounceValidation{
			create_time: template.clone().adjust.timestamp,
			receive_time: template.receive_time,
			block_templates: template.clone().adjust.data,
			sender_header:template.clone().adjust.header,
		}
	}

	/// Transform
	pub fn from_announce_and_timestamp(
		announce: AdjustAnnounce<<B as BlockT>::Header>,
		receive_time: u128,
	) -> Self {
		AdjustAnnounceValidation{
			create_time: announce.clone().timestamp,
			receive_time,
			block_templates: announce.clone().data,
			sender_header:announce.clone().header,
		}
	}
}


impl<B: BlockT> AdjustTemplate<B> {
	/// Create a new AdjustTemplate
	pub fn new(
		adjust: AdjustAnnounce<<B as BlockT>::Header>,
		receive_time: u128,
	) -> Self{

		Self {
			adjust,
			receive_time
		}
	}

	/// Decode self inner slot, if slot data is not empty
	pub fn slot(&self) -> Option<u64>{
		let &engine_id = b"slot";

		let header = self.clone().adjust.header;
		if let Some(digest) = header.digest().pre_runtime_id(engine_id){
			if let Ok(adjust_slot) = u64::decode(&mut digest.as_slice()) {
				Some(adjust_slot)
			} else {
				None
			}
		} else{
			None
		}
	}

	/// Check if inner data can be decoded
	pub fn check_block_validity(&self) -> bool {

		if let Some(raw_data) = self.clone().adjust.data{
			if let Ok(_blocks) = BlockTemplates::<B>::decode(&mut raw_data.as_slice()){
				return true
			}
		}

		false
	}

	/// Check if
	/// 1. slot in the header of the adjust
	/// 2. slot in each block inside of inner Block
	/// is less than a certain number
	pub fn created_before_slot(&self, slot: u64) -> bool {
		let blocks_data = self.clone().adjust.data;

		// Block data should not be empty
		if blocks_data.is_none(){
			return false
		}

		let &engine_id = b"slot";

		// Check adjust
		let header = self.clone().adjust.header;
		if let Some(digest) = header.digest().pre_runtime_id(engine_id){
			if let Ok(adjust_slot) = u64::decode(&mut digest.as_slice()) {
				if slot <= adjust_slot {
					log::info!("[ERROR] Slot in adjust of {:?}({:?}) should not be greater or equal than current slot {:?} ",
						header.number(), header.hash(), slot
					);
					return false
				}

			} else {
				log::info!("[ERROR] Slot info. decode error in Adjust {:?}, {:?}", header.number(), header.hash());
				return false
			};
		} else{
			log::info!("[ERROR] No slot info. in Adjust {:?}, {:?}", header.number(), header.hash());
			return false
		};

		// Check inner blocks
		if let Ok(blocks) = BlockTemplates::<B>::decode(&mut blocks_data.unwrap().as_slice()){
			for block in blocks.blocks(){
				if let Some(block_slot) = block.slot(){
					if slot < block_slot  {
						return false
					}
				} else {
					// Empty slot data
					return false
				}
			}
			true
		} else {
			false
		}
	}
}

/// Generic types.
pub mod generic {
	use super::{
		BlockAttributes, BlockState, ConsensusEngineId, Direction, RemoteCallResponse,
		RemoteReadResponse, RequestId, StorageProof, Transactions,
	};
	use bitflags::bitflags;
	use codec::{Decode, Encode, Input, Output};
	use sp_runtime::{EncodedJustification, Justifications};

	bitflags! {
		/// Bitmask of the roles that a node fulfills.
		pub struct Roles: u8 {
			/// No network.
			const NONE = 0b00000000;
			/// Full node, does not participate in consensus.
			const FULL = 0b00000001;
			/// Light client node.
			const LIGHT = 0b00000010;
			/// Act as an authority
			const AUTHORITY = 0b00000100;
		}
	}

	impl Roles {
		/// Does this role represents a client that holds full chain data locally?
		pub fn is_full(&self) -> bool {
			self.intersects(Self::FULL | Self::AUTHORITY)
		}

		/// Does this role represents a client that does not participates in the consensus?
		pub fn is_authority(&self) -> bool {
			*self == Self::AUTHORITY
		}

		/// Does this role represents a client that does not hold full chain data locally?
		pub fn is_light(&self) -> bool {
			!self.is_full()
		}
	}

	impl<'a> From<&'a crate::config::Role> for Roles {
		fn from(roles: &'a crate::config::Role) -> Self {
			match roles {
				crate::config::Role::Full => Self::FULL,
				crate::config::Role::Light => Self::LIGHT,
				crate::config::Role::Authority { .. } => Self::AUTHORITY,
			}
		}
	}

	impl codec::Encode for Roles {
		fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
			dest.push_byte(self.bits())
		}
	}

	impl codec::EncodeLike for Roles {}

	impl codec::Decode for Roles {
		fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
			Self::from_bits(input.read_byte()?).ok_or_else(|| codec::Error::from("Invalid bytes"))
		}
	}

	/// Consensus is mostly opaque to us
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct ConsensusMessage {
		/// Identifies consensus engine.
		pub protocol: ConsensusEngineId,
		/// Message payload.
		pub data: Vec<u8>,
	}

	/// Block data sent in the response.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct BlockData<Header, Hash, Extrinsic> {
		/// Block header hash.
		pub hash: Hash,
		/// Block header if requested.
		pub header: Option<Header>,
		/// Block body if requested.
		pub body: Option<Vec<Extrinsic>>,
		/// Block body indexed transactions if requested.
		pub indexed_body: Option<Vec<Vec<u8>>>,
		/// Block receipt if requested.
		pub receipt: Option<Vec<u8>>,
		/// Block message queue if requested.
		pub message_queue: Option<Vec<u8>>,
		/// Justification if requested.
		pub justification: Option<EncodedJustification>,
		/// Justifications if requested.
		pub justifications: Option<Justifications>,
	}

	/// Identifies starting point of a block sequence.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub enum FromBlock<Hash, Number> {
		/// Start with given hash.
		Hash(Hash),
		/// Start with given block number.
		Number(Number),
	}

	/// A network message.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub enum Message<Header, Hash, Number, Extrinsic> {
		/// Status packet.
		Status(Status<Hash, Number>),
		/// Block request.
		BlockRequest(BlockRequest<Hash, Number>),
		/// Block response.
		BlockResponse(BlockResponse<Header, Hash, Extrinsic>),
		/// Block announce.
		BlockAnnounce(BlockAnnounce<Header>),
		/// Transactions.
		Transactions(Transactions<Extrinsic>),
		/// Consensus protocol message.
		Consensus(ConsensusMessage),
		/// Remote method call request.
		RemoteCallRequest(RemoteCallRequest<Hash>),
		/// Remote method call response.
		RemoteCallResponse(RemoteCallResponse),
		/// Remote storage read request.
		RemoteReadRequest(RemoteReadRequest<Hash>),
		/// Remote storage read response.
		RemoteReadResponse(RemoteReadResponse),
		/// Remote header request.
		RemoteHeaderRequest(RemoteHeaderRequest<Number>),
		/// Remote header response.
		RemoteHeaderResponse(RemoteHeaderResponse<Header>),
		/// Remote changes request.
		RemoteChangesRequest(RemoteChangesRequest<Hash>),
		/// Remote changes response.
		RemoteChangesResponse(RemoteChangesResponse<Number, Hash>),
		/// Remote child storage read request.
		RemoteReadChildRequest(RemoteReadChildRequest<Hash>),
		/// Batch of consensus protocol messages.
		// NOTE: index is incremented by 2 due to finality proof related
		// messages that were removed.
		#[codec(index = 17)]
		ConsensusBatch(Vec<ConsensusMessage>),
	}

	/// Status sent on connection.
	// TODO https://github.com/paritytech/substrate/issues/4674: replace the `Status`
	// struct with this one, after waiting a few releases beyond `NetworkSpecialization`'s
	// removal (https://github.com/paritytech/substrate/pull/4665)
	//
	// and set MIN_VERSION to 6.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct CompactStatus<Hash, Number> {
		/// Protocol version.
		pub version: u32,
		/// Minimum supported version.
		pub min_supported_version: u32,
		/// Supported roles.
		pub roles: Roles,
		/// Best block number.
		pub best_number: Number,
		/// Best block hash.
		pub best_hash: Hash,
		/// Genesis block hash.
		pub genesis_hash: Hash,
	}

	/// Status sent on connection.
	#[derive(Debug, PartialEq, Eq, Clone, Encode)]
	pub struct Status<Hash, Number> {
		/// Protocol version.
		pub version: u32,
		/// Minimum supported version.
		pub min_supported_version: u32,
		/// Supported roles.
		pub roles: Roles,
		/// Best block number.
		pub best_number: Number,
		/// Best block hash.
		pub best_hash: Hash,
		/// Genesis block hash.
		pub genesis_hash: Hash,
		/// DEPRECATED. Chain-specific status.
		pub chain_status: Vec<u8>,
	}

	impl<Hash: Decode, Number: Decode> Decode for Status<Hash, Number> {
		fn decode<I: Input>(value: &mut I) -> Result<Self, codec::Error> {
			const LAST_CHAIN_STATUS_VERSION: u32 = 5;
			let compact = CompactStatus::decode(value)?;
			let chain_status = match <Vec<u8>>::decode(value) {
				Ok(v) => v,
				Err(e) =>
					if compact.version <= LAST_CHAIN_STATUS_VERSION {
						return Err(e)
					} else {
						Vec::new()
					},
			};

			let CompactStatus {
				version,
				min_supported_version,
				roles,
				best_number,
				best_hash,
				genesis_hash,
			} = compact;

			Ok(Self {
				version,
				min_supported_version,
				roles,
				best_number,
				best_hash,
				genesis_hash,
				chain_status,
			})
		}
	}

	/// Request block data from a peer.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct BlockRequest<Hash, Number> {
		/// Unique request id.
		pub id: RequestId,
		/// Bits of block data to request.
		pub fields: BlockAttributes,
		/// Start from this block.
		pub from: FromBlock<Hash, Number>,
		/// End at this block. An implementation defined maximum is used when unspecified.
		pub to: Option<Hash>,
		/// Sequence direction.
		pub direction: Direction,
		/// Maximum number of blocks to return. An implementation defined maximum is used when
		/// unspecified.
		pub max: Option<u32>,
	}

	/// Response to `BlockRequest`
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct BlockResponse<Header, Hash, Extrinsic> {
		/// Id of a request this response was made for.
		pub id: RequestId,
		/// Block data for the requested sequence.
		pub blocks: Vec<BlockData<Header, Hash, Extrinsic>>,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub enum AnnounceMessage<Header>{
		AdjustAnnounce(AdjustAnnounce<Header>),
		BlockAnnounce(BlockAnnounce<Header>),
	}

	/// Announce a new complete relay chain block on the network.
	#[derive(Debug, PartialEq, Eq, Clone)]
	pub struct BlockAnnounce<H> {
		/// New block header.
		pub header: H,
		/// Block state. TODO: Remove `Option` and custom encoding when v4 becomes common.
		pub state: Option<BlockState>,
		/// Data associated with this block announcement, e.g. a candidate message.
		pub data: Option<Vec<u8>>,
	}

	// Custom Encode/Decode impl to maintain backwards compatibility with v3.
	// This assumes that the packet contains nothing but the announcement message.
	// TODO: Get rid of it once protocol v4 is common.
	impl<H: Encode> Encode for BlockAnnounce<H> {
		fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
			self.header.encode_to(dest);
			if let Some(state) = &self.state {
				state.encode_to(dest);
			}
			if let Some(data) = &self.data {
				data.encode_to(dest)
			}
		}
	}

	impl<H: Decode> Decode for BlockAnnounce<H> {
		fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
			let header = H::decode(input)?;
			let state = BlockState::decode(input).ok();
			let data = Vec::decode(input).ok();
			Ok(Self { header, state, data })
		}
	}

	/// Announce a new complete relay chain block Adjust information on the network.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct AdjustAnnounce<H> {
		/// New block header. Currently only header hash is useful
		pub header: H,
		/// Create time. Or Send time.
		pub timestamp: u128,
		/// Block state. TODO: Remove `Option` and custom encoding when v4 becomes common.
		pub state: Option<BlockState>,
		/// Data associated with this block announcement, e.g. a candidate message.
		pub data: Option<Vec<u8>>,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	/// Remote call request.
	pub struct RemoteCallRequest<H> {
		/// Unique request id.
		pub id: RequestId,
		/// Block at which to perform call.
		pub block: H,
		/// Method name.
		pub method: String,
		/// Call data.
		pub data: Vec<u8>,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	/// Remote storage read request.
	pub struct RemoteReadRequest<H> {
		/// Unique request id.
		pub id: RequestId,
		/// Block at which to perform call.
		pub block: H,
		/// Storage key.
		pub keys: Vec<Vec<u8>>,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	/// Remote storage read child request.
	pub struct RemoteReadChildRequest<H> {
		/// Unique request id.
		pub id: RequestId,
		/// Block at which to perform call.
		pub block: H,
		/// Child Storage key.
		pub storage_key: Vec<u8>,
		/// Storage key.
		pub keys: Vec<Vec<u8>>,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	/// Remote header request.
	pub struct RemoteHeaderRequest<N> {
		/// Unique request id.
		pub id: RequestId,
		/// Block number to request header for.
		pub block: N,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	/// Remote header response.
	pub struct RemoteHeaderResponse<Header> {
		/// Id of a request this response was made for.
		pub id: RequestId,
		/// Header. None if proof generation has failed (e.g. header is unknown).
		pub header: Option<Header>,
		/// Header proof.
		pub proof: StorageProof,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	/// Remote changes request.
	pub struct RemoteChangesRequest<H> {
		/// Unique request id.
		pub id: RequestId,
		/// Hash of the first block of the range (including first) where changes are requested.
		pub first: H,
		/// Hash of the last block of the range (including last) where changes are requested.
		pub last: H,
		/// Hash of the first block for which the requester has the changes trie root. All other
		/// affected roots must be proved.
		pub min: H,
		/// Hash of the last block that we can use when querying changes.
		pub max: H,
		/// Storage child node key which changes are requested.
		pub storage_key: Option<Vec<u8>>,
		/// Storage key which changes are requested.
		pub key: Vec<u8>,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	/// Remote changes response.
	pub struct RemoteChangesResponse<N, H> {
		/// Id of a request this response was made for.
		pub id: RequestId,
		/// Proof has been generated using block with this number as a max block. Should be
		/// less than or equal to the RemoteChangesRequest::max block number.
		pub max: N,
		/// Changes proof.
		pub proof: Vec<Vec<u8>>,
		/// Changes tries roots missing on the requester' node.
		pub roots: Vec<(N, H)>,
		/// Missing changes tries roots proof.
		pub roots_proof: StorageProof,
	}
}
