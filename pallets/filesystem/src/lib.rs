#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime definition of an event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn something)]
    // Learn more about declaring storage items:
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
    pub type Something<T> = StorageValue<_, u32>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A directory was created [who, name]
        DirectoryCreated(T::AccountId, Vec<u8>),
        /// A directory was deleted [who, name]
        DirectoryDeleted(T::AccountId, Vec<u8>),
        /// A directory was renamed [who, old_name, new_name]
        DirectoryRenamed(T::AccountId, Vec<u8>, Vec<u8>),
        /// A directory was moved to another directory [who, old_path, new_path]
        DirectoryMoved(T::AccountId, Vec<u8>, Vec<u8>),

        /// A file was created [who, name]
        FileCreated(T::AccountId, Vec<u8>),
        /// A file was deleted [who, name]
        FileDeleted(T::AccountId, Vec<u8>),
        /// A file was changed [who, name]
        FileChanged(T::AccountId, Vec<u8>),
        /// A file was renamed [who, old_name, new_name]
        FileRenamed(T::AccountId, Vec<u8>, Vec<u8>),
        /// A file was moved [who, old_path, new_path]
        FileMoved(T::AccountId, Vec<u8>, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A directory/file with the same name is already exists
        AlreadyExists,
        /// A directory/file with this name does not exist
        DoesNotExist,
        /// No such directory
        IncorrectPath,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T:Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;

            // Update storage.
            <Something<T>>::put(something);

            // Emit an event.
            Self::deposit_event(Event::SomethingStored(something, who));
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        /// An example dispatchable that may throw a custom error.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Read a value from storage.
            match <Something<T>>::get() {
                // Return an error if the value has not been set.
                None => Err(Error::<T>::NoneValue)?,
                Some(old) => {
                    // Increment the value read from storage; will error in the event of overflow.
                    let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
                    // Update the value in storage with the incremented result.
                    <Something<T>>::put(new);
                    Ok(())
                },
            }
        }
    }
}
