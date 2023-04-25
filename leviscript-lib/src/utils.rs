//! contains small utility functions that have nowhere else to go

/// "casts" a reference to the corresponding byte-sequence
pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>()) }
}

/// returns the crate version
pub fn get_version() -> [u16; 3] {
    let version_str = env!("CARGO_PKG_VERSION");
    version_str
        .split('.')
        .map(|x| {
            x.parse::<u16>()
                .expect("invalid version string, can't parse elems as u16")
        })
        .collect::<Vec<_>>()
        .try_into()
        .expect("Invalid version string (wrong number of dots, expected two)")
}
