#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::RawOrigin, pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Zero;
    use pallet_proxy::WeightInfo;

    type BalanceOf<T> = <<T as pallet_proxy::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_proxy::Config {
        type ProxyDepositBase: Get<BalanceOf<Self>>;

        type ProxyDepositFactor: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        OnlyFirstProxyCanBeFree,
    }

    #[pallet::storage]
    #[pallet::getter(fn is_free_proxy)]
    pub type FreeProxyFlag<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(< T as pallet_proxy::Config >::WeightInfo::add_proxy(T::MaxProxies::get()))]
        pub fn add_free_proxy(
            origin: OriginFor<T>,
            delegate: T::AccountId,
            proxy_type: T::ProxyType,
            delay: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let proxy_count = pallet_proxy::Proxies::<T>::get(&who).0.len();

            ensure!(proxy_count == 0, Error::<T>::OnlyFirstProxyCanBeFree);

            FreeProxyFlag::<T>::set(true);

            let add_proxy_res = pallet_proxy::Pallet::<T>::add_proxy(
                RawOrigin::Signed(who).into(),
                delegate,
                proxy_type,
                delay,
            );

            FreeProxyFlag::<T>::kill();

            add_proxy_res
        }
    }

    pub struct AdjustedProxyDepositBase<T>(PhantomData<T>);
    impl<T: Config> Get<BalanceOf<T>> for AdjustedProxyDepositBase<T> {
        fn get() -> BalanceOf<T> {
            if FreeProxyFlag::<T>::get() {
                Zero::zero()
            } else {
                <T as Config>::ProxyDepositBase::get()
            }
        }
    }

    pub struct AdjustedProxyDepositFactor<T>(PhantomData<T>);
    impl<T: Config> Get<BalanceOf<T>> for AdjustedProxyDepositFactor<T> {
        fn get() -> BalanceOf<T> {
            if FreeProxyFlag::<T>::get() {
                Zero::zero()
            } else {
                <T as Config>::ProxyDepositFactor::get()
            }
        }
    }
}
