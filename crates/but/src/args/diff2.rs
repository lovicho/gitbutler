use crate::args::atoms::CliIdArg;

#[derive(Debug, clap::Parser)]
#[cfg_attr(feature = "raw-clap-docs", clap(verbatim_doc_comment))]
pub struct Platform {
    pub target: Option<CliIdArg>,
}
