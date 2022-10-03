// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

//! Utility stream for yielding slots in a loop.
//!
//! This is used instead of `futures_timer::Interval` because it was unreliable.

use super::{InherentDataProviderExt, Slot};
use sp_consensus::{Error, SelectChain};
use sp_inherents::{CreateInherentDataProviders, InherentData, InherentDataProvider};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_runtime::generic::BlockId;

use sp_api::ProvideRuntimeApi;
use sc_client_api::{backend::AuxStore, BlockchainEvents, ProvideUncles};
use sp_blockchain::{Error as ClientError, HeaderMetadata};
use sp_consensus_babe::BabeApi;
use sp_block_builder::BlockBuilder;
use codec::Decode;
use sc_network::protocol::message::AdjustExtracts;

use sc_client_api::UsageProvider;
use sc_client_api::client::BlockBackend;
use sp_blockchain::HeaderBackend;

use std::time::SystemTime;
use crate::{
	ERA_DURATION_IN_SLOTS, SLOT_DURATION,
	MIN_MILLISECS_PER_BLOCK, MAX_MILLISECS_PER_BLOCK,
	EPOCH_DURATION_IN_SLOTS, W1, W2
};

use futures_timer::Delay;
use std::time::{Duration, Instant};
use std::sync::Arc;
/// Returns current duration since unix epoch.
pub fn duration_now() -> Duration {
	let now = SystemTime::now();
	now.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_else(|e| {
		panic!("Current time {:?} is before unix epoch. Something is wrong: {:?}", now, e)
	})
}

/// Returns the duration until the next slot from now.
pub fn time_until_next_slot(slot_duration: Duration) -> Duration {
	let now = duration_now().as_millis();

	let next_slot = (now + slot_duration.as_millis()) / slot_duration.as_millis();
	let remaining_millis = next_slot * slot_duration.as_millis() - now;
	Duration::from_millis(remaining_millis as u64)
}

/// Information about a slot.
pub struct SlotInfo<B: BlockT> {
	/// The slot number as found in the inherent data.
	pub slot: Slot,
	/// Current timestamp as found in the inherent data.
	pub timestamp: sp_timestamp::Timestamp,
	/// The instant at which the slot ends.
	pub ends_at: Instant,
	/// The inherent data.
	pub inherent_data: InherentData,
	/// Slot duration.
	pub duration: Duration,
	/// The chain header this slot is based on.
	pub chain_head: B::Header,
	/// Some potential block size limit for the block to be authored at this slot.
	///
	/// For more information see [`Proposer::propose`](sp_consensus::Proposer::propose).
	pub block_size_limit: Option<usize>,
}

impl<B: BlockT> SlotInfo<B> {
	/// Create a new [`SlotInfo`].
	///
	/// `ends_at` is calculated using `timestamp` and `duration`.
	pub fn new(
		slot: Slot,
		timestamp: sp_timestamp::Timestamp,
		inherent_data: InherentData,
		duration: Duration,
		chain_head: B::Header,
		block_size_limit: Option<usize>,
	) -> Self {
		Self {
			slot,
			timestamp,
			inherent_data,
			duration,
			chain_head,
			block_size_limit,
			ends_at: Instant::now() + time_until_next_slot(duration),
		}
	}
}

/// A stream that returns every time there is a new slot.
pub(crate) struct Slots<Block, C, IDP> {
	last_slot: Slot,
	slot_duration: Duration,
	inner_delay: Option<Delay>,
	create_inherent_data_providers: IDP,
	client: C,
	_phantom: std::marker::PhantomData<Block>,
}

impl<Block, C, IDP> Slots<Block, C, IDP> {
	/// Create a new `Slots` stream.
	pub fn new(slot_duration: Duration, create_inherent_data_providers: IDP, client: C) -> Self {
		Slots {
			last_slot: 0.into(),
			slot_duration,
			inner_delay: None,
			create_inherent_data_providers,
			client,
			_phantom: Default::default(),
		}
	}
}

impl<Block, C, IDP> Slots<Block, C, IDP>
where
	Block: BlockT,
	C: SelectChain<Block>,
	IDP: CreateInherentDataProviders<Block, ()>,
	IDP::InherentDataProviders: crate::InherentDataProviderExt,

