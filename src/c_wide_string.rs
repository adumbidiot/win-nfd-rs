use std::ffi::OsStr;
use std::fmt::Write;
use std::ops::Deref;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

/// Implemented for types that can be converted into wide types
pub trait IntoWide {
    /// Convert this into a vec of wide chars
    ///
    /// Implementors should reserve 1 extra element of space for the nul terminator.
    /// The vec may contain nul elements, but they will be rejected in some places in the api.
    fn into_wide(self) -> Vec<u16>;
}

impl IntoWide for Vec<u16> {
    fn into_wide(mut self) -> Vec<u16> {
        self.reserve(1);
        self
    }
}

impl IntoWide for &OsStr {
    fn into_wide(self) -> Vec<u16> {
        let mut ret = Vec::with_capacity(self.encode_wide().count() + 1);
        ret.extend(self.encode_wide());
        ret
    }
}

impl IntoWide for &Path {
    fn into_wide(self) -> Vec<u16> {
        self.as_os_str().into_wide()
    }
}

impl IntoWide for &str {
    fn into_wide(self) -> Vec<u16> {
        OsStr::new(self).into_wide()
    }
}

impl IntoWide for &CWideStr {
    fn into_wide(self) -> Vec<u16> {
        let slice = self.as_slice();
        let mut ret = Vec::with_capacity(slice.len() + 1);
        ret.extend(slice);
        ret
    }
}

/// A wide analog of https://doc.rust-lang.org/std/ffi/struct.CString.html
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone)]
pub struct CWideString(Box<[u16]>);

impl CWideString {
    pub fn new<D>(data: D) -> Result<Self, NulError>
    where
        D: IntoWide,
    {
        let mut data = data.into_wide();
        if let Some(index) = data.iter().copied().position(|el| el == 0) {
            return Err(NulError(index, data));
        }
        data.push(0);

        Ok(unsafe { Self::from_vec_with_nul_unchecked(data) })
    }

    /// Make a new [`CWideString`] from a vec that is nul terminated.
    ///
    /// # Errors
    /// Errors if data contains interior nuls or is not nul terminated
    pub fn from_vec_with_nul(data: Vec<u16>) -> Result<Self, FromVecWithNulError> {
        let nul_pos = data.iter().copied().position(|el| el == 0);
        match nul_pos {
            Some(nul_pos) if nul_pos == data.len() - 1 => {
                // The only nul is the terminator
            }
            None => {
                return Err(FromVecWithNulError {
                    error_kind: FromWideWithNulErrorKind::NotNulTerminated,
                    data,
                });
            }
            Some(nul_pos) => {
                return Err(FromVecWithNulError {
                    error_kind: FromWideWithNulErrorKind::InteriorNul(nul_pos),
                    data,
                });
            }
        }

        Ok(Self(data.into_boxed_slice()))
    }

    /// Make a new [`CWideString`] from a vec that is nul terminated without checks.
    ///
    /// # Safety
    /// * data must contain no interior nuls
    /// * data must be nul terminated
    pub unsafe fn from_vec_with_nul_unchecked(data: Vec<u16>) -> Self {
        Self(data.into_boxed_slice())
    }

    /// Get this as a [`CWideStr`].
    pub fn as_c_wide_str(&self) -> &CWideStr {
        unsafe { CWideStr::from_wide_with_nul_unchecked(&self.0) }
    }
}

impl Deref for CWideString {
    type Target = CWideStr;

    fn deref(&self) -> &Self::Target {
        self.as_c_wide_str()
    }
}

impl std::fmt::Debug for CWideString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_c_wide_str().fmt(f)
    }
}

impl std::borrow::Borrow<CWideStr> for CWideString {
    fn borrow(&self) -> &CWideStr {
        self.as_c_wide_str()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NulError(usize, Vec<u16>);

impl NulError {
    /// The position of the nul
    pub fn nul_position(&self) -> usize {
        self.0
    }

    /// Return the erroneous wide string
    pub fn into_vec(self) -> Vec<u16> {
        self.1
    }
}

impl std::fmt::Display for NulError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "nul wide char found in provided data at position: {}",
            self.0
        )
    }
}

impl std::error::Error for NulError {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromVecWithNulError {
    error_kind: FromWideWithNulErrorKind,
    data: Vec<u16>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FromWideWithNulErrorKind {
    InteriorNul(usize),
    NotNulTerminated,
}

impl std::fmt::Display for FromVecWithNulError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_kind {
            FromWideWithNulErrorKind::InteriorNul(pos) => {
                write!(
                    f,
                    "data provided contains an interior nul wide char at pos {}",
                    pos
                )
            }
            FromWideWithNulErrorKind::NotNulTerminated => {
                write!(f, "data provided is not nul terminated")
            }
        }
    }
}

impl std::error::Error for FromVecWithNulError {}

pub struct CWideStr {
    inner: [u16],
}

impl CWideStr {
    /// Make a new [`CWideStr`] from wide chars that are nul terminated without checks.
    ///
    /// # Safety
    /// * data must be nul terminated
    /// * data must contain no interior nuls
    pub unsafe fn from_wide_with_nul_unchecked(data: &[u16]) -> &Self {
        &*(data as *const [u16] as *const CWideStr)
    }

    /// Get a pointer to the data.
    pub fn as_ptr(&self) -> *const u16 {
        self.inner.as_ptr()
    }

    /// Get this as a wide slice.
    ///
    /// Does NOT include the NUL terminator.
    pub fn as_slice(&self) -> &[u16] {
        &self.inner[..self.inner.len() - 1]
    }

    /// Get this as a wide slice.
    ///
    /// Does include the NUL terminator.
    pub fn as_slice_with_nul(&self) -> &[u16] {
        &self.inner[..self.inner.len()]
    }

    /// Try to iterate over the chars in this string.
    pub fn chars(&self) -> impl Iterator<Item = Result<char, std::char::DecodeUtf16Error>> + '_ {
        std::char::decode_utf16(self.as_slice().iter().copied())
    }
}

impl std::fmt::Debug for CWideStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        for c in self
            .chars()
            .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
        {
            for c in c.escape_debug() {
                f.write_char(c)?
            }
        }

        f.write_char('"')?;

        Ok(())
    }
}

impl std::ops::Index<std::ops::RangeFrom<usize>> for CWideStr {
    type Output = CWideStr;

    fn index(&self, index: std::ops::RangeFrom<usize>) -> &CWideStr {
        let slice = self.as_slice_with_nul();
        if index.start < slice.len() {
            unsafe { CWideStr::from_wide_with_nul_unchecked(&slice[index.start..]) }
        } else {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                slice.len(),
                index.start
            );
        }
    }
}

impl std::borrow::ToOwned for CWideStr {
    type Owned = CWideString;

    fn to_owned(&self) -> Self::Owned {
        CWideString::new(self).expect("invalid CWideStr")
    }
}
