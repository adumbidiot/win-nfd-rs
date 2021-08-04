use crate::get_full_path_name;
use crate::CWideStr;
use crate::CWideString;
use skylight::CoTaskMemWideString;
use skylight::HResult;
use std::borrow::Cow;
use std::convert::TryInto;
use std::ops::Deref;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr::NonNull;
use winapi::shared::guiddef::REFIID;
use winapi::shared::ntdef::HRESULT;
use winapi::shared::ntdef::PCWSTR;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::FAILED;
use winapi::um::combaseapi::CLSCTX_ALL;
use winapi::um::shobjidl::IFileDialog;
use winapi::um::shobjidl::IFileOpenDialog;
use winapi::um::shobjidl::IFileSaveDialog;
use winapi::um::shobjidl_core::CLSID_FileOpenDialog;
use winapi::um::shobjidl_core::CLSID_FileSaveDialog;
use winapi::um::shobjidl_core::IModalWindow;
use winapi::um::shobjidl_core::IShellItem;
use winapi::um::shobjidl_core::SHCreateItemFromParsingName;
use winapi::um::shobjidl_core::SIGDN;
use winapi::um::shobjidl_core::SIGDN_DESKTOPABSOLUTEEDITING;
use winapi::um::shobjidl_core::SIGDN_DESKTOPABSOLUTEPARSING;
use winapi::um::shobjidl_core::SIGDN_FILESYSPATH;
use winapi::um::shobjidl_core::SIGDN_NORMALDISPLAY;
use winapi::um::shobjidl_core::SIGDN_PARENTRELATIVE;
use winapi::um::shobjidl_core::SIGDN_PARENTRELATIVEEDITING;
use winapi::um::shobjidl_core::SIGDN_PARENTRELATIVEFORADDRESSBAR;
use winapi::um::shobjidl_core::SIGDN_PARENTRELATIVEFORUI;
use winapi::um::shobjidl_core::SIGDN_PARENTRELATIVEPARSING;
use winapi::um::shobjidl_core::SIGDN_URL;
use winapi::um::shtypes::COMDLG_FILTERSPEC;
use winapi::um::shtypes::PCIDLIST_ABSOLUTE;
use winapi::um::shtypes::PIDLIST_ABSOLUTE;
use winapi::um::shtypes::PIDLIST_RELATIVE;
use winapi::Interface;

#[repr(transparent)]
pub struct ModalWindow(NonNull<IModalWindow>);

impl ModalWindow {
    /// Show the window
    pub fn show(&self, parent: Option<HWND>) -> Result<(), HResult> {
        let ret = unsafe { self.0.as_ref().Show(parent.unwrap_or(std::ptr::null_mut())) };

        if FAILED(ret) {
            Err(HResult::from(ret))
        } else {
            Ok(())
        }
    }
}

impl Drop for ModalWindow {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().Release();
        }
    }
}

#[repr(transparent)]
pub struct FileDialog(NonNull<IFileDialog>);

impl FileDialog {
    /// Set the default folder
    pub fn set_default_folder(&self, item: ShellItem) -> Result<(), HResult> {
        let ret = unsafe { self.0.as_ref().SetDefaultFolder(item.0.as_ptr()) };
        // Ownership passed to com
        std::mem::forget(item);

        if FAILED(ret) {
            Err(HResult::from(ret))
        } else {
            Ok(())
        }
    }

    /// Set the folder to open
    pub fn set_folder(&self, item: ShellItem) -> Result<(), HResult> {
        let ret = unsafe { self.0.as_ref().SetFolder(item.0.as_ptr()) };
        // Ownership passed to com
        std::mem::forget(item);

        if FAILED(ret) {
            Err(HResult::from(ret))
        } else {
            Ok(())
        }
    }