{
	/// Returns a future that fires when the next slot starts.
	pub async fn next_slot(&mut self) -> Result<SlotInfo<Block>, Error> {
		loop {

			// Calculate left time and set inner_delay
			self.inner_delay = match self.inner_delay.take() {
				None => {
					// schedule wait.
					let wait_dur = time_until_next_slot(self.slot_duration);
					Some(Delay::new(wait_dur))
				},
				Some(d) => Some(d),
			};

			// log::info!("before inner_delay.await;");

			// Wait until time to expire
			if let Some(inner_delay) = self.inner_delay.take() {
				inner_delay.await;
			}

			// log::info!("after inner_delay.await;");
			// timeout has fired.
			// During this time, other running task maintain block import
			let ends_in = time_until_next_slot(self.slot_duration);
			// log::info!("slots.next_slot() {}", line!());
			// reschedule delay for next slot.
			self.inner_delay = Some(Delay::new(ends_in));

			let ends_at = Instant::now() + ends_in;

			let chain_head = match self.client.best_chain().await {
				Ok(x) => x,
				Err(e) => {
					log::warn!(
						target: "slots",
						"Unable to author block in slot. No best block header: {:?}",
						e,
					);
					// Let's try at the next slot..
					self.inner_delay.take();
					continue
				},
			};

			let inherent_data_providers = self
				.create_inherent_data_providers
				.create_inherent_data_providers(chain_head.hash(), ())
				.await?;

			if Instant::now() > ends_at {
				log::warn!(
					target: "slots",
					"Creating inherent data providers took more time than we had left for the slot.",
				);
			}

			let timestamp = inherent_data_providers.timestamp();
			let slot = inherent_data_providers.slot();
			let inherent_data = inherent_data_providers.create_inherent_data()?;

			// Inherent Data
			{
				// log::info!("inherent_data len {}", inherent_data.len());
				// let inherent_identifier = *b"testnets";// [u8;8]
				// let inherent_0 = vec![1, 2, 3];
				// inherent_data.put_data(inherent_identifier,&inherent_0).unwrap();
			}

			// never yield the same slot twice.
			if slot > self.last_slot {
				self.last_slot = slot;
				// log::info!("slots.next_slot() return");
				break Ok(SlotInfo::new(
					slot,
					timestamp,
					inherent_data,
					self.slot_duration,
					chain_head,
					None,
				))
			}
		}
	}

	pub async fn next_slot_with_client<Client> (&mut self, client: Option<Arc<Client>>) -> Result<SlotInfo<Block>, Error>
		where
			Client:	 ProvideRuntimeApi<Block>
			+ ProvideUncles<Block>
			+ BlockchainEvents<Block>
			+ AuxStore
			+ UsageProvider<Block>
			+ HeaderBackend<Block>
			+ HeaderMetadata<Block, Error = ClientError>
			+ BlockBackend<Block>
			+ Send
			+ Sync
			+ 'static,
			Client::Api: BabeApi<Block> + BlockBuilder<Block>,
	{
		let client = client.unwrap();
		loop {
			// Calculate left time and set inner_delay
			self.inner_delay = match self.inner_delay.take() {
				None => {
					// schedule wait.
					let wait_dur = if let Some((slot, era, length, start_time))
						= calculate_current_slot(client.clone())
					{
						log::info!("[A Nxt] slot {} era {}, length {}, start_time {}", slot, era, length, start_time);
						let now = duration_now().as_millis();
						let remaining_millis = start_time + length as u128 - now;
						Duration::from_millis(remaining_millis as u64)
					} else {
						log::info!("[A Nxt] Using default time_until_next_slot()");
						time_until_next_slot(self.slot_duration)
					};

					// let wait_dur = time_until_next_slot(self.slot_duration);

					Some(Delay::new(wait_dur))
				},
				Some(d) => Some(d),
			};

			// SlotEnd functionality
			// Wait until time to expire
			if let Some(inner_delay) = self.inner_delay.take() {
				inner_delay.await;
			}

			log::info!("");
			log::info!("");

			// Timeout has fired. New slot has began
			// During this time, other running task maintain block import

			let mut slot_res: Option<Slot>  = None ;
			let ends_in = if let Some((slot_in, era, length, start_time))
				= calculate_current_slot(client.clone())
			{
				log::info!("[A Nxt] slot {} era {}, length {}, start_time {}", slot_in, era, length, start_time);
				let now = duration_now().as_millis();
				let remaining_millis = start_time + length as u128 - now;
				slot_res = Some(Slot::from(slot_in));
				Duration::from_millis(remaining_millis as u64)

			} else{
				log::info!("[A Nxt] Using default time_until_next_slot()");
				time_until_next_slot(self.slot_duration)
			};

			// let ends_in = time_until_next_slot(self.slot_duration);
			// log::info!("slots.next_slot() {}", line!());
			// reschedule delay for next slot.
			self.inner_delay = Some(Delay::new(ends_in));

			let ends_at = Instant::now() + ends_in;

			let chain_head = match self.client.best_chain().await {
				Ok(x) => x,
				Err(e) => {
					log::warn!(
						target: "slots",
						"Unable to author block in slot. No best block header: {:?}",
						e,
					);
					// Let's try at the next slot..
					self.inner_delay.take();
					continue
				},
			};

			let inherent_data_providers = self
				.create_inherent_data_providers
				.create_inherent_data_providers(chain_head.hash(), ())
				.await?;

			if Instant::now() > ends_at {
				log::warn!(
					target: "slots",
					"Creating inherent data providers took more time than we had left for the slot.",
				);
			}

			let timestamp = inherent_data_providers.timestamp();

			let slot = if let Some(slot) = slot_res{
				slot
			} else{
				inherent_data_providers.slot()
			};

			let inherent_data = inherent_data_providers.create_inherent_data()?;

			// Inherent Data
			{
				// log::info!("inherent_data len {}", inherent_data.len());
				// let inherent_identifier = *b"testnets";// [u8;8]
				// let inherent_0 = vec![1, 2, 3];
				// inherent_data.put_data(inherent_identifier,&inherent_0).unwrap();
			}

			// never yield the same slot twice.
			if slot > self.last_slot {
				self.last_slot = slot;
				// log::info!("slots.next_slot() return");
				break Ok(SlotInfo::new(
					slot,
					timestamp,
					inherent_data,
					self.slot_duration,
					chain_head,
					None,
				))
			}
		}
	}

}

