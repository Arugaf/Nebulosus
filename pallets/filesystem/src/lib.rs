#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::codec::{Encode, Decode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

// Todo: правильно раскидать трейты
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Eq, PartialEq, Default)]
pub struct INodeStruct<Account, /*SizeT, Group,*/ Time, Block, FileMode, Permissions/*, TextT*/> {
    owner: Account,
    // size: SizeT,
    // owner_group: Group,
    modified: Time,
    changed: Time,
    created: Time,
    block: Block,
    file_mode: FileMode,
    // mime_type: TextT,
    owner_permissions: Permissions,
    group_permissions: Permissions,
    others_permissions: Permissions,
}

impl<
    Account: Default,
    // SizeT: Default,
    // Group: Default,
    Time: Default,
    Block: Default,
    FileMode: Default,
    Permissions: Default,
    // TextT: Default,
> INodeStruct<Account, /*SizeT, Group,*/ Time, Block, FileMode, Permissions/*, TextT*/> {
    pub fn new(owner: Account,
               // size: SizeT,
               // owner_group: Group,
               modified: Time,
               changed: Time,
               created: Time,
               block: Block,
               file_mode: FileMode,
               // mime_type: TextT,
               owner_permissions: Permissions,
               group_permissions: Permissions,
               others_permissions: Permissions) -> Self {
        INodeStruct {
            owner,
            // size,
            // owner_group,
            modified,
            changed,
            created,
            block,
            file_mode,
            // mime_type,
            owner_permissions,
            group_permissions,
            others_permissions,
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    use crate::INodeStruct;

    type Text = Vec<u8>;
    type Bytes = Vec<u8>;

    pub type INode<T> = INodeStruct<
        <T as frame_system::Config>::AccountId,
        // <T as Config>::FileSizeT,
        // <T as Config>::Groups,
        <T as pallet_timestamp::Config>::Moment,
        <T as frame_system::Config>::BlockNumber,
        u8,
        u8,
        // Vec<u8>
    >;

    pub const EXECUTE: u8 = 0x01;
    pub const WRITE: u8 = 0x02;
    pub const READ: u8 = 0x04;
    pub const ALL: u8 = EXECUTE | WRITE | READ;

    pub const REGULAR: u8 = 0x00;
    pub const DIRECTORY: u8 = 0x01;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// Runtime definition of an event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Groups: Default + Decode + Encode;
        type FileSizeT: Default + Decode + Encode;

        // max_file_size /// Maximum size of single file
        // max_fs_size /// Maximum size of all files compiled
        /// Maximum num of inodes
        #[pallet::constant]
        type MaxNumOfFiles: Get<u32>;
        /// Maximum length of filename in bytes
        #[pallet::constant]
        type MaxFilename: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    // Todo: make it more generic instead of u32?
    #[pallet::storage]
    pub(super) type Inodes<T: Config> = StorageMap<
        _,
        Blake2_256,
        u32,
        INode<T>,
        ValueQuery
    >;

    #[pallet::storage]
    pub(super) type Directories<T: Config> = StorageMap<
        _,
        Blake2_256,
        u32,
        Vec<(Text, u32)>,
        ValueQuery
    >;

    #[pallet::storage]
    pub(super) type CurrentInode<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    pub(super) type FreeInodes<T: Config> = StorageValue<_, Vec<u32>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub start_inode_num: u32,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                start_inode_num: 0u32,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            CurrentInode::<T>::put(&self.start_inode_num);
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A directory was created [who, name, inode]
        DirectoryCreated(T::AccountId, Text, u32),
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
        IncorrectParentInode,
        DirectoryAlreadyExists,
        NameIsTooBig,
        NotEnoughPermissions,
        TooManyFiles,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000_000)]
        pub(super) fn create_dir(
            origin: OriginFor<T>,
            dir_name: Text,
            parent_inode: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // -------------------------------------------------------------------------delete
            // Temporary implementation cause can't understand how to work with genesis
            // Todo: как бы разобраться как именно это работает и пофиксить
            // Todo: пофиксить genesis инициализацию рута и просто инициализацию структуры (просто кидать структуру не работает)

            let cur_node = CurrentInode::<T>::get();
            // if let cur_node = CurrentInode::<T>::get()
            if cur_node == 0 {
                // Todo: SIZE - КАК БЫ ДЖЕНЕРИК ТАЙП, А КАК ЕГО ЗАСТАВИТЬ РАБОТАТЬ? ОЛО??

                Inodes::<T>::insert(cur_node, INode::<T>::new(
                    who.clone(),
                    // <T>::FileSizeT::default(),
                    // <T>::Groups::default(),
                    <pallet_timestamp::Pallet<T>>::get(),
                    <pallet_timestamp::Pallet<T>>::get(),
                    <pallet_timestamp::Pallet<T>>::get(),
                    <frame_system::Pallet<T>>::block_number(),
                    DIRECTORY,
                    // Vec::new(),
                    EXECUTE | READ | WRITE,
                    EXECUTE | READ | WRITE,
                    EXECUTE | READ | WRITE,
                ));

                let empty_vec: Vec<(Text, u32)> = Vec::new();
                Directories::<T>::insert(cur_node, empty_vec);

                CurrentInode::<T>::put(cur_node + 1); // Todo: отдельный метод на инкремент

                // Todo: а как в расте положить значение в переменную из внутреннего блока?
            }

            // --------------------------------------------------------------------end_delete

            let mut free_inodes = FreeInodes::<T>::get();
            let cur_node = match (&free_inodes).len() {
                0 => CurrentInode::<T>::get(),
                _ => (&free_inodes)[(&free_inodes).len() - 1]
            };

            // Check if there is place for new files
            ensure!(T::MaxNumOfFiles::get() >= cur_node, Error::<T>::TooManyFiles);

            // Check if filename length is less than max
            ensure!(T::MaxFilename::get() as usize >= dir_name.len(), Error::<T>::NameIsTooBig);

            // Check if there is directory with such node
            ensure!(Directories::<T>::contains_key(&parent_inode), // Заменить на метод is_directory для структуры inode?
                    Error::<T>::IncorrectParentInode);

            // Check permissions
            let parent_inode_data = Inodes::<T>::get(&parent_inode);
            if &who == &parent_inode_data.owner { // groups?
                ensure!(WRITE & parent_inode_data.owner_permissions > 1, Error::<T>::NotEnoughPermissions); // sudo?
            } else {
                ensure!(WRITE & parent_inode_data.others_permissions > 1, Error::<T>::NotEnoughPermissions); // sudo?
            } // match?

            match Directories::<T>::get(&parent_inode)
                // Search for a given name in current directory
                .binary_search_by(|probe| probe.0.cmp(&dir_name)) {
                // We cannot create directory with already existing name in the directory
                Ok(_) => Err(Error::<T>::DirectoryAlreadyExists.into()),

                Err(index) => {
                    let cur_timestamp = <pallet_timestamp::Pallet<T>>::get();

                    // Create new directory metadata and store it into list of Inodes
                    Inodes::<T>::insert(cur_node, INode::<T>::new(
                        who.clone(),
                        // <T>::FileSizeT::default(),
                        // <T>::Groups::default(),
                        cur_timestamp.clone(),
                        cur_timestamp.clone(),
                        cur_timestamp.clone(),
                        <frame_system::Pallet<T>>::block_number(),
                        DIRECTORY,
                        // Vec::new(),
                        READ | WRITE | EXECUTE,
                        READ | WRITE | EXECUTE,
                        READ | EXECUTE,
                    ));

                    // Add new directory to the parent inode
                    Directories::<T>::mutate(parent_inode, |inode| {
                        inode.insert(index, (dir_name.clone(), cur_node));
                    });

                    // Change last modified and last changed timestamps of parent node
                    Inodes::<T>::mutate(parent_inode, |inode| {
                        inode.modified = cur_timestamp.clone();
                        inode.changed = cur_timestamp.clone();
                    });

                    // Add new directory to list of directories
                    // Todo: как вставить новый пустой типизированый вектор напрямую?
                    let empty_vec: Vec<(Text, u32)> = Vec::new();
                    Directories::<T>::insert(cur_node, empty_vec);

                    // Increment our current_node if it's not free_inode // Todo: change it later to some kind of method
                    if (&free_inodes).len() > 0 && cur_node == (&free_inodes)[(&free_inodes).len() - 1] {
                        free_inodes.pop();
                    } else {
                        CurrentInode::<T>::put(cur_node + 1);
                    }

                    Self::deposit_event(Event::DirectoryCreated(who, dir_name, cur_node));

                    Ok(().into())
                }
            }
        }
    }
}
