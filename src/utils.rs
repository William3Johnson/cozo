/*
 * Copyright 2022, The Cozo Project Authors. Licensed under AGPL-3 or later.
 */

#[inline(always)]
pub(crate) fn swap_option_result<T, E>(d: Result<Option<T>, E>) -> Option<Result<T, E>> {
    match d {
        Ok(Some(s)) => Some(Ok(s)),
        Ok(None) => None,
        Err(e) => Some(Err(e)),
    }
}