/// Keep track of each known era with it's slot length
#[derive(Default, Debug)]
pub struct EraSlot(Vec<u64>);

impl EraSlot{
	/// Create a given length EraSlot
	pub fn new(length: usize) -> Self{
		Self(vec![0; length])
	}

	/// Set a value for a certain index
	pub fn set_value(&mut self, index: usize, value: u64) {
		if index >= self.0.len() {
			self.0.resize(index + 1, 0 );
		}
		self.0[index] = value;
	}

	/// Get record slot length for an era
	pub fn era_slot_length(&self, era:usize) -> u64 {
		if era >= self.0.len() {
			0
		} else {
			self.0[era]
		}
	}
}

/// Calculate slot length
/// In this model Era length in slots should be at least twice as Epoch length in slots
pub fn calculate_current_slot<Client, B>(
	client: Arc<Client>,
) -> Option<(u64,<<B as BlockT>::Header as HeaderT>::Number,u64, u128)>
	where
	Client: UsageProvider<B>
	+ HeaderBackend<B>
	+ BlockBackend<B>,
	B: BlockT
{
	let w1 = W1;
	let w2 = W2;
	let w3 = 1.0 - w1 - w2 ;
	//
	let best_block_number = client.clone().usage_info().chain.best_number;
	let zero = as_number::<B>(0u32);
	let one = as_number::<B>(1u32);

	let era_length = as_number::<B>(ERA_DURATION_IN_SLOTS as u32); // currently `1 Era = 2 Epoch`
	//
	let target_era = best_block_number / era_length;
	let mut slot_length_set = EraSlot::new(into_u32::<B>(target_era) as usize);

	// Get genesis slot and time

	let mut genesis_time: u128 = 0;
	let mut genesis_slot: u64 = 0;
	if let Ok(Some(block_one_hash)) = (*client).block_hash(one){
		let engine_id = *b"slot";
		if let Some(adjust_raw) = (*client).adjusts_raw(engine_id, &BlockId::hash(block_one_hash)){
			match Slot::decode(&mut adjust_raw.as_slice()){
				Ok(a) => {
					genesis_slot = u64::from(a);
					genesis_time = (u64::from(a) as u128) * (SLOT_DURATION as u128);
				},
				Err(e) => {
					log::error!("[Test] Genesis Error {:?}", e);
				},
			};
		} else{
			log::error!("[Test] Genesis Error");
		}
	};

	// check and make sure `genesis_time > 0` and `genesis_slot > 0`
	if genesis_time <= 0 || genesis_slot <= 0{
		return None
	}
	log::debug!("[Test] Genesis Slot {}, Genesis Time {:?} target_era {:?}", genesis_slot, genesis_time, target_era);

	//
	let mut counter = 0;
	let slot_length_init = SLOT_DURATION ;
	let mut slot_length = slot_length_init;

	// Enum from 0 to best_block_number with 1 Era at a step
	// block 0 is excluded for that it does not contain useful adjust information
	let now = duration_now().as_millis();
	let mut current_era = zero;
	let mut current_time = genesis_time;
	let mut current_block = one;
	let mut current_slot = genesis_slot;
	log::debug!("[Test] before loop now {:?}, slot_length_init {:?}, genesis_slot {:?}, genesis_time {:?}, counter {:?},",
		now, slot_length, current_slot, current_time, counter,
	);
	{
		loop {

			if current_era == zero {
				// At first Era, slot length is the initial slot length
				slot_length = slot_length_init;

				slot_length_set.set_value(0, slot_length_init);

				current_time += (slot_length as u128) * (ERA_DURATION_IN_SLOTS as u128);

				current_slot += ERA_DURATION_IN_SLOTS;

				counter += 1;
			} else if current_era == one {
				// At second Era, slot length is calculated differently than the following era

				// Slot interval used to calculate new slot length
				let start_slot = genesis_slot + EPOCH_DURATION_IN_SLOTS ;
				let end_slot = genesis_slot + ERA_DURATION_IN_SLOTS - EPOCH_DURATION_IN_SLOTS / 2;
				//
				let last_slot_length = slot_length_init;
				let this_slot_length = last_slot_length;
				let start_time = genesis_time + EPOCH_DURATION_IN_SLOTS as u128 * SLOT_DURATION as u128;

				let default_exit = counter + 2 * EPOCH_DURATION_IN_SLOTS;
				let mut slot_pointer = genesis_slot;
				let mut delay = AverageDelay::new();

				// Calculate AdjustExtracts in each Block for delay
				loop {

					if let Some(adjusts) = extract_block_data(client.clone(), current_block){
						// log::info!("current_block [{}] ", current_block);
						if adjusts.biggest_slot().is_none() {
							counter += 1;
							current_block = current_block + one;
							continue
						}

						slot_pointer = adjusts.biggest_slot().unwrap();

						log::trace!("current_block [{}] slot_pointer {:?}", current_block, slot_pointer);
						log::trace!("start_slot {:?} end_slot {:?} this_slot_length {} start_time {}", start_slot, end_slot, this_slot_length, start_time);

						let res = deal_adjusts(adjusts, start_slot, end_slot, zero, this_slot_length, last_slot_length, start_time);

						if let Some((adjust_delay, block_delay)) = res {
							log::trace!("Block [{}] (a,b) = {:?}", current_block, res);
							delay.insert_adjust_block(adjust_delay, block_delay);
						}

					} else {
						log::error!("[Error] 1 Block [{}] not found", current_block);
						return None;
					}

					if slot_pointer >= end_slot {
						break
					}

					if counter >= default_exit{
						log::error!("[Error] Using default exit Era count");
						break;
					}

					counter += 1;
					current_block = current_block + one;
				}

				let (average_adjust_delay, average_block_delay) = delay.average_adjust_block_delay();

				let era_1_slot_length = (w3 * SLOT_DURATION as f64 + w2 * average_adjust_delay as f64 + w1 * average_block_delay as f64) as u64;

				log::debug!("Era 1 slot length {}*{} + {}*{} + {}*{} = {}",
					w3, SLOT_DURATION, w2, average_adjust_delay, w1, average_block_delay, era_1_slot_length
				);

				// Calculated results
				slot_length = in_between(MAX_MILLISECS_PER_BLOCK, MIN_MILLISECS_PER_BLOCK, era_1_slot_length);

				// Record results
				slot_length_set.set_value(1, slot_length);

				// Mark current time, until Era 1 end
				current_time += (slot_length as u128) * (ERA_DURATION_IN_SLOTS as u128);

				// Mark current Era, until Era 1 end
				current_slot += ERA_DURATION_IN_SLOTS;

			} else {
				// At Era n, slot length need to be calculated

				// Slot interval used to calculate new slot length
				let start_slot = current_slot - ERA_DURATION_IN_SLOTS ;
				let end_slot = current_slot - EPOCH_DURATION_IN_SLOTS / 2;

				//
				let last_slot_length = slot_length_set.era_slot_length(into_u32::<B>(current_era) as usize - 2);
				let this_slot_length = slot_length_set.era_slot_length(into_u32::<B>(current_era) as usize - 1);
				let start_time = current_time - (ERA_DURATION_IN_SLOTS * this_slot_length) as u128;

				let default_exit = counter + ERA_DURATION_IN_SLOTS + 1;
				let mut slot_pointer = start_slot - EPOCH_DURATION_IN_SLOTS / 2;
				let mut delay = AverageDelay::new();
				log::debug!("last_slot_length {} this_slot_length {} start_time {}", last_slot_length, this_slot_length, start_time);
				loop {

					if let Some(adjusts) = extract_block_data(client.clone(), current_block){
						// log::info!("current_block [{}] ", current_block);
						if adjusts.biggest_slot().is_none() {
							counter += 1;
							current_block = current_block + one;
							continue
						}

						slot_pointer = adjusts.biggest_slot().unwrap();

						log::debug!("current_block [{}] slot_pointer {:?}", current_block, slot_pointer);
						log::debug!("start_slot {:?} end_slot {:?} this_slot_length {} start_time {}", start_slot, end_slot, this_slot_length, start_time);

						let res = deal_adjusts(adjusts, start_slot, end_slot, current_era - one, this_slot_length, last_slot_length, start_time);

						if let Some((adjust_delay, block_delay)) = res {
							log::trace!("Block [{}] (a,b) = {:?}", current_block, res);
							delay.insert_adjust_block(adjust_delay, block_delay);
						}

					} else {
						log::error!("[Error] Era n Block [{}] not found", current_block);
						return None;
					}

					if slot_pointer >= end_slot {
						break
					}

					if counter >= default_exit{
						log::error!("[Error] Using default exit Era count");
						break;
					}

					counter += 1;
					current_block = current_block + one;
				}

				let (average_adjust_delay, average_block_delay) = delay.average_adjust_block_delay();

				let era_n_slot_length = (w3 * this_slot_length as f64 + w2 * average_adjust_delay as f64 + w1 * average_block_delay as f64) as u64;

				log::debug!("Era {} slot length {}*{} + {}*{} + {}*{} = {}",
					current_era, w3, this_slot_length, w2, average_adjust_delay, w1, average_block_delay, era_n_slot_length
				);

				// Calculated results
				slot_length = in_between(MAX_MILLISECS_PER_BLOCK, MIN_MILLISECS_PER_BLOCK, era_n_slot_length);

				// Record results
				slot_length_set.set_value(into_u32::<B>(current_era) as usize, slot_length);

				// Mark current time, until Era n-1 end
				current_time += (slot_length as u128) * (ERA_DURATION_IN_SLOTS as u128);

				// Mark current Era, until Era n-1 end
				current_slot += ERA_DURATION_IN_SLOTS;

			}

			if current_time > now {
				loop{
					if current_time <= now{
						log::debug!("current_time < now => {} < {}", current_time, now);
						break
					}

					current_time -= slot_length as u128;
					current_slot -= 1;
				}
				break;
			}

			current_era = current_era + one;
		}
	}
	log::debug!("[Test] after loop now {:?}, slot_length {:?}, current_era {:?},  current_block {:?}, current_slot {:?}, current_time {:?}, counter {:?},",
		now, slot_length, current_era,  current_block, current_slot, current_time, counter,
	);
	let slot_start_time = current_time;
	let out = (current_slot, current_era, slot_length, slot_start_time);//
	log::debug!("[Test] loop {:?} times", counter);

	Some(out)

}

