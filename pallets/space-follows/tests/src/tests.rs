use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use sp_runtime::traits::Zero;

use pallet_parachain_utils::Error as UtilsError;
use pallet_spaces::Error as SpacesError;
use pallet_permissions::SpacePermission as SP;
use pallet_parachain_utils::mock_functions::*;
use pallet_space_follows::Error as SpaceFollowsError;

use crate::mock::*;
use crate::tests_utils::*;

#[test]
fn follow_space_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_default_follow_space()); // Follow SpaceId 1 by ACCOUNT2

        assert_eq!(Spaces::space_by_id(SPACE1).unwrap().followers_count, 2);
        assert_eq!(
            SpaceFollows::spaces_followed_by_account(ACCOUNT2),
            vec![SPACE1]
        );
        assert_eq!(
            SpaceFollows::space_followers(SPACE1),
            vec![ACCOUNT1, ACCOUNT2]
        );
        assert_eq!(
            SpaceFollows::space_followed_by_account((ACCOUNT2, SPACE1)),
            true
        );
    });
}

#[test]
fn follow_space_should_fail_when_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _default_follow_space(),
            SpacesError::<Test>::SpaceNotFound
        );
    });
}

#[test]
fn follow_space_should_fail_when_account_is_already_space_follower() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_default_follow_space()); // Follow SpaceId 1 by ACCOUNT2

        assert_noop!(
            _default_follow_space(),
            SpaceFollowsError::<Test>::AlreadySpaceFollower
        );
    });
}

#[test]
fn follow_space_should_fail_when_trying_to_follow_hidden_space() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_update_space(
            None,
            None,
            Some(space_update(None, None, Some(true)))
        ));

        assert_noop!(
            _default_follow_space(),
            SpaceFollowsError::<Test>::CannotFollowHiddenSpace
        );
    });
}

#[test]
fn unfollow_space_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_default_follow_space());
        // Follow SpaceId 1 by ACCOUNT2
        assert_ok!(_default_unfollow_space());

        assert_eq!(Spaces::space_by_id(SPACE1).unwrap().followers_count, 1);
        assert!(SpaceFollows::spaces_followed_by_account(ACCOUNT2).is_empty());
        assert_eq!(SpaceFollows::space_followers(SPACE1), vec![ACCOUNT1]);
    });
}

#[test]
fn unfollow_space_should_fail_when_space_not_found() {
    ExtBuilder::build_with_space_follow_no_space().execute_with(|| {
        assert_noop!(
            _default_unfollow_space(),
            SpacesError::<Test>::SpaceNotFound
        );
    });
}

#[test]
fn unfollow_space_should_fail_when_account_is_not_space_follower_yet() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _default_unfollow_space(),
            SpaceFollowsError::<Test>::NotSpaceFollower
        );
    });
}
