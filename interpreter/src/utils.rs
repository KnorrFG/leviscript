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
