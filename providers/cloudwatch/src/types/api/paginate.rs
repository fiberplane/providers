use crate::client::{request_state, CanonicalRequest, ClientCommon};
use fiberplane_pdk::prelude::Error;
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, HashMap};

/// Trait for request that can combine with a "next page" token to create
/// a new version of the same request, but for the next page.
pub trait Paginate: Into<CanonicalRequest<{ request_state::STEM }>> {
    /// Forge a subsequent request for the next page.
    /// Return None if the pagination_token is absent or empty.
    #[must_use("nothing will happen unless this request is sent by a client.")]
    fn next_page(self, pagination_token: Option<String>) -> Option<Self>;
}

/// Helper trait for containers that give a meaningful size estimation
pub trait SizedContainer {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> SizedContainer for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<K, V> SizedContainer for HashMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<K, V> SizedContainer for BTreeMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}

/// Paginate through a request with AWS API.
///
/// The argument hint on results_accessor_function controls the target
/// deserialization type from API response.
///
/// The argument hint on the first argument of fold_function controls
/// the type returned by the helper
///
///
/// - `client` is an [API Client](ClientCommon) ready to make requests
/// - `init_request` is the first request to start paginating on. This initial request failing causes the function to fail completely.
///    Any other request failing also causes the whole function to fail.
/// - `results_accessor_function` takes ownership of the deserialized API call response, to extract an iterator of relevant
///   items to be accumulated.
/// - `pagination_token_accessor_function` borrows the deserialized API call response, and maybe returns a token for the next
///    page (used to compute the next page request with the [Paginate]() trait)
/// - `fold_function` mutates a local accumulator of results using the partial results from a single call. `fold_function` is
///   called once per page, with the accumulated state as first argument and the new batch, ready to be iterated on, as second
/// - `limit` is an optional limit to the number of results we want to see. None means "all results", Some(i) means "at least
///   i results, don't fetch another page once we have i results"
pub async fn paginate<R, F, A, I, C, D, P, J>(
    client: &ClientCommon,
    init_request: R,
    results_accessor_function: A,
    pagination_token_accessor_function: P,
    fold_function: F,
    limit: Option<usize>,
) -> Result<C, Error>
where
    C: Default + SizedContainer,
    R: Paginate + Clone,
    A: Fn(D) -> Option<J>,
    J: Iterator<Item = I>,
    F: Fn(&mut C, J),
    P: Fn(&D) -> Option<String>,
    D: DeserializeOwned,
{
    let mut acc: C = Default::default();
    let mut current_request = init_request;

    loop {
        let response: D = client.send(current_request.clone(), None).await?;
        let next_token = pagination_token_accessor_function(&response);
        if let Some(new_results) = results_accessor_function(response) {
            fold_function(&mut acc, new_results);
        }
        if limit.map_or(false, |limit| acc.len() >= limit) {
            break Ok(acc);
        }

        if let Some(next_page) = current_request.next_page(next_token) {
            current_request = next_page;
        } else {
            break Ok(acc);
        }
    }
}

/// Paginate through a request with AWS API.
///
/// The argument hint on results_accessor_function controls the target
/// deserialization type from API response.
///
/// The type of items returned by the iterator in results_accessor_function
/// controls the type returned by the helper.
///
/// This function is a special use-case of [paginate](paginate) where the
/// returned type is known to be a vector, therefore the "fold_function" is
/// elided.
pub async fn paginate_vec<R, A, I, D, P, J>(
    client: &ClientCommon,
    init_request: R,
    results_accessor_function: A,
    pagination_token_accessor_function: P,
    limit: Option<usize>,
) -> Result<Vec<I>, Error>
where
    R: Paginate + Clone,
    A: Fn(D) -> Option<J>,
    J: Iterator<Item = I>,
    P: Fn(&D) -> Option<String>,
    D: DeserializeOwned,
{
    paginate(
        client,
        init_request,
        results_accessor_function,
        pagination_token_accessor_function,
        |acc: &mut Vec<I>, new_results| acc.extend(new_results),
        limit,
    )
    .await
}
