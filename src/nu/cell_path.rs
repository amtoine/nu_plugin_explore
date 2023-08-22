use nu_protocol::{ast::PathMember, Span};

#[allow(dead_code)]
/// a simplified [`PathMember`] that can be put in a single vector, without being too long
pub(crate) enum PM<'a> {
    // the [`PathMember::String`] variant
    S(&'a str),
    // the [`PathMember::Int`] variant
    I(usize),
}

#[allow(dead_code)]
pub(crate) fn to_path_member_vec(cell_path: Vec<PM>) -> Vec<PathMember> {
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