    /// Set the file types
    ///
    /// # Panics
    /// Panics if the number of filters cannot fit in a usize.
    pub fn set_filetypes(&self, filters: &FileFilters) -> Result<(), HResult> {
        let filters_len = filters
            .len()
            .try_into()
            .expect("length is longer than a u32");

        // Alright, I'm *fairly* certain this performs a deep copy so I can free filters immediately.
        // Even though some projects like
        // https://chromium.googlesource.com/chromium/src/+/refs/tags/72.0.3591.0/ui/shell_dialogs/execute_select_file_win.cc#75
        // suggest that we need to manually manage the memory,
        // Some projects like gecko:
        // https://github.com/mozilla/gecko-dev/blob/d36cf98aa85f24ceefd07521b3d16b9edd2abcb7/widget/windows/nsFilePicker.h
        // https://github.com/mozilla/gecko-dev/blob/b31b78eea683b0eb341c676adb422cd129909fe9/widget/windows/nsFilePicker.cpp
        // don't seem to worry about it, though perhaps care is taken through the codebase to avoid misusing their wrapper.
        // Some emulation programs, like Wine and ReactOS:
        // https://github.com/wine-mirror/wine/blob/e909986e6ea5ecd49b2b847f321ad89b2ae4f6f1/dlls/comdlg32/itemdlg.c#L2398-L2423
        // https://doxygen.reactos.org/da/da4/dll_2win32_2comdlg32_2itemdlg_8c_source.html
        // perform a copy internally.
        // Windows 10 21H1 120.2212.3530.0 seems to perform a copy as well.
        // This was tested by mutating the string after the SetFileTypes call but before showing the window.
        // Changes in the string were not displayed in the resulting window.
        // This suggests that windows also performs an internal copy,
        // though this cannot be proven for all versions of windows,
        // past or future.
        // In conclusion, it is probably safe to call SetFileTypes with a collection of temporary filters.
        let ret = unsafe { self.0.as_ref().SetFileTypes(filters_len, filters.as_ptr()) };

        if FAILED(ret) {
            return Err(HResult::from(ret));
        }

        Ok(())
    }

    /// Set filename
    pub fn set_filename(&self, filename: &CWideStr) -> Result<(), HResult> {
        let ret = unsafe { self.0.as_ref().SetFileName(filename.as_ptr()) };

        if FAILED(ret) {
            return Err(HResult::from(ret));
        }

        Ok(())
    }

    /// Get single result
    pub fn get_result(&self) -> Result<ShellItem, HResult> {
        let mut ptr = std::ptr::null_mut();
        let ret = unsafe { self.0.as_ref().GetResult(&mut ptr) };

        if FAILED(ret) {
            return Err(HResult::from(ret));
        }
        let ptr = NonNull::new(ptr).expect("ptr was null");
        Ok(ShellItem(ptr))
    }

    /// Show the window
    pub fn show(&self, parent: Option<HWND>) -> Result<(), HResult> {
        let ret = unsafe { self.0.as_ref().Show(parent.unwrap_or(std::ptr::null_mut())) };

        if FAILED(ret) {
            return Err(HResult::from(ret));
        }

        Ok(())
    }
}

impl Deref for FileDialog {
    type Target = ModalWindow;

    fn deref(&self) -> &Self::Target {
        // Safety:
        // ModalWindow's repr is a subset of FileDialog's.
        unsafe { std::mem::transmute::<&FileDialog, &ModalWindow>(self) }
    }
}

impl Drop for FileDialog {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().Release();
        }
    }
}

/// A File Open Dialog
#[repr(transparent)]
pub struct FileOpenDialog(NonNull<IFileOpenDialog>);

impl FileOpenDialog {
    /// Make a new [`FileOpenDialog`].
    pub fn new() -> Result<Self, HResult> {
        let ptr = unsafe { skylight::create_instance(&CLSID_FileOpenDialog, CLSCTX_ALL)? };
        let ptr = NonNull::new(ptr).expect("ptr is null");
        Ok(Self(ptr))
    }
}

impl Deref for FileOpenDialog {
    type Target = FileDialog;

    fn deref(&self) -> &Self::Target {
        // Safety:
        // FileDialog's repr is a subset of FileOpenDialog's.
        unsafe { std::mem::transmute::<&FileOpenDialog, &FileDialog>(self) }
    }
}

impl Drop for FileOpenDialog {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().Release();
        }
    }
}

/// A File Save Dialog
#[repr(transparent)]
pub struct FileSaveDialog(NonNull<IFileSaveDialog>);

impl FileSaveDialog {
    /// Make a new [`FileSaveDialog`].
    pub fn new() -> Result<Self, HResult> {
        let ptr = unsafe { skylight::create_instance(&CLSID_FileSaveDialog, CLSCTX_ALL)? };
        let ptr = NonNull::new(ptr).expect("ptr is null");
        Ok(Self(ptr))
    }
}

impl Deref for FileSaveDialog {
    type Target = FileDialog;

    fn deref(&self) -> &Self::Target {
        // Safety:
        // FileDialog's repr is a subset of FileSaveDialog's.
        unsafe { std::mem::transmute::<&FileSaveDialog, &FileDialog>(self) }
    }
}

impl Drop for FileSaveDialog {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().Release();
        }
    }
}

/// File type filter list
pub struct FileFilters<'s> {
    filters: Vec<COMDLG_FILTERSPEC>,

    storage: Vec<(Cow<'s, CWideStr>, Cow<'s, CWideStr>)>,
}

