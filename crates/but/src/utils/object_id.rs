use gix::prelude::ObjectIdExt as _;

/// Shorten a commit object id using repository disambiguation (`core.abbrev`), and return
/// the result as a hex-string.
pub fn shorten_object_id(repo: &gix::Repository, oid: impl Into<gix::ObjectId>) -> String {
    oid.into().attach(repo).shorten_or_id().to_string()
}

/// Try to shorten a hex object id string using repository disambiguation.
///
/// If `hex` cannot be parsed as an object id, return it unchanged.
pub fn shorten_hex_object_id(repo: &gix::Repository, hex: &str) -> String {
    gix::ObjectId::from_hex(hex.as_bytes())
        .map(|oid| shorten_object_id(repo, oid))
        .unwrap_or_else(|_| hex.to_owned())
}
