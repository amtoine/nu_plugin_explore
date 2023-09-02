use nu_protocol::{ast::PathMember, Span};

/// a simplified [`PathMember`] that can be put in a single vector, without being too long
pub(crate) enum PM<'a> {
    // the [`PathMember::String`] variant
    S(&'a str),
    // the [`PathMember::Int`] variant
    I(usize),
}

pub(crate) fn to_path_member_vec(cell_path: &[PM]) -> Vec<PathMember> {
    cell_path
        .iter()
        .map(|x| match *x {
            PM::S(val) => PathMember::String {
                val: val.into(),
                span: Span::test_data(),
                optional: false,
            },
            PM::I(val) => PathMember::Int {
                val,
                span: Span::test_data(),
                optional: false,
            },
        })
        .collect::<Vec<_>>()
}

impl<'a> PM<'a> {
    pub(crate) fn as_cell_path(members: &[Self]) -> String {
        format!(
            "$.{}",
            members
                .iter()
                .map(|m| {
                    match m {
                        Self::I(val) => val.to_string(),
                        Self::S(val) => val.to_string(),
                    }
                })
                .collect::<Vec<String>>()
                .join(".")
        )
    }
}