pub(crate) fn extract_block_data<Client, B>(client: Arc<Client>,  number: <<B as BlockT>::Header as HeaderT>::Number)
	-> Option<AdjustExtracts<B>>
where
	Client: BlockBackend<B>,
	B: BlockT,
 {
	let engine_id = *b"ajst";
	let adjust = if let Ok(Some(hash)) = (*client).block_hash(number){
		if let Some(adjust_raw) = (*client).adjusts_raw(engine_id, &BlockId::hash(hash)){
			match AdjustExtracts::<B>::decode(&mut adjust_raw.as_slice()){
				Ok(a) => {
					Some(a)
				},
				Err(e) => {
					log::info!("[Error][Test] extract_block_data adjust_raw error {:?}", e);
					None
				},
			}
		} else {
			log::info!("[Test] extract_block_data get no adjust_raw");
			None
		}
	} else {
		None
	};

	adjust

}
/// Return value between max and min
fn in_between(max: u64, min: u64, num: u64) -> u64 {
	if max < num{
		max
	} else if min < num && num < max {
		num
	} else {
		min
	}
}

/// Calculate `average_adjust_delay`, `average_block_delay` between two given slot.
/// An AdjustExtracts contain multiple Adjusts.
/// An Adjust contains multiple Blocks,
/// `average_adjust_delay` is calculated from multiple Adjusts,
/// `average_block_delay` is calculated from multiple Blocks.
/// Option<(i32, i32)> => Option<(average_adjust_delay, average_block_delay)>.
pub fn deal_adjusts<B:BlockT>(
	adjusts: AdjustExtracts<B>,
	era_start_slot: u64,
	end_slot: u64,
	era: <<B as BlockT>::Header as HeaderT>::Number, // currently useless
	this_slot_length: u64,
	last_slot_length: u64,
	start_time: u128
) -> Option<(i32, i32)>{
	let mut average_adjust_delay: i32 = 0 ;
	let mut average_block_delay: i32 = 0 ;

	if era_start_slot > end_slot {
		log::error!("[Error] start_slot > end_slot");
		return None
	}

	let mut adjust_number = 0;
	let mut block_number = 0;

	let mut sum_adjust_delay = 0;
	let mut sum_block_delay = 0;

	for adjust in adjusts.adjusts() {
		if adjust.slot.is_none() {
			log::error!("[Error] adjust.slot.is_none()");
			return None
		}

		let slot = adjust.slot.unwrap();

		if era_start_slot - EPOCH_DURATION_IN_SLOTS / 2 <= slot && slot < end_slot {

			// calculate adjust delay
			let delay = if adjust.send_time > adjust.receive_time {
				(adjust.send_time - adjust.receive_time) as i32
			} else {
				- ((adjust.receive_time - adjust.send_time) as i32)
			};

			log::trace!("adjust.receive_time {}, adjust.receive_time {} slot {:?} {}", adjust.receive_time, adjust.receive_time, slot, line!());

			sum_adjust_delay += delay;

			if adjust.blocks.is_none(){
				log::error!("[Error] adjust.blocks.is_none()");
				return None
			}

			let blocks = adjust.blocks.unwrap();

			// calculate delays in each blocks
			for block in blocks.blocks() {

				if block.slot.is_none(){
					log::error!("[Error] block.slot.is_none()");
					return None
				}

				let slot = block.slot.unwrap();
				let block_era = as_number::<B>((slot / ERA_DURATION_IN_SLOTS) as u32);

				if  era_start_slot >=  slot{
					// `slot` should be less than `era_start_slot`, i.e. (slot < start_slot) = true,
					// For when code goes here, `era_start_slot` must be the start slot of an Era
					// and, `slot` belongs to former era

					if block_era < era {
						// TODO: two if `block_era < era` and `slot >= start_slot` could switch places
						// This will only happen when calculating the bigger half of an era,
						// the starting few slot could contain adjust with blocks from last era
						// continue
					}

					let gap = era_start_slot - slot;
					let slot_length = last_slot_length;
					let slot_start_time = start_time - (gap * slot_length) as u128 ;
					let mut delay = 0;

					if block.receive_time > slot_start_time as u128 {
						delay = (block.receive_time - slot_start_time) as i32;
					} else {
						delay = - ((slot_start_time - block.receive_time) as i32);
					}

					log::trace!("block.receive_time {}, slot_start_time {} slot {:?} gap {:?} {}", block.receive_time, slot_start_time, slot, gap, line!());

					sum_block_delay += delay;

				} else {
					let slot_length = this_slot_length;
					let gap = slot - era_start_slot;
					let slot_start_time = start_time + (gap * slot_length) as u128 ;
					let mut delay = 0;

					if block.receive_time > slot_start_time as u128 {
						delay = (block.receive_time - slot_start_time) as i32;
					} else {
						delay = - ((slot_start_time - block.receive_time) as i32);
					}

					log::trace!("block.receive_time {}, slot_start_time {} slot {:?} gap {:?} {}", block.receive_time, slot_start_time, slot, gap, line!());

					sum_block_delay += delay;
				}

				block_number += 1;
			}
			adjust_number += 1;

		} else{
			return None
		}

	}

	if adjust_number != 0 {
		average_adjust_delay = sum_adjust_delay / adjust_number;
	}

	if block_number != 0 {
		average_block_delay = sum_block_delay / block_number;
	}

	Some((average_adjust_delay, average_block_delay))
}

