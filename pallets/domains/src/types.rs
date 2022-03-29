use std::convert::TryInto;
use std::thread::sleep;
use frame_support::metadata::StorageEntryModifier::Default;
use frame_support::pallet_prelude::*;
use frame_support::storage::child::len;
use frame_support::traits::{ConstU32, Currency, Len};
use itertools::Itertools;
use super::*;

pub const MAX_DOMAIN_NAME_SIZE: u32 = 64;
pub const MAX_DOMAIN_LABEL_SIZE: u32 = 255;
pub const MAX_LABELS_PER_DOMAIN: u32 = 128;
pub const MAX_RECORD_KEY_SIZE: u32 = 128;
pub const MAX_RECORD_SIZE: u32 = 1024;

pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;

/// A part of the domain name.
///
/// Example: "dev.tarekkma.sub" have 3 labels: dev, tarekkma, sub.
pub(crate) type DomainLabel = BoundedVec<u8, ConstU32<MAX_DOMAIN_LABEL_SIZE>>;

type DomainLabels = BoundedVec<DomainLabel, ConstU32<MAX_LABELS_PER_DOMAIN>>;

/// The key of a record.
pub(crate) type RecordKey = BoundedVec<u8, ConstU32<MAX_RECORD_KEY_SIZE>>;

/// The value of a record.
pub(crate) type Record = BoundedVec<u8, ConstU32<MAX_RECORD_SIZE>>;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct DomainName {
    labels: DomainLabels,
    len: u32,
}

/// The full domain name, that consists of multiple labels.
///
/// Example: "valdp.sub", "wallet.dev.tarek.dot"
impl DomainName {
    /// Creates a new domain name from multiple labels if the total length is less than [MAX_DOMAIN_NAME_SIZE].
    pub fn new(labels: DomainLabels) -> Option<Self> {
        let len = labels.iter().fold(0u32, |cur_len, label| cur_len + label.len() as u32);

        if len > MAX_DOMAIN_NAME_SIZE || len == 0 {
            None
        } else {
            Some(Self {
                labels,
                len,
            })
        }
    }

    /// Retrieves the total length of the domain name.
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn is_tld(&self) -> bool {
        self.labels.len() == 1
    }

    /// Retrieves the TLD.
    pub fn tld(&self) -> &DomainLabel {
        self.labels.last().unwrap()
    }

    /// Retrieves the TLD along with the second level domain.
    pub fn root_domain(&self) -> Option<DomainName> {
        if self.labels.len() < 2 {
            None
        } else {
            let last_2_labels = self.labels.as_slice()[self.labels.len()-2..].to_vec().try_into().ok()?;
            let name = DomainName::new(last_2_labels)?;
            Some(name)
        }
    }

    pub fn create_sub_domain(&self, label: DomainLabel) -> Option<DomainName> {

    }
}

/// Metadata set for a domain name.
pub struct DomainMeta<T: Config> {
    // TODO: rebase on parachain utils.
    // /// When the domain was created.
    // created: WhoAndWhenOf<T>,
    // /// When the domain was updated.
    // updated: Option<WhoAndWhenOf<T>>,

    // Specific block, when the domain will become unavailable.
    pub(super) expires_at: T::BlockNumber,

    // The domain owner.
    pub(super) owner: T::AccountId,

    // The amount was held as a deposit for storing this structure.
    pub(super) domain_deposit: BalanceOf<T>,
    // The amount was held as a deposit for storing outer value.
    pub(super) outer_value_deposit: BalanceOf<T>,
}