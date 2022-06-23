//! # Spaces Module
//!
//! Spaces are the primary components of Subsocial. This module allows you to create a Space
//! and customize it by updating its' owner(s), content, unique handle, and permissions.
//!
//! To understand how Spaces fit into the Subsocial ecosystem, you can think of how
//! folders and files work in a file system. Spaces are similar to folders, that can contain Posts,
//! in this sense. The permissions of the Space and Posts can be customized so that a Space
//! could be as simple as a personal blog (think of a page on Facebook) or as complex as community
//! (think of a subreddit) governed DAO.
//!
//! Spaces can be compared to existing entities on web 2.0 platforms such as:
//!
//! - Blogs on Blogger,
//! - Publications on Medium,
//! - Groups or pages on Facebook,
//! - Accounts on Twitter and Instagram,
//! - Channels on YouTube,
//! - Servers on Discord,
//! - Forums on Discourse.

#![cfg_attr(not(feature = "std"), no_std)]

// pub mod rpc;
pub mod types;

pub use pallet::*;

use df_traits::SpaceFollowsProvider;
use pallet_permissions::{SpacePermission, SpacePermissions};
use pallet_parachain_utils::{Content, SpaceId, new_who_and_when};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use types::*;

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use df_traits::{
        moderation::{IsAccountBlocked, IsContentBlocked},
        PermissionChecker, SpaceForRoles, SpaceForRolesProvider,
    };
    use pallet_permissions::{Pallet as Permissions, SpacePermissionsContext};
    use pallet_parachain_utils::{
        Error as UtilsError, ensure_content_is_valid, throw_utils_error,
    };

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_permissions::Config + pallet_timestamp::Config
    {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Roles: PermissionChecker<AccountId = Self::AccountId>;

        type SpaceFollows: SpaceFollowsProvider<AccountId = Self::AccountId>;

        type BeforeSpaceCreated: BeforeSpaceCreated<Self>;

        type AfterSpaceUpdated: AfterSpaceUpdated<Self>;

        type IsAccountBlocked: IsAccountBlocked<Self::AccountId>;

        type IsContentBlocked: IsContentBlocked;

        #[pallet::constant]
        type MaxHandleLen: Get<u32>;

        #[pallet::constant]
        type MaxSpacesPerAccount: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SpaceCreated(T::AccountId, SpaceId),
        SpaceUpdated(T::AccountId, SpaceId),
        SpaceDeleted(T::AccountId, SpaceId),
    }

    // TODO_REMOVE_IF_NO_EVENT
    /// Old name generated by `decl_event`.
    #[deprecated(note = "use `Event` instead")]
    pub type RawEvent<T> = Event<T>;

    #[pallet::error]
    pub enum Error<T> {
        /// Space was not found by id.
        SpaceNotFound,
        /// Space handle is not unique.
        SpaceHandleIsNotUnique,
        /// Handles are disabled in `PalletSettings`.
        HandlesAreDisabled,
        /// Nothing to update in this space.
        NoUpdatesForSpace,
        /// Only space owners can manage this space.
        NotASpaceOwner,
        /// User has no permission to update this space.
        NoPermissionToUpdateSpace,
        /// User has no permission to create subspaces within this space.
        NoPermissionToCreateSubspaces,
        /// Space is at root level, no `parent_id` specified.
        SpaceIsAtRoot,
        /// New spaces' settings don't differ from the old ones.
        NoUpdatesForSpacesSettings,
        /// There are too many spaces created by this account already
        TooManySpacesPerAccount,
    }

    #[pallet::type_value]
    pub fn DefaultForNextSpaceId() -> SpaceId {
        RESERVED_SPACE_COUNT + 1
    }

    /// The next space id.
    #[pallet::storage]
    #[pallet::getter(fn next_space_id)]
    pub type NextSpaceId<T: Config> = StorageValue<_, SpaceId, ValueQuery, DefaultForNextSpaceId>;

    /// Get the details of a space by its' id.
    #[pallet::storage]
    #[pallet::getter(fn space_by_id)]
    pub type SpaceById<T: Config> = StorageMap<_, Twox64Concat, SpaceId, Space<T>>;

    /// Find a given space id by its' unique handle.
    /// If a handle is not registered, nothing will be returned (`None`).
    #[pallet::storage]
    #[pallet::getter(fn space_id_by_handle)]
    pub type SpaceIdByHandle<T: Config> = StorageMap<_, Blake2_128Concat, Handle<T>, SpaceId>;

    /// Find the ids of all spaces owned, by a given account.
    #[pallet::storage]
    #[pallet::getter(fn space_ids_by_owner)]
    pub type SpaceIdsByOwner<T: Config> =
    StorageMap<_, Twox64Concat, T::AccountId, SpacesByAccount<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn settings)]
    pub type PalletSettings<T: Config> = StorageValue<_, SpacesSettings, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub endowed_account: Option<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                endowed_account: None,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            if let Some(endowed_account) = self.endowed_account.clone() {
                let mut spaces = Vec::new();

                for id in FIRST_SPACE_ID..=RESERVED_SPACE_COUNT {
                    spaces.push((
                        id,
                        Space::<T>::new(id, None, endowed_account.clone(), Content::None, None),
                    ));
                }
                spaces.iter().for_each(|(k, v)| {
                    SpaceById::<T>::insert(k, v);
                });
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(500_000 + T::DbWeight::get().reads_writes(5, 4))]
        pub fn create_space(
            origin: OriginFor<T>,
            parent_id_opt: Option<SpaceId>,
            // FIXME: unused since domains release
            handle_opt: Option<Vec<u8>>,
            content: Content,
            permissions_opt: Option<SpacePermissions>,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;

            ensure!(handle_opt.is_some(), Error::<T>::HandlesAreDisabled);

            ensure_content_is_valid(content.clone())?;

            Self::ensure_space_limit_not_reached(&owner)?;

            // TODO: add tests for this case
            if let Some(parent_id) = parent_id_opt {
                let parent_space = Self::require_space(parent_id)?;

                ensure!(
                    T::IsAccountBlocked::is_allowed_account(owner.clone(), parent_id),
                    throw_utils_error(UtilsError::AccountIsBlocked)
                );
                ensure!(
                    T::IsContentBlocked::is_allowed_content(content.clone(), parent_id),
                    throw_utils_error(UtilsError::ContentIsBlocked)
                );

                Self::ensure_account_has_space_permission(
                    owner.clone(),
                    &parent_space,
                    SpacePermission::CreateSubspaces,
                    Error::<T>::NoPermissionToCreateSubspaces.into(),
                )?;
            }

            let permissions =
                permissions_opt.map(|perms| Permissions::<T>::override_permissions(perms));

            let space_id = Self::next_space_id();
            let new_space = &mut Space::new(
                space_id,
                parent_id_opt,
                owner.clone(),
                content,
                permissions,
            );

            // FIXME: What's about handle reservation if this fails?
            T::BeforeSpaceCreated::before_space_created(owner.clone(), new_space)?;

            SpaceById::<T>::insert(space_id, new_space);
            SpaceIdsByOwner::<T>::mutate(
                &owner, |ids| {
                    ids.try_push(space_id).expect("qed; too many spaces per account")
                }
            );
            NextSpaceId::<T>::mutate(|n| *n += 1);

            Self::deposit_event(Event::SpaceCreated(owner, space_id));
            Ok(().into())
        }

        #[pallet::weight(500_000 + T::DbWeight::get().reads_writes(3, 3))]
        pub fn update_space(
            origin: OriginFor<T>,
            space_id: SpaceId,
            update: SpaceUpdate,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            let has_updates = update.parent_id.is_some()
                || update.handle.is_some()
                || update.content.is_some()
                || update.hidden.is_some()
                || update.permissions.is_some();

            ensure!(has_updates, Error::<T>::NoUpdatesForSpace);

            let mut space = Self::require_space(space_id)?;

            ensure!(
                T::IsAccountBlocked::is_allowed_account(owner.clone(), space.id),
                throw_utils_error(UtilsError::AccountIsBlocked)
            );

            Self::ensure_account_has_space_permission(
                owner.clone(),
                &space,
                SpacePermission::UpdateSpace,
                Error::<T>::NoPermissionToUpdateSpace.into(),
            )?;

            let mut is_update_applied = false;
            let mut old_data = SpaceUpdate::default();

            // TODO: add tests for this case
            if let Some(parent_id_opt) = update.parent_id {
                if parent_id_opt != space.parent_id {
                    if let Some(parent_id) = parent_id_opt {
                        let parent_space = Self::require_space(parent_id)?;

                        Self::ensure_account_has_space_permission(
                            owner.clone(),
                            &parent_space,
                            SpacePermission::CreateSubspaces,
                            Error::<T>::NoPermissionToCreateSubspaces.into(),
                        )?;
                    }

                    old_data.parent_id = Some(space.parent_id);
                    space.parent_id = parent_id_opt;
                    is_update_applied = true;
                }
            }

            if let Some(content) = update.content {
                if content != space.content {
                    ensure_content_is_valid(content.clone())?;

                    ensure!(
                        T::IsContentBlocked::is_allowed_content(content.clone(), space.id),
                        throw_utils_error(UtilsError::ContentIsBlocked)
                    );
                    if let Some(parent_id) = space.parent_id {
                        ensure!(
                            T::IsContentBlocked::is_allowed_content(content.clone(), parent_id),
                            throw_utils_error(UtilsError::ContentIsBlocked)
                        );
                    }

                    old_data.content = Some(space.content);
                    space.content = content;
                    is_update_applied = true;
                }
            }

            if let Some(hidden) = update.hidden {
                if hidden != space.hidden {
                    old_data.hidden = Some(space.hidden);
                    space.hidden = hidden;
                    is_update_applied = true;
                }
            }

            if let Some(overrides_opt) = update.permissions {
                if space.permissions != overrides_opt {
                    old_data.permissions = Some(space.permissions);

                    if let Some(overrides) = overrides_opt.clone() {
                        space.permissions = Some(Permissions::<T>::override_permissions(overrides));
                    } else {
                        space.permissions = overrides_opt;
                    }

                    is_update_applied = true;
                }
            }

            // Update this space only if at least one field should be updated:
            if is_update_applied {
                space.updated = Some(new_who_and_when::<T>(owner.clone()));

                SpaceById::<T>::insert(space_id, space.clone());
                T::AfterSpaceUpdated::after_space_updated(owner.clone(), &space, old_data);

                Self::deposit_event(Event::SpaceUpdated(owner, space_id));
            }
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn update_settings(
            origin: OriginFor<T>,
            new_settings: SpacesSettings,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let space_settings = Self::settings();
            ensure!(
                space_settings != new_settings,
                Error::<T>::NoUpdatesForSpacesSettings
            );

            PalletSettings::<T>::mutate(|settings| *settings = new_settings);

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Check that there is a `Space` with such `space_id` in the storage
        /// or return`SpaceNotFound` error.
        pub fn ensure_space_exists(space_id: SpaceId) -> DispatchResult {
            ensure!(
                <SpaceById<T>>::contains_key(space_id),
                Error::<T>::SpaceNotFound
            );
            Ok(())
        }

        /// Get `Space` by id from the storage or return `SpaceNotFound` error.
        pub fn require_space(space_id: SpaceId) -> Result<Space<T>, DispatchError> {
            Ok(Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?)
        }

        pub fn ensure_account_has_space_permission(
            account: T::AccountId,
            space: &Space<T>,
            permission: SpacePermission,
            error: DispatchError,
        ) -> DispatchResult {
            let is_owner = space.is_owner(&account);
            let is_follower = space.is_follower(&account);

            let ctx = SpacePermissionsContext {
                space_id: space.id,
                is_space_owner: is_owner,
                is_space_follower: is_follower,
                space_perms: space.permissions.clone(),
            };

            T::Roles::ensure_account_has_space_permission(account, ctx, permission, error)
        }

        pub fn ensure_handles_enabled() -> DispatchResult {
            ensure!(
                Self::settings().handles_enabled,
                Error::<T>::HandlesAreDisabled
            );
            Ok(())
        }

        pub fn try_move_space_to_root(space_id: SpaceId) -> DispatchResult {
            let mut space = Self::require_space(space_id)?;
            space.parent_id = None;

            SpaceById::<T>::insert(space_id, space);
            Ok(())
        }

        pub fn mutate_space_by_id<F: FnOnce(&mut Space<T>)>(
            space_id: SpaceId,
            f: F,
        ) -> Result<Space<T>, DispatchError> {
            <SpaceById<T>>::mutate(space_id, |space_opt| {
                if let Some(ref mut space) = space_opt.clone() {
                    f(space);
                    *space_opt = Some(space.clone());

                    return Ok(space.clone());
                }

                Err(Error::<T>::SpaceNotFound.into())
            })
        }

        pub fn ensure_space_limit_not_reached(owner: &T::AccountId) -> DispatchResult {
            ensure!(
                Self::space_ids_by_owner(&owner).len() < T::MaxSpacesPerAccount::get() as usize,
                Error::<T>::TooManySpacesPerAccount,
            );
            Ok(())
        }
    }

    impl<T: Config> SpaceForRolesProvider for Pallet<T> {
        type AccountId = T::AccountId;

        fn get_space(id: SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError> {
            let space = Pallet::<T>::require_space(id)?;

            Ok(SpaceForRoles {
                owner: space.owner,
                permissions: space.permissions,
            })
        }
    }

    pub trait BeforeSpaceCreated<T: Config> {
        fn before_space_created(follower: T::AccountId, space: &mut Space<T>) -> DispatchResult;
    }

    impl<T: Config> BeforeSpaceCreated<T> for () {
        fn before_space_created(_follower: T::AccountId, _space: &mut Space<T>) -> DispatchResult {
            Ok(())
        }
    }

    #[impl_trait_for_tuples::impl_for_tuples(10)]
    pub trait AfterSpaceUpdated<T: Config> {
        fn after_space_updated(sender: T::AccountId, space: &Space<T>, old_data: SpaceUpdate);
    }
}