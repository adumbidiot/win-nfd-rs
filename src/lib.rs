pub mod c_wide_string;
pub mod fileapi;
pub mod shobjidl;

pub use self::c_wide_string::CWideStr;
pub use self::c_wide_string::CWideString;
pub use self::c_wide_string::NulError;
pub use self::fileapi::get_full_path_name;
pub use self::shobjidl::DisplayNameType;
pub use self::shobjidl::FileDialog;
pub use self::shobjidl::FileFilters;
pub use self::shobjidl::FileOpenDialog;
pub use self::shobjidl::FileSaveDialog;
pub use self::shobjidl::ModalWindow;
pub use self::shobjidl::ShellItem;
pub use skylight::CoTaskMemWideString;
pub use skylight::HResult;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

/// An error  that may occur during the use of a file dialog
#[derive(Debug, thiserror::Error)]
pub enum NfdError {
    /// An API call failed
    #[error(transparent)]
    HResult(#[from] skylight::HResult),

    /// A string contained an interior NUL
    #[error("a string contained an interior NUL")]
    NulError(#[from] NulError),
}

/// Builder for a [`FileOpenDialog`]
pub struct FileOpenDialogBuilder<'a, 'b, 'c> {
    /// Whether to init com
    pub init_com: bool,

    /// Path to open by default
    pub default_path: Option<&'a Path>,

    /// Path to open, regardless of past choices
    pub path: Option<&'b Path>,

    /// File types
    pub filetypes: FileFilters<'static>,

    /// Filename
    pub filename: Option<&'c OsStr>,
}

impl<'a, 'b, 'c> FileOpenDialogBuilder<'a, 'b, 'c> {
    /// Make a new [`FileOpenDialogBuilder`].
    pub fn new() -> Self {
        FileOpenDialogBuilder {
            init_com: false,
            default_path: None,
            path: None,
            filetypes: FileFilters::new(),
            filename: None,
        }
    }

    /// Whether to init com
    pub fn init_com(&mut self) -> &mut Self {
        self.init_com = true;
        self
    }

    /// Set the default path where the dialog will open
    pub fn default_path(&mut self, default_path: &'a Path) -> &mut Self {
        self.default_path = Some(default_path);
        self
    }

    /// Set the path where the dialog will open
    pub fn path(&mut self, path: &'b Path) -> &mut Self {
        self.path = Some(path);
        self
    }

    /// Add a file type.
    ///
    /// # Panics
    /// Panics if the name of filter contain an interior NUL.
    pub fn filetype(&mut self, name: &OsStr, filter: &OsStr) -> &mut Self {
        let name = Cow::Owned(CWideString::new(name).expect("name contained an interior NUL"));
        let filter =
            Cow::Owned(CWideString::new(filter).expect("filter contained an interior NUL"));
        self.filetypes.add_filter(name, filter);
        self
    }

    /// Set the default filename
    pub fn filename(&mut self, filename: &'c OsStr) -> &mut Self {
        self.filename = Some(filename);
        self
    }

    /// Build a dialog.
    pub fn build(&self) -> Result<FileOpenDialog, NfdError> {
        if self.init_com {
            skylight::init_mta_com_runtime()?;
        }

        let dialog = FileOpenDialog::new()?;

        if let Some(default_path) = self.default_path {
            let shell_item = ShellItem::from_path(default_path)?;
            dialog.set_default_folder(shell_item)?;
        }

        if let Some(path) = self.path {
            let shell_item = ShellItem::from_path(path)?;
            dialog.set_folder(shell_item)?;
        }

        if !self.filetypes.is_empty() {
            dialog.set_filetypes(&self.filetypes)?;
        }

        if let Some(filename) = self.filename {
            let filename = CWideString::new(filename)?;
            dialog.set_filename(&filename)?;
        }

        Ok(dialog)
    }

    /// Execute a dialog.
    pub fn execute(&self) -> Result<PathBuf, NfdError> {
        let dialog = self.build()?;

        dialog.show(None)?;
        let shellitem = dialog.get_result()?;

        Ok(PathBuf::from(
            shellitem
                .get_display_name(DisplayNameType::FileSysPath)?
                .as_os_string(),
        ))
    }
}

impl Default for FileOpenDialogBuilder<'_, '_, '_> {
    fn default() -> Self {
        FileOpenDialogBuilder::new()
    }
}

/// Builder for a FileSaveDialog
pub struct FileSaveDialogBuilder<'a, 'b, 'c> {
    /// Whether to init com
    pub init_com: bool,

    /// Path to open by default
    pub default_path: Option<&'a Path>,

    /// Path to open, regardless of past choices
    pub path: Option<&'b Path>,

    /// File types
    pub filetypes: FileFilters<'static>,

    /// Filename
    pub filename: Option<&'c OsStr>,
}

impl<'a, 'b, 'c> FileSaveDialogBuilder<'a, 'b, 'c> {
    /// Make a new FileSaveDialogBuilder
    pub fn new() -> Self {
        FileSaveDialogBuilder {
            init_com: false,
            default_path: None,
            path: None,
            filetypes: FileFilters::new(),
            filename: None,
        }
    }

