use scrypto::prelude::*;

/// The contents of a lock.
#[derive(ScryptoSbor, Clone, Eq, PartialEq, Debug)]
pub enum LockContents {
    Fungible(Decimal),
    NonFungible(IndexSet<NonFungibleLocalId>),
}

/// Non-fungible data for a lock receipt.
#[derive(ScryptoSbor, NonFungibleData, Clone, Eq, PartialEq, Debug)]
pub struct LockReceipt {
    pub name: String,
    pub description: String,
    pub key_image_url: Url,
    pub resource: ResourceAddress,
    pub locked_contents: LockContents,
    pub locked_at: Instant,
    pub unlockable_at: Option<Instant>,
    #[mutable] pub unlocked_at: Option<Instant>,
}

/// Event emitted when an item is locked.
#[derive(ScryptoSbor, ScryptoEvent, Clone, Eq, PartialEq, Debug)]
pub struct EventLock {
    pub lock_id: NonFungibleLocalId,
    pub resource: ResourceAddress,
    pub locked_contents: LockContents,
    pub locked_at: Instant,
    pub unlockable_at: Option<Instant>,
}

/// Event emitted when an item is unlocked.
#[derive(ScryptoSbor, ScryptoEvent, Clone, Eq, PartialEq, Debug)]
pub struct EventUnlock {
    pub lock_id: NonFungibleLocalId,
    pub resource: ResourceAddress,
    pub locked_contents: LockContents,
    pub locked_at: Instant,
    pub unlockable_at: Option<Instant>,
    pub unlocked_at: Instant,
}

#[blueprint]
#[types(
    NonFungibleLocalId,
    Vault,
)]
#[events(
    EventLock,
    EventUnlock,
)]
mod locker_mod {
    struct Locker {
        lock_receipt_manager: ResourceManager,
        counter: u64,
        vaults: KeyValueStore<NonFungibleLocalId, Vault>,
        used_lock_receipts: Vault,
    }

    impl Locker {
        /// Creates a new locker.
        /// 
        /// # Arguments
        /// 
        /// * `owner_resource` - The owner role resource used to update metadata.
        /// 
        /// # Returns
        /// 
        /// A new locker.
        /// 
        pub fn new(
            owner_resource: ResourceAddress,
            info_url: Url,
            icon_url: Url,
        ) -> Global<Locker> {
            // Assign component address
            let (address_reservation, this) = Runtime::allocate_component_address(Locker::blueprint_id());

            // Construct owner role used to update metadata
            let owner_role = OwnerRole::Updatable(rule!(require(owner_resource)));

            // Create the lock receipt manager
            let lock_receipt_manager = ResourceBuilder::new_integer_non_fungible::<LockReceipt>(owner_role.clone())
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(this)));
                    minter_updater => rule!(deny_all);
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater => rule!(require(global_caller(this)));
                    non_fungible_data_updater_updater => rule!(deny_all);
                })
                .metadata(metadata! {
                    init {
                        "name" => "LockReceipt", updatable;
                        "description" => "Don't trust, verify.", updatable;
                        "info_url" => info_url, updatable;
                        "icon_url" => icon_url, updatable;
                        "locker" => this, locked;
                    }
                })
                .create_with_no_initial_supply();
            
            // Instantiate the locker
            Self {
                lock_receipt_manager,
                counter: 0,
                vaults: KeyValueStore::new_with_registered_type(),
                used_lock_receipts: Vault::new(lock_receipt_manager.address()),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .with_address(address_reservation)
            .globalize()
        }

        /// Locks an item and returns a lock receipt.
        /// 
        /// # Arguments
        /// 
        /// * `item` - The item to lock.
        /// * `unlockable_at` - The time at which the item can be unlocked.
        /// 
        /// # Returns
        /// 
        /// A bucket containing the lock receipt.
        /// 
        /// # Emits
        /// 
        /// * `EventLock` - An event emitted when a item is locked.
        /// 
        pub fn lock(
            &mut self,
            item: Bucket,
            unlockable_at: Option<Instant>,
            name: String,
            description: String,
            key_image_url: Url,
        ) -> Bucket {
            // Get the current time
            let current_time = Clock::current_time(TimePrecisionV2::Second);

            // Get the next id and increment the counter
            let id = NonFungibleLocalId::integer(self.counter);
            self.counter += 1;
            
            // Get the resource address and the bucket contents
            let resource = item.resource_address();
            let locked_contents = match item.resource_manager().resource_type() {
                ResourceType::Fungible{..} => LockContents::Fungible(item.amount()),
                ResourceType::NonFungible{..} => LockContents::NonFungible(item.as_non_fungible().non_fungible_local_ids()),
            };

            // Deposit the item into the vault
            self.vaults.insert(id.clone(), Vault::with_bucket(item));

            // Emit new lock event
            Runtime::emit_event(EventLock {
                lock_id: id.clone(),
                resource,
                locked_contents: locked_contents.clone(),
                locked_at: Clock::current_time(TimePrecisionV2::Second),
                unlockable_at,
            });

            // Mint and return the lock receipt
            self.lock_receipt_manager.mint_non_fungible(&id, LockReceipt {
                name,
                description,
                key_image_url,
                resource,
                locked_contents,
                locked_at: current_time,
                unlockable_at,
                unlocked_at: None,
            })
        }
        
        /// Takes one or more lock receipts and returns the items they unlock.
        /// 
        /// # Arguments
        /// 
        /// * `lock_receipts` - The lock receipts to unlock.
        /// 
        /// # Returns
        /// 
        /// A vector of buckets, each containing the item that was unlocked.
        /// 
        /// # Emits
        /// 
        /// * `EventUnlock` - An event emitted when an item is unlocked.
        /// 
        /// # Panics
        /// 
        /// * If the lock receipts are invalid.
        /// * If any lock is not unlockable.
        /// 
        pub fn unlock(&mut self, lock_receipts: Bucket) -> Vec<Bucket> {
            // Assert valid lock receipts
            assert!(
                lock_receipts.resource_address() == self.lock_receipt_manager.address(),
                "Invalid lock receipts"
            );

            // Get the current time
            let current_time = Clock::current_time(TimePrecisionV2::Second);

            // Check each lock receipt
            let mut items = Vec::new();
            for lock_receipt in lock_receipts.as_non_fungible().non_fungibles::<LockReceipt>() {
                // Get the id and the lock receipt data
                let id = lock_receipt.local_id();
                let lock_receipt_data = lock_receipt.data();

                // Assert item is unlockable
                assert!(
                    lock_receipt_data.unlockable_at.is_some() && current_time >= lock_receipt_data.unlockable_at.unwrap(),
                    "Item can not yet be unlocked, unlockable at: {:?}", lock_receipt_data.unlockable_at
                );

                // Take the item from the vault
                let item = self.vaults.get_mut(id).unwrap().take_all();                
                items.push(item);

                // Update the lock receipt
                self.lock_receipt_manager.update_non_fungible_data(&id, "unlocked_at", Some(current_time));

                // Emit unlock event
                Runtime::emit_event(EventUnlock {
                    lock_id: id.clone(),
                    resource: lock_receipt_data.resource,
                    locked_contents: lock_receipt_data.locked_contents,
                    locked_at: lock_receipt_data.locked_at,
                    unlockable_at: lock_receipt_data.unlockable_at,
                    unlocked_at: current_time,
                });
            }

            // Store the used lock receipts
            self.used_lock_receipts.put(lock_receipts);

            // Return the items
            items
        }
    }
}
