#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::codec::{Encode, Decode};

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    type Text = Vec<u8>;
    type Bytes = Vec<u8>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime definition of an event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /*#[derive(Encode, Decode, Default, Clone, PartialEq)]
    pub struct File<T: Config> {
        name: Vec<u8>,
        file_type: Vec<u8>,
        owner: T::AccountId,
        // changes: Vec<(T::AccountId, u64)>,
        content: Vec<u8>,
        // permissions
        block: T::BlockNumber,
    }*/

    #[pallet::storage]
    pub(super) type Files<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Text,
        (Text, Text, T::AccountId, Bytes, T::BlockNumber),
        ValueQuery>;

    #[pallet::storage]
    // Temporary implementation (Vec<Vec<u8>>)
    pub(super) type Directories<T: Config> = StorageMap<_, Blake2_128Concat, Text, Vec<Text>, ValueQuery>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A directory was created [who, name]
        DirectoryCreated(T::AccountId, Text),
        /// A directory was deleted [who, name]
        DirectoryDeleted(T::AccountId, Text),
        /// A directory was renamed [who, old_name, new_name]
        DirectoryRenamed(T::AccountId, Text, Text),
        /// A directory was moved to another directory [who, old_path, new_path]
        DirectoryMoved(T::AccountId, Text, Text),

        /// A file was created [who, name]
        FileCreated(T::AccountId, Text),
        /// A file was deleted [who, name]
        FileDeleted(T::AccountId, Text),
        /// A file was changed [who, name]
        FileChanged(T::AccountId, Text),
        /// A file was renamed [who, old_name, new_name]
        FileRenamed(T::AccountId, Text, Text),
        /// A file was moved [who, old_path, new_path]
        FileMoved(T::AccountId, Text, Text),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A directory/file with the same name is already exists
        AlreadyExists,
        /// A directory/file with this name does not exist
        DoesNotExist,
        /// No such directory
        IncorrectPath,
        /// Name should start with / and contains full path to file
        IncorrectName,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T:Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub(super) fn create_dir(
            origin: OriginFor<T>,
            mut name: Text,
        ) -> DispatchResultWithPostInfo {

            let sender = ensure_signed(origin)?;

            ensure!(name[0] == b'/', Error::<T>::IncorrectName);
            if name[name.len() - 1] == b'/' {
                name.remove(name.len() - 1);
            }

            ensure!(!Directories::<T>::contains_key(&name), Error::<T>::AlreadyExists);

            // Directories::<T>::insert(&name, (&name, file_type, &sender, content, current_block));
            Self::deposit_event(Event::FileCreated(sender, name));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub(super) fn create_file(
            origin: OriginFor<T>,
            name: Text,
            file_type: Text,
            content: Bytes
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(!Files::<T>::contains_key(&name), Error::<T>::AlreadyExists);

            let current_block = <frame_system::Module<T>>::block_number();
            Files::<T>::insert(&name, (&name, file_type, &sender, content, current_block));
            Self::deposit_event(Event::FileCreated(sender, name));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub(super) fn rename_file(
            origin: OriginFor<T>,
            old_name: Text,
            new_name: Text
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(Files::<T>::contains_key(&old_name), Error::<T>::DoesNotExist);
            ensure!(Files::<T>::contains_key(&new_name), Error::<T>::AlreadyExists);

            let current_block = <frame_system::Module<T>>::block_number();
            let mut file = Files::<T>::get(&old_name);
            file.0 = new_name.clone();

            // Error check
            Files::<T>::remove(&old_name);

            Files::<T>::insert(&new_name, file);

            // Files::<T>::mutate(&old_name, (&name, file_type, &sender, content, current_block));
            Self::deposit_event(Event::FileRenamed(sender, old_name, new_name));

            Ok(().into())
        }
    }
}