/// Used to generate new new era
#[allow(dead_code)]
pub struct NextEraConfig<B:BlockT> {
	start_slot: u64,
	end_slot: u64,
	pub era: <<B as BlockT>::Header as HeaderT>::Number, // currently useless
	this_slot_length: u64,
	last_slot_length: u64,
	start_time: u128
}

#[allow(dead_code)]
impl<B:BlockT> NextEraConfig <B> {
	pub fn new(
		start_slot: u64,
		end_slot: u64,
		era: <<B as BlockT>::Header as HeaderT>::Number, // currently useless
		this_slot_length: u64,
		last_slot_length: u64,
		start_time: u128
	) -> Self {

		Self{
			start_slot,
			end_slot,
			era,
			this_slot_length,
			last_slot_length,
			start_time
		}
	}
}

/// Used to calculate value of
/// `average_adjust_delay`, `average_block_delay`
/// by recording counts and sum
pub struct AverageDelay{
	adjust_count: i32,
	block_count: i32,

	sum_adjust_delay: i32,
	sum_block_delay: i32,
}
#[allow(dead_code)]
impl AverageDelay {
	pub fn new() -> Self {
		Self{
			adjust_count: 0,
			block_count: 0,
			sum_adjust_delay: 0,
			sum_block_delay: 0,
		}
	}

