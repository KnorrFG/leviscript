pub fn sequence_result<Iter, Itt, T, E>(results: Iter) -> Result<Vec<T>, E>
where
    Iter: IntoIterator<Item = Result<T, E>, IntoIter = Itt>,
    Itt: Iterator<Item = Result<T, E>>,
{
    let mut res = vec![];
    for r in results {
        res.push(r?);
    }

    Ok(res)
}

pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>()) }
}

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

macro_rules! bug {
    ($msg:literal $(, $args:tt)*) => {
       panic!(concat!("An Interpreter bug occured:\n\n", $msg) $(, $args)*);
    };
}

pub(crate) use bug;