    /// Whether to init com
    pub fn init_com(&mut self) -> &mut Self {
        self.init_com = true;
        self
    }

    /// Set the default path where the dialog will open
    pub fn default_path(&mut self, default_path: &'a Path) -> &mut Self {
        self.default_path = Some(default_path);
        self
    }

    /// Set the path where the dialog will open
    pub fn path(&mut self, path: &'b Path) -> &mut Self {
        self.path = Some(path);
        self
    }

    /// Add a file type.
    ///
    /// # Panics
    /// Panics if the name of filter contain an interior NUL.
    pub fn filetype(&mut self, name: &OsStr, filter: &OsStr) -> &mut Self {
        let name = Cow::Owned(CWideString::new(name).expect("name contained an interior NUL"));
        let filter =
            Cow::Owned(CWideString::new(filter).expect("filter contained an interior NUL"));
        self.filetypes.add_filter(name, filter);
        self
    }

    /// Set the default filename
    pub fn filename(&mut self, filename: &'c OsStr) -> &mut Self {
        self.filename = Some(filename);
        self
    }

    /// Build a dialog.
    pub fn build(&self) -> Result<FileSaveDialog, NfdError> {
        if self.init_com {
            skylight::init_mta_com_runtime()?;
        }

        let dialog = FileSaveDialog::new()?;

        if let Some(default_path) = self.default_path {
            let shell_item = ShellItem::from_path(default_path)?;
            dialog.set_default_folder(shell_item)?;
        }

        if let Some(path) = self.path {
            let shell_item = ShellItem::from_path(path)?;
            dialog.set_folder(shell_item)?;
        }

        if !self.filetypes.is_empty() {
            dialog.set_filetypes(&self.filetypes)?;
        }

        if let Some(filename) = self.filename {
            let filename = CWideString::new(filename)?;
            dialog.set_filename(&filename)?;
        }

        Ok(dialog)
    }

    /// Execute a dialog.
    pub fn execute(&self) -> Result<PathBuf, NfdError> {
        let dialog = self.build()?;

        dialog.show(None)?;
        let shellitem = dialog.get_result()?;

        Ok(PathBuf::from(
            shellitem
                .get_display_name(DisplayNameType::FileSysPath)?
                .as_os_string(),
        ))
    }
}

impl Default for FileSaveDialogBuilder<'_, '_, '_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Default nfd open dialog.
/// Look at this functions impl and write your own if you need more control
pub fn nfd_open() -> Result<PathBuf, NfdError> {
    FileOpenDialogBuilder::new().init_com().execute()
}

/// Default nfd save dialog.
/// Look at this functions impl and write your own if you need more control
pub fn nfd_save() -> Result<PathBuf, NfdError> {
    FileSaveDialogBuilder::new().init_com().execute()
}

/// Shothand for `FileOpenDialogBuilder::new().init_com()`
pub fn nfd_open_builder<'a, 'b, 'c>() -> FileOpenDialogBuilder<'a, 'b, 'c> {
    let mut builder = FileOpenDialogBuilder::new();
    builder.init_com();
    builder
}

/// Shothand for `FileSaveDialogBuilder::new().init_com()`
pub fn nfd_save_builder<'a, 'b, 'c>() -> FileSaveDialogBuilder<'a, 'b, 'c> {
    let mut builder = FileSaveDialogBuilder::new();
    builder.init_com();
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use winapi::um::shellscalingapi::{SetProcessDpiAwareness, PROCESS_PER_MONITOR_DPI_AWARE};

    /// Make the dialog window dpi aware
    fn set_dpi() {
        static SET_DPI: Once = Once::new();
        unsafe {
            SET_DPI.call_once(|| {
                SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
            });
        }
    }

    #[test]
    #[ignore]
    fn it_works_open_default() {
        set_dpi();

        println!(
            "Open File Path (nfd): {}",
            nfd_open().expect("nfd").display()
        );
    }

    #[test]
    #[ignore]
    fn it_works_open() {
        set_dpi();

        let path = FileOpenDialogBuilder::new()
            .init_com()
            .default_path(".".as_ref())
            .path(".".as_ref())
            .filetype("toml".as_ref(), "*.toml".as_ref())
            .filetype("sks".as_ref(), "*.txt;*.lbl".as_ref())
            .execute()
            .expect("file dialog failed to execute");

        println!("Open File Path (builder): {}", path.display());
    }

    #[test]
    #[ignore]
    fn it_works_save_default() {
        set_dpi();

        println!(
            "Save File Path (nfd): {}",
            nfd_open().expect("open nfd failed").display()
        );
    }

    #[test]
    #[ignore]
    fn it_works_save() {
        set_dpi();

        let path = FileSaveDialogBuilder::new()
            .init_com()
            .default_path(".".as_ref())
            .path(".".as_ref())
            .filetype("toml".as_ref(), "*.toml".as_ref())
            .filetype("sks".as_ref(), "*.txt;*.lbl".as_ref())
            .filename("level.txt".as_ref())
            .execute()
            .expect("file dialog failed to exececute");

        println!("Save File Path (builder): {}", path.display());
    }
}
