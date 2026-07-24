//! Arguments for `uncommit`.

#![deny(missing_docs)]

use crate::args::atoms::CliIdArg;

/// Uncommit changes from commits or committed files to the uncommitted area.
#[derive(Debug, clap::Parser)]
#[cfg_attr(feature = "raw-clap-docs", clap(verbatim_doc_comment))]
pub struct Platform {
    /// TODO
    #[clap(required = true)]
    pub sources: Vec<CliIdArg>,
}
