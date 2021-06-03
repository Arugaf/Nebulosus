#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::codec::{Encode, Decode};

use sp_std::ops::{BitAnd, BitOr};
use frame_support::sp_runtime::traits::Zero;
use frame_support::sp_runtime::traits::Member;
use frame_support::sp_runtime::traits::MaybeSerializeDeserialize;
use frame_support::Parameter;

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
    Account: Default + Parameter + Member + MaybeSerializeDeserialize + Ord,
    // SizeT: Default,
    // Group: Default,
    Time: Default,
    Block: Default,
    FileMode: Default,
    Permissions: Zero + Copy + From<u8> + BitAnd<Output=Permissions> + BitOr<Output=Permissions>,
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

    // groups?
    pub fn check_permissions(&self, who: &Account) -> bool {
        if who == &self.owner {
            !(self.owner_permissions & WRITE.into()).is_zero()
        } else {
            !(self.others_permissions & WRITE.into()).is_zero()
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
        // type FileSizeT: Default + Decode + Encode;

        /// Maximum size of single file in bytes
        #[pallet::constant]
        type MaxFileSize: Get<u32>;
        /// Maximum size of all files compiled (up to 4Gb)
        type MaxFsSize: Get<u32>;
        /// Maximum num of inodes
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
    pub(super) type Files<T: Config> = StorageMap<
        _,
        Blake2_256,
        u32,
        Bytes,
        ValueQuery
    >;

    #[pallet::storage]
    pub(super) type CurrentInode<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    pub(super) type FreeInodes<T: Config> = StorageValue<_, Vec<u32>, ValueQuery>;

    #[pallet::storage]
    pub(super) type CurrentFsSize<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub start_fs_size: u32,
        pub start_inode_num: u32,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                start_fs_size: 0u32,
                start_inode_num: 0u32,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            CurrentInode::<T>::put(&self.start_inode_num);
            CurrentFsSize::<T>::put(&self.start_fs_size);
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A directory was created [who, name, inode]
        DirectoryCreated(T::AccountId, Text, u32),
        /// A directory was deleted [who, name, inode]
        DirectoryDeleted(T::AccountId, Text, u32),
        /// A directory was renamed [who, old_name, new_name]
        DirectoryRenamed(T::AccountId, Text, Text),
        /// A directory was moved to another directory [who, old_path, new_path]
        DirectoryMoved(T::AccountId, Text, Text),

        /// A file was created [who, name, inode, dir_inode]
        FileCreated(T::AccountId, Text, u32, u32),
        /// A file was deleted [who, name, inode]
        FileDeleted(T::AccountId, Text, u32),
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
        DirIsNotEmpty,
        NotDirectory,
        NoDirectoryWithSuchName,
        FileAlreadyExists,
        NoSpace,
        FileIsTooBig,
        NotFile,
        NoFileWithSuchName,
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

            let free_inodes = FreeInodes::<T>::get();
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
            ensure!(Inodes::<T>::get(&parent_inode).check_permissions(&who),
                    Error::<T>::NotEnoughPermissions);

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
                        FreeInodes::<T>::mutate(|inodes| {
                            inodes.pop();
                        })
                    } else {
                        CurrentInode::<T>::put(cur_node + 1);
                    }

                    Self::deposit_event(Event::DirectoryCreated(who, dir_name, cur_node));

                    Ok(().into())
                }
            }
        }

        #[pallet::weight(1_000_000)]
        pub(super) fn delete_dir_by_name(
            origin: OriginFor<T>,
            dir_name: Text,
            parent_inode: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check if there is directory with such inode
            ensure!(Directories::<T>::contains_key(&parent_inode),
                    Error::<T>::IncorrectParentInode);

            // Check permissions for parent inode
            ensure!(Inodes::<T>::get(&parent_inode).check_permissions(&who),
                    Error::<T>::NotEnoughPermissions);

            let parent_dir = Directories::<T>::get(&parent_inode);
            // Search for a given name in current directory
            match parent_dir.binary_search_by(|probe| probe.0.cmp(&dir_name)) {
                // If found...
                Ok(index) => {
                    let inode_to_del = parent_dir[index].1;
                    let dir_to_del_inode = Inodes::<T>::get(inode_to_del);

                    // Check permissions for inode we want to delete
                    ensure!(dir_to_del_inode.check_permissions(&who),
                            Error::<T>::NotEnoughPermissions);

                    // Check if directory
                    ensure!(dir_to_del_inode.file_mode == DIRECTORY, Error::<T>::NotDirectory);

                    // Check for emptiness
                    ensure!(Directories::<T>::get(inode_to_del).is_empty(), Error::<T>::DirIsNotEmpty);

                    // Delete directory from parent dir
                    Directories::<T>::mutate(parent_inode, |inode| {
                        inode.remove(index);
                    });

                    Directories::<T>::remove(inode_to_del);

                    let cur_timestamp = <pallet_timestamp::Pallet<T>>::get();
                    // Change last modified and last changed timestamps of parent inode
                    Inodes::<T>::mutate(parent_inode, |inode| {
                        inode.modified = cur_timestamp.clone();
                        inode.changed = cur_timestamp.clone();
                    });

                    // Remove inode from inodes list
                    Inodes::<T>::remove(inode_to_del);

                    // Update FreeInodes list
                    FreeInodes::<T>::mutate(|inodes| {
                        inodes.push(inode_to_del);
                    });

                    Self::deposit_event(Event::DirectoryDeleted(who, dir_name, inode_to_del));

                    Ok(().into())
                }

                Err(_) => Err(Error::<T>::NoDirectoryWithSuchName.into()),
            }
        }

        #[pallet::weight(1_000_000)]
        pub(super) fn create_file(
            origin: OriginFor<T>,
            filename: Text,
            file_content: Bytes,
            parent_inode: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Todo: вынести дублирование кода в отдельные функции
            let free_inodes = FreeInodes::<T>::get();
            let cur_node = match (&free_inodes).len() {
                0 => CurrentInode::<T>::get(),
                _ => (&free_inodes)[(&free_inodes).len() - 1]
            };

            // Check if there is place and space for new files
            ensure!(T::MaxNumOfFiles::get() >= cur_node, Error::<T>::TooManyFiles);
            ensure!(CurrentFsSize::<T>::get() <= T::MaxFsSize::get(), Error::<T>::NoSpace);

            // Check if file size is too large
            ensure!(file_content.len() <= T::MaxFileSize::get() as usize, Error::<T>::FileIsTooBig);

            // Check if filename length is less than max
            ensure!(T::MaxFilename::get() as usize >= filename.len(), Error::<T>::NameIsTooBig);

            // Check if there is directory with such node
            ensure!(Directories::<T>::contains_key(&parent_inode), // Заменить на метод is_directory для структуры inode?
                    Error::<T>::IncorrectParentInode);

            // Check permissions
            ensure!(Inodes::<T>::get(&parent_inode).check_permissions(&who),
                    Error::<T>::NotEnoughPermissions);

            match Directories::<T>::get(&parent_inode)
                // Search for a given name in current directory
                .binary_search_by(|probe| probe.0.cmp(&filename)) {
                // We cannot create file with already existing name in the directory
                Ok(_) => Err(Error::<T>::FileAlreadyExists.into()),

                Err(index) => {
                    let cur_timestamp = <pallet_timestamp::Pallet<T>>::get();

                    // Create new file metadata and store it into list of Inodes
                    Inodes::<T>::insert(cur_node, INode::<T>::new(
                        who.clone(),
                        // <T>::FileSizeT::default(),
                        // <T>::Groups::default(),
                        cur_timestamp.clone(),
                        cur_timestamp.clone(),
                        cur_timestamp.clone(),
                        <frame_system::Pallet<T>>::block_number(),
                        REGULAR,
                        // Vec::new(),
                        READ | WRITE,
                        READ | WRITE,
                        READ,
                    ));

                    // Add new file to the parent inode
                    Directories::<T>::mutate(parent_inode, |inode| {
                        inode.insert(index, (filename.clone(), cur_node));
                    });

                    // Change last modified and last changed timestamps of parent node
                    Inodes::<T>::mutate(parent_inode, |inode| {
                        inode.modified = cur_timestamp.clone();
                        inode.changed = cur_timestamp.clone();
                    });

                    // Add new file to list of files
                    Files::<T>::insert(cur_node, &file_content);

                    // Increment our current_node if it's not free_inode // Todo: change it later to some kind of method
                    if (&free_inodes).len() > 0 && cur_node == (&free_inodes)[(&free_inodes).len() - 1] {
                        FreeInodes::<T>::mutate(|inodes| {
                            inodes.pop();
                        })
                    } else {
                        CurrentInode::<T>::put(cur_node + 1);
                    }

                    // It's okay to convert into u32 cause MaxFileSize: u32
                    CurrentFsSize::<T>::put(CurrentFsSize::<T>::get() + file_content.len() as u32);

                    Self::deposit_event(Event::FileCreated(who, filename, cur_node, parent_inode));

                    Ok(().into())
                }
            }
        }

        #[pallet::weight(1_000_000)]
        pub(super) fn delete_file_by_name(
            origin: OriginFor<T>,
            filename: Text,
            parent_inode: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check if there is directory with such inode
            ensure!(Directories::<T>::contains_key(&parent_inode),
                    Error::<T>::IncorrectParentInode);

            // Check permissions for parent inode
            ensure!(Inodes::<T>::get(&parent_inode).check_permissions(&who),
                    Error::<T>::NotEnoughPermissions);

            let parent_dir = Directories::<T>::get(&parent_inode);
            // Search for a given name in current directory
            match parent_dir.binary_search_by(|probe| probe.0.cmp(&filename)) {
                // If found...
                Ok(index) => {
                    let inode_to_del = parent_dir[index].1;
                    let file_to_del_inode = Inodes::<T>::get(inode_to_del);

                    // Check permissions for inode we want to delete
                    ensure!(file_to_del_inode.check_permissions(&who),
                            Error::<T>::NotEnoughPermissions);

                    // Check if file
                    ensure!(file_to_del_inode.file_mode == REGULAR, Error::<T>::NotFile);

                    // Delete file from parent dir
                    Directories::<T>::mutate(parent_inode, |inode| {
                        inode.remove(index);
                    });

                    Files::<T>::remove(inode_to_del);

                    let cur_timestamp = <pallet_timestamp::Pallet<T>>::get();
                    // Change last modified and last changed timestamps of parent inode
                    Inodes::<T>::mutate(parent_inode, |inode| {
                        inode.modified = cur_timestamp.clone();
                        inode.changed = cur_timestamp.clone();
                    });

                    CurrentFsSize::<T>::put(CurrentFsSize::<T>::get() - Files::<T>::get(inode_to_del).len() as u32);

                    Files::<T>::remove(inode_to_del);

                    // Remove inode from inodes list
                    Inodes::<T>::remove(inode_to_del);

                    // Update FreeInodes list
                    FreeInodes::<T>::mutate(|inodes| {
                        inodes.push(inode_to_del);
                    });

                    Self::deposit_event(Event::FileDeleted(who, filename, inode_to_del));

                    Ok(().into())
                }

                Err(_) => Err(Error::<T>::NoFileWithSuchName.into()),
            }
        }
    }
}
