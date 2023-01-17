use nom::multi::many1;

use super::primitives::*;
use crate::{fn_name, nom_prelude::*};

pub fn parse_generic_arg(s: &str) -> NomResult<&str, Ident> {
    context(
        fn_name!(),
        delimited(
            ws0_then(tag("<var ignore>")),
            cut(ws0_then(alt((
                delimited(tag("|"), ws0_then(Ident::parse), tag("|")),
                ws0_then(Ident::parse),
            )))),
            ws0_then(tag("</var>")),
        ),
    )(s)
}

pub fn parse_trait_name(s: &str) -> NomResult<&str, String> {
    context(
        fn_name!(),
        map(
            delimited(
                tag("[="),
                cut(take_while1(|c: char| {
                    c.is_whitespace() || c.is_alphanumeric()
                })),
                tag("=]"),
            ),
            ToString::to_string,
        ),
    )(s)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionBound {
    is_one_of: Vec<Ty>,
}

impl UnionBound {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        //[=i32=], [=u32=], or [=f32=]
        let parser = separated_list1(
            alt((
                ws0_then(recognize(pair(tag(","), delimited(ws0, tag("or"), ws1)))),
                ws0_then(tag(",")),
                ws0_then(terminated(tag("or"), ws1)),
            )),
            ws0_then(alt((
                context("[=ty=]", delimited(tag("[="), cut(Ty::parse), tag("=]"))),
                context("`ty`", delimited(tag("`"), Ty::parse, tag("`"))),
            ))),
        );
        map(context(fn_name!(), parser), |is_one_of| UnionBound {
            is_one_of,
        })(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitBound {
    is_a: String,
}

impl TraitBound {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        let parser = ws1_then(parse_trait_name);
        map(context(fn_name!(), parser), |is_a| TraitBound { is_a })(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundKind {
    Union(UnionBound),
    Trait(TraitBound),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
    type_param: Ident,
    bound_kind: BoundKind,
}

impl Bound {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        // the "a" | "an" word before a trait bound, e.g. "T is a [=texel format=]"
        let a_an = ws0_then(alt((tag("an"), tag("a"))));

        let parser = separated_pair(
            ws0_then(parse_generic_arg),
            delimited(ws0, tag("is"), ws1),
            ws0_then(alt((
                preceded(a_an, cut(map(TraitBound::parse, BoundKind::Trait))),
                map(UnionBound::parse, BoundKind::Union),
            ))),
        );
        map(context(fn_name!(), parser), |(type_param, bound_kind)| {
            Bound {
                type_param,
                bound_kind,
            }
        })(s)
    }
}

/*
    case0:
    <td><var ignore>A</var> is [=i32=], or [=u32=]<br>
        <var ignore>L</var> is [=i32=], or [=u32=]
    <td>

    case1:
    <tr algorithm="textureSampleLevel 3d">
    <td><var ignore>T</var> is `texture_3d<f32>`, or `texture_cube<f32>`
    <td>

    case2:
     <td>|F| is a [=texel format=]<br>
        <var ignore>C</var> is [=i32=], or [=u32=]<br>
        <var ignore>A</var> is [=i32=], or [=u32=]<br>
        <var ignore>CF</var> depends on the storage texel format |F|.
        [See the texel format table](#storage-texel-formats) for the mapping of texel
        format to channel format.
    <td>

    case3:
    <tr algorithm="textureDimensions 1d">
    <td><var ignore>ST</var> is [=i32=], [=u32=], or [=f32=]<br>
        <var ignore>F</var> is a [=texel format=]<br>
        <var ignore>A</var> is an [=access mode=]<br><br>
        |T| is `texture_1d<ST>` or `texture_storage_1d<F,A>`
    <td>
*/
// <tr algorithm="textureSampleLevel 2d array">
//     <td><var ignore>A</var> is [=i32=], or [=u32=]
//     <td><xmp highlight=rust>
// fn textureSampleLevel(t: texture_2d_array<f32>,
//                       s: sampler,
//                       coords: vec2<f32>,
//                       array_index: A,
//                       level: f32) -> vec4<f32></xmp>

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parametrization(Vec<Bound>);

impl Parametrization {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        let parser = many1(terminated(
            ws0_then(Bound::parse),
            many0(ws0_then(tag("<br>"))),
        ));
        map(context(fn_name!(), parser), Parametrization)(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadRow {
    pub algorithm_attr: String,
    pub parametrization: Parametrization,
    pub fn_decl: FnDecl,
}

impl OverloadRow {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        let parse_tr = delimited(
            tag("<tr algorithm=\""),
            take_till(|c: char| c == '"'),
            tag("\">"),
        );
        let parse_param = Parametrization::parse;
        let parse_decl = delimited(
            tag("<xmp highlight=rust>"),
            ws0_then(FnDecl::parse),
            ws0_then(tag("</xmp>")),
        );

        let parser = tuple((
            map(parse_tr, |s: &str| s.to_string()),
            ws0_then(preceded(tag("<td>"), ws0_then(parse_param))),
            ws0_then(preceded(tag("<td>"), ws0_then(parse_decl))),
        ));

        map(
            context(fn_name!(), parser),
            |(algorithm_attr, parametrization, fn_decl)| OverloadRow {
                algorithm_attr,
                parametrization,
                fn_decl,
            },
        )(s)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_overload_row() {
        let str = r#"<tr algorithm="textureSampleLevel 2d array">
    <td><var ignore>A</var> is [=i32=], or [=u32=]<br>
    <var ignore>X</var> is [=i32=] or [=u32=]
    <var ignore>Y</var> is [=i32=], [=u32=] or [=f32=]
    <td><xmp highlight=rust>
        fn textureSampleLevel(t: texture_2d_array<f32>,
                            s: sampler,
                            coords: vec2<f32>,
                            array_index: A,
                            level: f32) -> vec4<f32></xmp>"#;
        assert!(OverloadRow::parse(str).is_ok());
    }
}
