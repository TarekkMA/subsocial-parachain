#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::{dispatch::DispatchResult, ensure, traits::Get};
use frame_system::ensure_signed;

#[cfg(feature = "std")]
use serde::Deserialize;
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::prelude::*;

use df_traits::moderation::IsAccountBlocked;
use pallet_permissions::SpacePermission;
use pallet_posts::{Module as Posts, PostById};
use pallet_spaces::Module as Spaces;
use pallet_utils::{remove_from_vec, Error as UtilsError, PostId, WhoAndWhen};

pub use pallet::*;
pub mod rpc;

pub type ReactionId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[cfg_attr(feature = "std", serde(untagged))]
pub enum ReactionKind {
    Upvote,
    Downvote,
}

impl Default for ReactionKind {
    fn default() -> Self {
        ReactionKind::Upvote
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Reaction<T: Config> {
    /// Unique sequential identifier of a reaction. Examples of reaction ids: `1`, `2`, `3`,
    /// and so on.
    pub id: ReactionId,

    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub kind: ReactionKind,
}

pub const FIRST_REACTION_ID: u64 = 1;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_utils::Config + pallet_posts::Config + pallet_spaces::Config
    {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let old_pallet_prefix = "ReactionsModule";
            let new_pallet_prefix = Self::name();
            frame_support::log::info!(
                "Move Storage from {} to {}",
                old_pallet_prefix,
                new_pallet_prefix
            );
            frame_support::migration::move_pallet(
                old_pallet_prefix.as_bytes(),
                new_pallet_prefix.as_bytes(),
            );
            T::BlockWeights::get().max_block
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(6, 5))]
        pub fn create_post_reaction(
            origin: OriginFor<T>,
            post_id: PostId,
            kind: ReactionKind,
        ) -> DispatchResult {
            use frame_support::StorageMap;
            let owner = ensure_signed(origin)?;

            let post = &mut Posts::require_post(post_id)?;
            ensure!(
                !<PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
                Error::<T>::AccountAlreadyReacted
            );

            let space = post.get_space()?;
            ensure!(!space.hidden, Error::<T>::CannotReactWhenSpaceHidden);
            ensure!(
                Posts::<T>::is_root_post_visible(post_id)?,
                Error::<T>::CannotReactWhenPostHidden
            );

            ensure!(
                T::IsAccountBlocked::is_allowed_account(owner.clone(), space.id),
                UtilsError::<T>::AccountIsBlocked
            );

            match kind {
                ReactionKind::Upvote => {
                    Spaces::ensure_account_has_space_permission(
                        owner.clone(),
                        &post.get_space()?,
                        SpacePermission::Upvote,
                        Error::<T>::NoPermissionToUpvote.into(),
                    )?;
                    post.inc_upvotes();
                }
                ReactionKind::Downvote => {
                    Spaces::ensure_account_has_space_permission(
                        owner.clone(),
                        &post.get_space()?,
                        SpacePermission::Downvote,
                        Error::<T>::NoPermissionToDownvote.into(),
                    )?;
                    post.inc_downvotes();
                }
            }

            PostById::<T>::insert(post_id, post.clone());
            let reaction_id = Self::insert_new_reaction(owner.clone(), kind);
            ReactionIdsByPostId::<T>::mutate(post.id, |ids| ids.push(reaction_id));
            PostReactionIdByAccount::<T>::insert((owner.clone(), post_id), reaction_id);

            Self::deposit_event(Event::PostReactionCreated(
                owner,
                post_id,
                reaction_id,
                kind,
            ));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 2))]
        pub fn update_post_reaction(
            origin: OriginFor<T>,
            post_id: PostId,
            reaction_id: ReactionId,
            new_kind: ReactionKind,
        ) -> DispatchResult {
            use frame_support::StorageMap;
            let owner = ensure_signed(origin)?;

            ensure!(
                <PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
                Error::<T>::ReactionByAccountNotFound
            );

            let mut reaction = Self::require_reaction(reaction_id)?;
            let post = &mut Posts::require_post(post_id)?;

            ensure!(
                owner == reaction.created.account,
                Error::<T>::NotReactionOwner
            );
            ensure!(reaction.kind != new_kind, Error::<T>::SameReaction);

            if let Some(space_id) = post.try_get_space_id() {
                ensure!(
                    T::IsAccountBlocked::is_allowed_account(owner.clone(), space_id),
                    UtilsError::<T>::AccountIsBlocked
                );
            }

            reaction.kind = new_kind;
            reaction.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

            match new_kind {
                ReactionKind::Upvote => {
                    post.inc_upvotes();
                    post.dec_downvotes();
                }
                ReactionKind::Downvote => {
                    post.inc_downvotes();
                    post.dec_upvotes();
                }
            }

            ReactionById::<T>::insert(reaction_id, reaction);
            PostById::<T>::insert(post_id, post);

            Self::deposit_event(Event::PostReactionUpdated(
                owner,
                post_id,
                reaction_id,
                new_kind,
            ));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(4, 4))]
        pub fn delete_post_reaction(
            origin: OriginFor<T>,
            post_id: PostId,
            reaction_id: ReactionId,
        ) -> DispatchResult {
            use frame_support::StorageMap;
            let owner = ensure_signed(origin)?;

            ensure!(
                <PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
                Error::<T>::ReactionByAccountNotFound
            );

            // TODO extract Self::require_reaction(reaction_id)?;
            let reaction = Self::require_reaction(reaction_id)?;
            let post = &mut Posts::require_post(post_id)?;

            ensure!(
                owner == reaction.created.account,
                Error::<T>::NotReactionOwner
            );
            if let Some(space_id) = post.try_get_space_id() {
                ensure!(
                    T::IsAccountBlocked::is_allowed_account(owner.clone(), space_id),
                    UtilsError::<T>::AccountIsBlocked
                );
            }

            match reaction.kind {
                ReactionKind::Upvote => post.dec_upvotes(),
                ReactionKind::Downvote => post.dec_downvotes(),
            }

            PostById::<T>::insert(post_id, post.clone());
            ReactionById::<T>::remove(reaction_id);
            ReactionIdsByPostId::<T>::mutate(post.id, |ids| remove_from_vec(ids, reaction_id));
            PostReactionIdByAccount::<T>::remove((owner.clone(), post_id));

            Self::deposit_event(Event::PostReactionDeleted(
                owner,
                post_id,
                reaction_id,
                reaction.kind,
            ));
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PostReactionCreated(T::AccountId, PostId, ReactionId, ReactionKind),
        PostReactionUpdated(T::AccountId, PostId, ReactionId, ReactionKind),
        PostReactionDeleted(T::AccountId, PostId, ReactionId, ReactionKind),
    }

    /// Old name generated by `decl_event`.
    #[deprecated(note = "use `Event` instead")]
    pub type RawEvent<T> = Event<T>;

    #[pallet::error]
    pub enum Error<T> {
        /// Reaction was not found by id.
        ReactionNotFound,
        /// Account has already reacted to this post/comment.
        AccountAlreadyReacted,
        /// There is no reaction by account on this post/comment.
        ReactionByAccountNotFound,
        /// Only reaction owner can update their reaction.
        NotReactionOwner,
        /// New reaction kind is the same as old one on this post/comment.
        SameReaction,

        /// Not allowed to react on a post/comment in a hidden space.
        CannotReactWhenSpaceHidden,
        /// Not allowed to react on a post/comment if a root post is hidden.
        CannotReactWhenPostHidden,

        /// User has no permission to upvote posts/comments in this space.
        NoPermissionToUpvote,
        /// User has no permission to downvote posts/comments in this space.
        NoPermissionToDownvote,
    }

    #[pallet::type_value]
    pub fn DefaultForNextReactionId() -> ReactionId {
        FIRST_REACTION_ID
    }

    /// The next reaction id.
    #[pallet::storage]
    #[pallet::getter(fn next_reaction_id)]
    pub type NextReactionId<T: Config> =
        StorageValue<_, ReactionId, ValueQuery, DefaultForNextReactionId>;

    #[pallet::storage]
    #[pallet::getter(fn reaction_by_id)]
    pub type ReactionById<T: Config> = StorageMap<_, Twox64Concat, ReactionId, Reaction<T>>;

    #[pallet::storage]
    #[pallet::getter(fn reaction_ids_by_post_id)]
    pub type ReactionIdsByPostId<T: Config> =
        StorageMap<_, Twox64Concat, PostId, Vec<ReactionId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn post_reaction_id_by_account)]
    pub type PostReactionIdByAccount<T: Config> =
        StorageMap<_, Twox64Concat, (T::AccountId, PostId), ReactionId, ValueQuery>;
}

impl<T: Config> Pallet<T> {
    pub fn insert_new_reaction(account: T::AccountId, kind: ReactionKind) -> ReactionId {
        let id = Self::next_reaction_id();
        let reaction: Reaction<T> = Reaction {
            id,
            created: WhoAndWhen::<T>::new(account),
            updated: None,
            kind,
        };

        ReactionById::<T>::insert(id, reaction);
        NextReactionId::<T>::mutate(|n| {
            *n += 1;
        });

        id
    }

    /// Get `Reaction` by id from the storage or return `ReactionNotFound` error.
    pub fn require_reaction(reaction_id: ReactionId) -> Result<Reaction<T>, DispatchError> {
        Ok(Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?)
    }
}
