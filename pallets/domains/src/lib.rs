#![cfg_attr(not(feature = "std"), no_std)]
///! # Pallet for registering and managing domains.

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_support::traits::{Currency, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use crate::types::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency trait.
		type Currency: ReservableCurrency<<Self as frame_system::Config>::AccountId>;

		/// Maximum amount of domains that can be registered per account.
		#[pallet::constant]
		type MaxDomainsPerAccount: Get<u32>;

		/// The amount held on deposit for registering a domain.
		#[pallet::constant]
		type DomainRegistrationDeposit: Get<BalanceOf<Self>>;

		/// The amount held on deposit per byte for storing record.
		#[pallet::constant]
		type RecordByteDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Stores record set for each domain name.
	#[pallet::storage]
	pub type DomainNameRecords<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		DomainName,
		Twox64Concat,
		RecordKey,
		Record,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
	//
	// #[pallet::hooks]
	// impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	//
	// // Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// // These functions materialize as "extrinsics", which are often compared to transactions.
	// // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	// #[pallet::call]
	// impl<T: Config> Pallet<T> {
	// 	/// An example dispatchable that takes a singles value as a parameter, writes the value to
	// 	/// storage and emits an event. This function must be dispatched by a signed extrinsic.
	// 	#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
	// 	pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResultWithPostInfo {
	// 		// Check that the extrinsic was signed and get the signer.
	// 		// This function will return an error if the extrinsic is not signed.
	// 		// https://docs.substrate.io/v3/runtime/origins
	// 		let who = ensure_signed(origin)?;
	//
	// 		// Update storage.
	// 		<Something<T>>::put(something);
	//
	// 		// Emit an event.
	// 		Self::deposit_event(Event::SomethingStored(something, who));
	// 		// Return a successful DispatchResultWithPostInfo
	// 		Ok(().into())
	// 	}
	//
	// 	/// An example dispatchable that may throw a custom error.
	// 	#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
	// 	pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
	// 		let _who = ensure_signed(origin)?;
	//
	// 		// Read a value from storage.
	// 		match <Something<T>>::get() {
	// 			// Return an error if the value has not been set.
	// 			None => Err(Error::<T>::NoneValue)?,
	// 			Some(old) => {
	// 				// Increment the value read from storage; will error in the event of overflow.
	// 				let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
	// 				// Update the value in storage with the incremented result.
	// 				<Something<T>>::put(new);
	// 				Ok(().into())
	// 			},
	// 		}
	// 	}
	// }
}
