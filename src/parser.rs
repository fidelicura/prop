use std::env::args;



#[inline]
pub(crate) fn get_args() -> Vec<String> {
    args().skip(1).collect()
}