impl<'s> FileFilters<'s> {
    /// Make an empty list of file type filters
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            storage: Vec::new(),
        }
    }

    /// Get the number of file filters
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            filters: Vec::with_capacity(cap),
            storage: Vec::with_capacity(cap),
        }
    }

    /// Get the number of file filters
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if this has file filters in it
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Get the inner COMDLG_FILTERSPEC list ptr
    pub fn as_ptr(&self) -> *const COMDLG_FILTERSPEC {
        self.filters.as_ptr()
    }

    /// Add a filter
    pub fn add_filter(
        &mut self,
        name: impl Into<Cow<'s, CWideStr>>,
        filter: impl Into<Cow<'s, CWideStr>>,
    ) {
        let name = name.into();
        let filter = filter.into();
        self.filters.push(COMDLG_FILTERSPEC {
            pszName: name.as_ptr(),
            pszSpec: filter.as_ptr(),
        });
        self.storage.push((name, filter));
    }
}

impl Default for FileFilters<'_> {
    fn default() -> Self {
        Self::new()
    }
}

extern "system" {
    fn SHCreateItemFromIDList(
        pidl: PCIDLIST_ABSOLUTE,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
}

/// A Shell Item
#[repr(transparent)]
pub struct ShellItem(NonNull<IShellItem>);

impl ShellItem {
    /// Try to create a [`ShellItem`] from a path.
    ///
    /// This will allocate internally to work with relative paths.
    ///
    /// # Panics
    /// Panics if the path contains interior NULs.
    ///
    /// # Errors
    /// Returns an error if the absolute path could not be acquired or if
    /// the shell item could not be created.
    pub fn from_path(path: &Path) -> Result<Self, HResult> {
        let path = CWideString::new(path).expect("path contains NUL");
        let (path, _filename_index) = get_full_path_name(&path)?;
        Self::from_parsing_name(&path)
    }

    /// Try to create a [`ShellItem`] from a path.
    ///
    /// Note that this does not work with relative paths.
    pub fn from_parsing_name(path: &CWideStr) -> Result<Self, HResult> {
        let mut ptr = std::ptr::null_mut();
        let ret = unsafe {
            SHCreateItemFromParsingName(
                path.as_ptr(),
                std::ptr::null_mut(),
                &IShellItem::uuidof(),
                &mut ptr,
            )
        };

        if FAILED(ret) {
            return Err(HResult::from(ret));
        }

        let ptr = NonNull::new(ptr).expect("ptr is null").cast();

        Ok(Self(ptr))
    }

    /// Try to create a [`ShellItem`] from an [`ItemIdList`].
    pub fn from_id_list(list: &ItemIdList) -> Result<Self, HResult> {
        let mut ptr = std::ptr::null_mut();
        let ret =
            unsafe { SHCreateItemFromIDList(*list.as_ptr(), &IShellItem::uuidof(), &mut ptr) };
        if FAILED(ret) {
            return Err(HResult::from(ret));
        }
        let ptr = NonNull::new(ptr).expect("ptr is null").cast();

        Ok(Self(ptr))
    }

    /// Get the display name of a shell item.
    pub fn get_display_name(
        &self,
        display_type: DisplayNameType,
    ) -> Result<CoTaskMemWideString, HResult> {
        let display_type: SIGDN = display_type.into();
        let mut ptr = std::ptr::null_mut();
        let ret = unsafe { self.0.as_ref().GetDisplayName(display_type, &mut ptr) };

        if FAILED(ret) {
            Err(HResult::from(ret))
        } else {
            let ptr = NonNull::new(ptr).expect("ptr was null");
            Ok(unsafe { CoTaskMemWideString::from_raw(ptr) })
        }
    }
}

impl Drop for ShellItem {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().Release();
        }
    }
}

/// Display name type for shellitem
/// Requests the form of an item's display name to retrieve through IShellItem::GetDisplayName and SHGetNameFromIDList.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum DisplayNameType {
    /// Returns the display name relative to the parent folder. In UI this name is generally ideal for display to the user.
    NormalDisplay,

    /// Returns the parsing name relative to the parent folder. This name is not suitable for use in UI.
    ParentRelativeParsing,

    /// Returns the parsing name relative to the desktop. This name is not suitable for use in UI.
    DesktopAbsoluteParsing,

    /// Returns the editing name relative to the parent folder. In UI this name is suitable for display to the user.
    ParentRelativeEditing,

    /// Returns the editing name relative to the desktop. In UI this name is suitable for display to the user.
    DesktopAbsoluteEditing,

    /// Returns the item's file system path, if it has one.
    /// Only items that report SFGAO_FILESYSTEM have a file system path.
    /// When an item does not have a file system path, a call to IShellItem::GetDisplayName on that item will fail.
    /// In UI this name is suitable for display to the user in some cases, but note that it might not be specified for all items.
    FileSysPath,

    /// Returns the item's URL, if it has one.
    /// Some items do not have a URL, and in those cases a call to IShellItem::GetDisplayName will fail.
    /// This name is suitable for display to the user in some cases, but note that it might not be specified for all items.
    Url,

    /// Returns the path relative to the parent folder in a friendly format as displayed in an address bar.
    /// This name is suitable for display to the user.
    ParentRelativeForAddressBar,

    /// Returns the path relative to the parent folder.
    ParentRelative,

    /// Introduced in Windows 8.
    ParentRelativeForUi,
}

