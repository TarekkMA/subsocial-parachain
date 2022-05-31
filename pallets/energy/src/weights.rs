//! Autogenerated weights for pallet_energy
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-31, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    // pallet_energy
    // --extrinsic
    // *
    // --steps
    // 50
    // --repeat
    // 20
    // --heap-pages
    // 4096
    // --output
    // ./pallets/energy/src/weights.rs
    // --template
    // ./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_energy.
pub trait WeightInfo {
    fn update_conversion_ratio() -> Weight;
    fn generate_energy() -> Weight;
}

/// Weights for pallet_energy using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
            // Storage: Energy ConversionRatio (r:0 w:1)
        fn update_conversion_ratio() -> Weight {
        (23_000_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
            // Storage: System Account (r:1 w:1)
            // Storage: Energy ConversionRatio (r:1 w:0)
            // Storage: Energy TotalEnergy (r:1 w:1)
            // Storage: Energy EnergyBalance (r:1 w:1)
        fn generate_energy() -> Weight {
        (72_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: Energy ConversionRatio (r:0 w:1)
        fn update_conversion_ratio() -> Weight {
        (23_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
            // Storage: System Account (r:1 w:1)
            // Storage: Energy ConversionRatio (r:1 w:0)
            // Storage: Energy TotalEnergy (r:1 w:1)
            // Storage: Energy EnergyBalance (r:1 w:1)
        fn generate_energy() -> Weight {
        (72_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
    }
