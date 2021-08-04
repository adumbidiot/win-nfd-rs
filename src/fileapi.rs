use crate::c_wide_string::CWideStr;
use crate::c_wide_string::CWideString;
use skylight::HResult;
use std::convert::TryInto;
use winapi::shared::minwindef::MAX_PATH;
use winapi::um::fileapi::GetFullPathNameW;

/// Get the full path name.
///
/// Returns a tuple. If the path refers to a file, the second element of the tuple is the starting index of the filename.
/// Get the [`CWideString`] as a slice and index that to access the filename.
pub fn get_full_path_name(input_path: &CWideStr) -> Result<(CWideString, Option<usize>), HResult> {
    let mut path = Vec::with_capacity(MAX_PATH);
    let mut file_part = std::ptr::null_mut();

    let mut size = MAX_PATH as u32;
    loop {
        size = unsafe {
            GetFullPathNameW(input_path.as_ptr(), size, path.as_mut_ptr(), &mut file_part)
        };

        if size == 0 {
            return Err(HResult::get_last_error());
        }

        let size_usize: usize = size.try_into().expect("path len cannot fit in a usize");
        if size_usize < MAX_PATH {
            unsafe {
                path.set_len(size_usize + 1);
            }
            let filename_offset = if !file_part.is_null() {
                // TODO: I think i'm doing this right, but is file_part always guaranteed to be larger than the path ptr?
                let diff = file_part as usize - path.as_ptr() as usize;
                // Divide by 2 since there are 2 bytes per wide char
                Some(diff / 2)
            } else {
                None
            };
            let ret = CWideString::from_vec_with_nul(path).expect("path contained interior NULs");
            return Ok((ret, filename_offset));
        }

        // The buffer was too small. Resize and try again.
        path.reserve(size_usize);
    }
}