	/// Input adjust data
	pub fn insert_adjust(&mut self, adjust_sum: i32){
		self.sum_adjust_delay += adjust_sum;
		self.adjust_count += 1;
	}

	/// Input block data
	pub fn insert_block(&mut self, block_sum: i32){
		self.sum_block_delay += block_sum;
		self.block_count += 1;
	}

	/// Input adjust data block data
	pub fn insert_adjust_block(&mut self, adjust_sum: i32, block_sum: i32){
		self.sum_adjust_delay += adjust_sum;
		self.adjust_count += 1;
		self.sum_block_delay += block_sum;
		self.block_count += 1;
	}

	/// Get results
	pub fn average_adjust_block_delay(&self) -> (i32, i32) {

		let mut average_adjust_delay = 0;
		let mut average_block_delay = 0;

		if self.adjust_count != 0 {
			average_adjust_delay = self.sum_adjust_delay / self.adjust_count;
		}

		if self.block_count != 0 {
			average_block_delay = self.sum_block_delay / self.block_count;
		}

		(average_adjust_delay, average_block_delay)
	}
}


/// Crate inner function,
/// transform `u32` into `BlockT::Header::Number`.
pub(crate) fn as_number<B: BlockT>(number: u32) -> <<B as BlockT>::Header as HeaderT>::Number{
	<<B as BlockT>::Header as HeaderT>::Number::from(number)
}

/// Crate inner function,
/// transform `BlockT::Header::Number` into `u32`.
pub(crate) fn into_u32<B: BlockT>(number: <<B as BlockT>::Header as HeaderT>::Number) -> u32{
	let mut result = 0;
	let mut counter = number;
	let one = as_number::<B>(1u32);
	let zero = as_number::<B>(0u32);
	while counter > zero{
		result += 1;
		counter = counter - one;
	}
	result
}



