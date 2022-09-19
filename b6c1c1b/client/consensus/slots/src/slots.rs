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

use sp_api::{ApiExt, ApiRef, ProvideRuntimeApi};
use sc_client_api::{backend::AuxStore, BlockchainEvents, ProvideUncles, UsageProvider, client::BlockBackend};
use sp_blockchain::{Error as ClientError, HeaderBackend, HeaderMetadata, Result as ClientResult};
use sp_consensus_babe::BabeApi;
use sp_block_builder::BlockBuilder;
use codec::{Decode, Encode};
use sc_network::{protocol::message::{ AdjustTemplate, AdjustExtracts, BlockTemplate}};

use std::time::SystemTime;
use crate::{MILLISECS_PER_BLOCK,
			ERA_DURATION_IN_SLOTS, SLOT_DURATION,
			EPOCH_DURATION_IN_BLOCKS, EPOCH_DURATION_IN_SLOTS
};

use futures_timer::Delay;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
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
					let wait_dur = time_until_next_slot(self.slot_duration);
					Some(Delay::new(wait_dur))
				},
				Some(d) => Some(d),
			};
			log::info!("calculate_current_slot");
			calculate_current_slot(client.clone());

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

}
/// Calculate slot length
pub fn calculate_slot_length<Client, B>(
	_client: Arc<Client>,
) where
	Client: ProvideRuntimeApi<B>
	+ ProvideUncles<B>
	+ BlockchainEvents<B>
	+ AuxStore
	+ UsageProvider<B>
	+ HeaderBackend<B>
	+ HeaderMetadata<B, Error = ClientError>
	+ BlockBackend<B>
	+ Send
	+ Sync
	+ 'static,
	Client::Api: BabeApi<B> + BlockBuilder<B>,
	B: BlockT
{
	// let w1 = 0.3;
	// let w2 = 0.1;
	// //
	// let best_block_number = client.clone().usage_info().chain.best_number;
	// let zero = as_number::<B>(0u32);
	// let one = as_number::<B>(1u32);
	// let two = as_number::<B>(2u32);
	// let epoch_length = as_number::<B>(EPOCH_DURATION_IN_SLOTS as u32); // currently `1 Era = 2 Epoch`
	// let era_length = as_number::<B>(ERA_DURATION_IN_SLOTS as u32); // currently `1 Era = 2 Epoch`
	// //
	// let length: u32 =  into_u32::<B>(target_era);
	// let mut slot_length_set = vec!(0u32; length as usize);
	//
	// // Get genesis slot and time
	// let engine_id = *b"slot";
	// let mut genesis_time: u128 = 0;
	// let mut genesis_slot: u64 = 0;
	// if let Ok(Some(block_one_hash)) = (*client).block_hash(one){
	// 	if let Some(adjust_raw) = (*client).adjusts_raw(engine_id, &BlockId::hash(block_one_hash)){
	// 		match Slot::decode(&mut adjust_raw.as_slice()){
	// 			Ok(a) => {
	// 				genesis_slot = u64::from(a);
	// 				genesis_time = (u64::from(a) as u128) * (SLOT_DURATION as u128);
	// 			},
	// 			Err(e) => {
	// 				log::info!("[Test] Genesis Error {:?}", e);
	// 			},
	// 		};
	// 	} else{
	// 		log::info!("[Test] Genesis Error");
	// 	}
	// };
	//
	// // check and make sure `genesis_time > 0` and `genesis_slot > 0`
	// if genesis_time <= 0 || genesis_slot <= 0{
	// 	return
	// }
	// log::info!("[Test] Genesis Slot {}, Genesis Time {:?}", genesis_slot, genesis_time);
	//
	// //
	// let target_era = best_block_number / era_length;
	// let mut counter = 0;
	// let slot_length_init = SLOT_DURATION as u32;
	// let mut slot_length = slot_length_init;
	//
	// // Enum from 0 to best_block_number with 1 Era at a step
	// // block 0 is excluded for that it does not contain useful adjust information
	// let mut current_era = zero;
	// {
	// 	let mut current_block = zero;
	// 	loop {
	//
	//
	// 		if current_era == zero {
	// 			// At first Era, slot length is the initial slot length
	// 			slot_length = slot_length_init;
	// 			slot_length_set[0] = slot_length_init;
	// 		} else if current_era == one {
	// 			// At second Era, slot length is calculated differently than the following era
	// 			let t_round = slot_length_init;
	// 			let mut t_round_new = t_round;
	// 			let start_slot_number = epoch_length / two;
	// 			let end_slot_number = era_length - epoch_length / two;
	//
	// 			let mut current_time = genesis_time ;
	// 			loop {
	// 				counter += 1;
	//
	// 				log::info!("Block[{:?}], slot", current_block);
	//
	//
	// 				current_block = current_block + one;
	// 			}
	//
	// 		} else {
	// 			// At other Era, slot length need to be calculated
	//
	// 			if slot_length_set[into_u32::<B>(target_era - one) as usize] == 0 {
	// 				log::error!("Error at Calculate Slot length: slot_length_set empty");
	// 				return
	// 			}
	// 			let t_round_1 = slot_length_set[into_u32::<B>(target_era - one) as usize]; // Era 1
	// 			let t_round_2 = slot_length_set[into_u32::<B>(target_era - one - one) as usize]; // Era 0
	// 			let mut t_round_1_new = t_round_1;
	// 			let mut t_round_2_new = t_round_2;
	//
	// 			// Calculate for t_round_1_new
	// 			let start_block_number_1 = (target_era - one - one) * era_length + era_length / two;
	// 			let end_block_number_1 = (target_era - one ) * era_length ;
	// 			let mut current_block = start_block_number_1;
	// 			while current_block < end_block_number_1 {
	// 				counter += 1;
	//
	//
	// 				current_block = current_block + one;
	// 			}
	//
	// 			// Calculate for t_round_2_new
	// 			let start_block_number_2 = (target_era - one ) * era_length ;
	// 			let end_block_number_2 = (target_era - one) * era_length + era_length / two;
	// 			let mut current_block = start_block_number_2;
	// 			while current_block < end_block_number_2 {
	// 				counter += 1;
	//
	//
	//
	// 				current_block = current_block + one;
	// 			}
	//
	// 		}
	//
	// 		current_era = current_era + one;
	// 	}
	// }
	//
	// log::info!("[Test] loop {:?} times", counter);
	// log::info!("[Test] best block hash {:?} from {:?}", (*client).block_hash(best_block_number), best_block_number);

	// let best_number = client.clone().usage_info().chain.best_number;
	//
	// // log::info!("BEFORE extract_block_data");
	// extract_block_data(client, best_number);
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
) where
	Client: ProvideRuntimeApi<B>
	+ ProvideUncles<B>
	+ BlockchainEvents<B>
	+ AuxStore
	+ UsageProvider<B>
	+ HeaderBackend<B>
	+ HeaderMetadata<B, Error = ClientError>
	+ BlockBackend<B>
	+ Send
	+ Sync
	+ 'static,
	Client::Api: BabeApi<B> + BlockBuilder<B>,
	B: BlockT
{
	let w1 = 0.03;
	let w2 = 0.01;
	let w3 = 1.0 - w1 - w2 ;
	//
	let best_block_number = client.clone().usage_info().chain.best_number;
	let zero = as_number::<B>(0u32);
	let one = as_number::<B>(1u32);
	let two = as_number::<B>(2u32);
	let epoch_length = as_number::<B>(EPOCH_DURATION_IN_SLOTS as u32); // currently `1 Era = 2 Epoch`
	let era_length = as_number::<B>(ERA_DURATION_IN_SLOTS as u32); // currently `1 Era = 2 Epoch`
	//
	let target_era = best_block_number / era_length;
	let mut slot_length_set = EraSlot::new(into_u32::<B>(target_era) as usize);

	// Get genesis slot and time
	let engine_id = *b"slot";
	let mut genesis_time: u128 = 0;
	let mut genesis_slot: u64 = 0;
	if let Ok(Some(block_one_hash)) = (*client).block_hash(one){
		if let Some(adjust_raw) = (*client).adjusts_raw(engine_id, &BlockId::hash(block_one_hash)){
			match Slot::decode(&mut adjust_raw.as_slice()){
				Ok(a) => {
					genesis_slot = u64::from(a);
					genesis_time = (u64::from(a) as u128) * (SLOT_DURATION as u128);
				},
				Err(e) => {
					log::info!("[Test] Genesis Error {:?}", e);
				},
			};
		} else{
			log::info!("[Test] Genesis Error");
		}
	};

	// check and make sure `genesis_time > 0` and `genesis_slot > 0`
	if genesis_time <= 0 || genesis_slot <= 0{
		return
	}
	log::info!("[Test] Genesis Slot {}, Genesis Time {:?}", genesis_slot, genesis_time);

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
	log::info!("[Test] before loop now {:?}, slot_length {:?}, current_era {:?},  current_block {:?}, current_slot {:?}, current_time {:?}, counter {:?},",
		now, slot_length, current_era,  current_block, current_slot, current_time, counter,
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
				let start_slot = genesis_slot + EPOCH_DURATION_IN_SLOTS / 2;
				let end_slot = genesis_slot + ERA_DURATION_IN_SLOTS - EPOCH_DURATION_IN_SLOTS / 2;
				//
				let last_slot_length = slot_length_init;
				let this_slot_length = last_slot_length;
				let start_time = genesis_time + (EPOCH_DURATION_IN_SLOTS / 2) as u128 * SLOT_DURATION as u128;

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

						// log::info!("current_block [{}] slot_pointer {:?}", current_block, slot_pointer);
						// log::info!("start_slot {:?} end_slot {:?} this_slot_length {} start_time {}", start_slot, end_slot, this_slot_length, start_time);

						let res = deal_adjusts(adjusts, start_slot, end_slot, zero, this_slot_length, last_slot_length, start_time);

						if let Some((adjust_delay, block_delay)) = res {
							// log::info!("Block [{}] (a,b) = {:?}", current_block, res);
							delay.insert_adjust_block(adjust_delay, block_delay);
						}

					} else {
						log::error!("[Error] Block [{}] not found", current_block)
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

				log::info!("Era 1 slot length {}*{} + {}*{} + {}*{} = {}",
					w3, SLOT_DURATION, w2, average_adjust_delay, w1, average_block_delay, era_1_slot_length
				);

				// Calculated results
				slot_length = era_1_slot_length ;

				// Record results
				slot_length_set.set_value(1, slot_length);

				// Mark current time, until Era 1 end
				current_time += (slot_length as u128) * (ERA_DURATION_IN_SLOTS as u128);

				// Mark current Era, until Era 1 end
				current_slot += ERA_DURATION_IN_SLOTS;

			} else {
				// At Era n, slot length need to be calculated
				// log::info!(" Calculate at Era n");
				// Slot interval used to calculate new slot length
				let start_slot = current_slot - ERA_DURATION_IN_SLOTS - EPOCH_DURATION_IN_SLOTS / 2;
				let end_slot = current_slot - EPOCH_DURATION_IN_SLOTS / 2;

				//
				let last_slot_length = slot_length_set.era_slot_length(into_u32::<B>(current_era) as usize - 2);
				let this_slot_length = slot_length_set.era_slot_length(into_u32::<B>(current_era) as usize - 1);
				let start_time = current_time - (ERA_DURATION_IN_SLOTS * this_slot_length) as u128
									- (EPOCH_DURATION_IN_SLOTS / 2 * last_slot_length) as u128;

				let default_exit = counter + 2 * EPOCH_DURATION_IN_SLOTS;
				let mut slot_pointer = start_slot;
				let mut delay = AverageDelay::new();

				loop {

					if let Some(adjusts) = extract_block_data(client.clone(), current_block){
						// log::info!("current_block [{}] ", current_block);
						if adjusts.biggest_slot().is_none() {
							counter += 1;
							current_block = current_block + one;
							continue
						}

						slot_pointer = adjusts.biggest_slot().unwrap();

						// log::info!("current_block [{}] slot_pointer {:?}", current_block, slot_pointer);
						// log::info!("start_slot {:?} end_slot {:?} this_slot_length {} start_time {}", start_slot, end_slot, this_slot_length, start_time);

						let res = deal_adjusts(adjusts, start_slot, end_slot, current_era - one, this_slot_length, last_slot_length, start_time);

						if let Some((adjust_delay, block_delay)) = res {
							// log::info!("Block [{}] (a,b) = {:?}", current_block, res);
							delay.insert_adjust_block(adjust_delay, block_delay);
						}

					} else {
						log::error!("[Error] Block [{}] not found", current_block)
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

				log::info!("Era {} slot length {}*{} + {}*{} + {}*{} = {}",
					current_era, w3, this_slot_length, w2, average_adjust_delay, w1, average_block_delay, era_n_slot_length
				);

				// Calculated results
				slot_length = era_n_slot_length ;

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
	log::info!("[Test] after loop now {:?}, slot_length {:?}, current_era {:?},  current_block {:?}, current_slot {:?}, current_time {:?}, counter {:?},",
		now, slot_length, current_era,  current_block, current_slot, current_time, counter,
	);

	log::info!("[Test] loop {:?} times", counter);
	log::info!("[Test] best block hash {:?} from {:?}", (*client).block_hash(best_block_number), best_block_number);

	let best_number = client.clone().usage_info().chain.best_number;

	// log::info!("BEFORE extract_block_data");
	extract_block_data(client, best_number);
}

pub(crate) fn extract_block_data<Client, B>(client: Arc<Client>,  number: <<B as BlockT>::Header as HeaderT>::Number)
	-> Option<AdjustExtracts<B>>
where
	Client: ProvideRuntimeApi<B>
	+ ProvideUncles<B>
	+ BlockchainEvents<B>
	+ AuxStore
	+ UsageProvider<B>
	+ HeaderBackend<B>
	+ HeaderMetadata<B, Error = ClientError>
	+ BlockBackend<B>
	+ Send
	+ Sync
	+ 'static,
	Client::Api: BabeApi<B> + BlockBuilder<B>,
	B: BlockT,
 {
	let engine_id = *b"ajst";
	let adjust = if let Ok(Some(hash)) = (*client).block_hash(number){
		if let Some(adjust_raw) = (*client).adjusts_raw(engine_id, &BlockId::hash(hash)){
			match AdjustExtracts::<B>::decode(&mut adjust_raw.as_slice()){
				Ok(a) => {
					// log::info!("[Test] On chain {:?} adjust_raw contains {:?}", best_hash, a.len());
					// log::info!("[Test] extract_block_data adjust_raw {:#?}", a);
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

/// Calculate `average_adjust_delay`, `average_block_delay` between two given slot.
/// An AdjustExtracts contain multiple Adjusts.
/// An Adjust contains multiple Blocks,
/// `average_adjust_delay` is calculated from multiple Adjusts,
/// `average_block_delay` is calculated from multiple Blocks.
/// Option<(i32, i32)> => Option<(average_adjust_delay, average_block_delay)>.
pub fn deal_adjusts<B:BlockT>(
	adjusts: AdjustExtracts<B>,
	start_slot: u64,
	end_slot: u64,
	era: <<B as BlockT>::Header as HeaderT>::Number, // currently useless
	this_slot_length: u64,
	last_slot_length: u64,
	start_time: u128
) -> Option<(i32, i32)>{
	let mut average_adjust_delay: i32 = 0 ;
	let mut average_block_delay: i32 = 0 ;

	if start_slot > end_slot {
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

		if start_slot <= slot && slot < end_slot {

			// calculate adjust delay
			let delay = if adjust.send_time > adjust.receive_time {
				(adjust.send_time - adjust.receive_time) as i32
			} else {
				- ((adjust.receive_time - adjust.send_time) as i32)
			};

			if delay > 6000 || delay < -6000{
				log::info!("{}, delay {}", line!(), delay);
				log::info!("adjust.receive_time {}, adjust.receive_time {} slot {:?} ", adjust.receive_time, adjust.receive_time, slot)
			}

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

				if  slot >= start_slot {
					// `slot` should be less than `start_slot`, i.e. (slot < start_slot) = true,
					// For when code goes here, start_slot must be the start slot of an Era
					// and, `slot` belongs to last era

					if block_era < era {
						// TODO: two if `block_era < era` and `slot >= start_slot` could switch places
						// This will only happen when calculating the bigger half of an era,
						// the starting few slot could contain adjust with blocks from last era
						// continue
					}

					let gap = start_slot - slot;
					let slot_length = last_slot_length;
					let slot_start_time = start_time - (gap * slot_length) as u128 ;
					let mut delay = 0;

					if block.receive_time > slot_start_time as u128 {
						delay = (block.receive_time - slot_start_time) as i32;
					} else {
						delay = - ((slot_start_time - block.receive_time) as i32);
					}

					if delay > 6000 || delay < -6000{
						log::info!("{}, delay {}", line!(), delay);
						log::info!("block.receive_time {}, slot_start_time {} slot {:?} gap {:?}", block.receive_time, slot_start_time, slot, gap)
					}

					sum_block_delay += delay;

				} else {
					let slot_length = this_slot_length;
					let gap = end_slot - slot;
					let slot_start_time = start_time + (gap * slot_length) as u128 ;
					let mut delay = 0;

					if block.receive_time > slot_start_time as u128 {
						delay = (block.receive_time - slot_start_time) as i32;
					} else {
						delay = - ((slot_start_time - block.receive_time) as i32);
					}

					if delay > 6000 || delay < -6000{
						log::info!("{}, delay {}", line!(), delay);
						log::info!("block.receive_time {}, slot_start_time {} slot {:?} gap {:?}", block.receive_time, slot_start_time, slot, gap)
					}

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



