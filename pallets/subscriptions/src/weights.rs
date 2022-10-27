//! Autogenerated weights for pallet_subscriptions
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-27, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./scripts/../target/release/subsocial-collator
// benchmark
// pallet
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// Compiled
// --pallet
// pallet_subscriptions
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --heap-pages
// 4096
// --output
// ./pallets/subscriptions/src/weights.rs
// --template
// ./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{constants::RocksDbWeight, Weight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_subscriptions.
pub trait WeightInfo {
    fn update_subscription_settings() -> Weight;
    fn subscribe() -> Weight;
    fn unsubscribe() -> Weight;
}

/// Weights for pallet_subscriptions using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    // Storage: Spaces SpaceById (r:1 w:0)
    // Storage: Roles RoleById (r:1 w:0)
    // Storage: Subscriptions SubscriptionSettingsBySpace (r:0 w:1)
    fn update_subscription_settings() -> Weight {
        (18_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Subscriptions SpaceSubscribers (r:1 w:1)
    // Storage: Subscriptions SubscriptionSettingsBySpace (r:1 w:0)
    // Storage: Spaces SpaceById (r:1 w:0)
    // Storage: System Account (r:2 w:2)
    // Storage: Roles RoleById (r:1 w:0)
    // Storage: Roles UsersByRoleId (r:1 w:1)
    // Storage: Roles RoleIdsByUserInSpace (r:1 w:1)
    fn subscribe() -> Weight {
        (56_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: Subscriptions SpaceSubscribers (r:1 w:1)
    fn unsubscribe() -> Weight {
        (13_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    // Storage: Spaces SpaceById (r:1 w:0)
    // Storage: Roles RoleById (r:1 w:0)
    // Storage: Subscriptions SubscriptionSettingsBySpace (r:0 w:1)
    fn update_subscription_settings() -> Weight {
        (18_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Subscriptions SpaceSubscribers (r:1 w:1)
    // Storage: Subscriptions SubscriptionSettingsBySpace (r:1 w:0)
    // Storage: Spaces SpaceById (r:1 w:0)
    // Storage: System Account (r:2 w:2)
    // Storage: Roles RoleById (r:1 w:0)
    // Storage: Roles UsersByRoleId (r:1 w:1)
    // Storage: Roles RoleIdsByUserInSpace (r:1 w:1)
    fn subscribe() -> Weight {
        (56_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(8 as Weight))
            .saturating_add(RocksDbWeight::get().writes(5 as Weight))
    }
    // Storage: Subscriptions SpaceSubscribers (r:1 w:1)
    fn unsubscribe() -> Weight {
        (13_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
}