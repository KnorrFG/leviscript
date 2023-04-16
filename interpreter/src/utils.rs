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