impl From<DisplayNameType> for SIGDN {
    fn from(dnt: DisplayNameType) -> Self {
        match dnt {
            DisplayNameType::NormalDisplay => SIGDN_NORMALDISPLAY,
            DisplayNameType::ParentRelativeParsing => SIGDN_PARENTRELATIVEPARSING,
            DisplayNameType::DesktopAbsoluteParsing => SIGDN_DESKTOPABSOLUTEPARSING,
            DisplayNameType::ParentRelativeEditing => SIGDN_PARENTRELATIVEEDITING,
            DisplayNameType::DesktopAbsoluteEditing => SIGDN_DESKTOPABSOLUTEEDITING,
            DisplayNameType::FileSysPath => SIGDN_FILESYSPATH,
            DisplayNameType::Url => SIGDN_URL,
            DisplayNameType::ParentRelativeForAddressBar => SIGDN_PARENTRELATIVEFORADDRESSBAR,
            DisplayNameType::ParentRelative => SIGDN_PARENTRELATIVE,
            DisplayNameType::ParentRelativeForUi => SIGDN_PARENTRELATIVEFORUI,
        }
    }
}

extern "system" {
    fn ILCreateFromPathW(pszPath: PCWSTR) -> PIDLIST_ABSOLUTE;
    fn ILFree(pidl: PIDLIST_RELATIVE);
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ItemIdList(PIDLIST_ABSOLUTE);

impl ItemIdList {
    /// Create an [`ItemIdList`] from a path.
    ///
    /// # Notes
    /// Alright this function's documentation is horrible, so please PLEASE send a PR if anything looks bad.
    /// This function appears(?) to return NULL if the path is rejected.
    /// I'm *fairly* certain I can get the last error for more info as well.
    /// I also know for a fact that this function rejects relative paths with a last error of 1008,
    /// but I'm not sure why.
    pub fn create_from_path(data: &CWideStr) -> Result<Self, HResult> {
        let ret = unsafe { ILCreateFromPathW(data.as_ptr()) };
        if ret.is_null() {
            return Err(HResult::get_last_error());
        }
        Ok(Self(ret))
    }

    /// Get a ptr to the inner data
    pub fn as_ptr(&self) -> *const PIDLIST_ABSOLUTE {
        &self.0
    }
}

impl Drop for ItemIdList {
    fn drop(&mut self) {
        unsafe { ILFree(self.0) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn shell_item_from_parsing_name() {
        skylight::init_mta_com_runtime().expect("failed to init com");
        let rel_path = CWideString::new("./Cargo.toml").expect("invalid c wide string");
        let (abs_path, filename_index) =
            get_full_path_name(&rel_path).expect("failed to get full path name");
        let filename = &abs_path[filename_index.expect("missing filename")..];
        dbg!(filename);
        dbg!(&abs_path);
        let item = ShellItem::from_parsing_name(&abs_path).expect("failed to make shell item");
        let path = item
            .get_display_name(DisplayNameType::FileSysPath)
            .expect("failed to get path");
        dbg!(path);
    }

    #[test]
    fn bad_id_list_creation() {
        // This rejects relative paths
        let rel_path = CWideString::new("./Cargo.toml").expect("invalid c wide string");
        let id_list = ItemIdList::create_from_path(&rel_path).unwrap_err();

        // I don't know why it does this, but im creating a test to remember that it does this.
        assert_eq!(id_list.0, 1008);
    }

    #[test]
    fn shell_item_from_item_id_list() {
        skylight::init_mta_com_runtime().expect("failed to init com");
        let rel_path = CWideString::new("./Cargo.toml").expect("invalid c wide string");
        let (abs_path, filename_index) =
            get_full_path_name(&rel_path).expect("failed to get full path name");
        let filename = &abs_path[filename_index.expect("missing filename")..];
        dbg!(filename);
        dbg!(&abs_path);
        let id_list = ItemIdList::create_from_path(&abs_path).expect("failed to create id list");
        let item = ShellItem::from_id_list(&id_list).expect("failed to create shell item");
        let path = item
            .get_display_name(DisplayNameType::FileSysPath)
            .expect("failed to get path");
        dbg!(path);
    }
}
