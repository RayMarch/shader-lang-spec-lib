use std::fmt::Display;

use derive_deref::Deref;
use nom::{
    bytes::complete::{take_until, take_until1},
    multi::{many1, many_till},
};

use super::fn_decl::*;
use crate::{fn_name, misc::normalize_whitespace, nom_prelude::*};

pub fn parse_generic_arg(s: &str) -> NomResult<&str, Ident> {
    let ident = || {
        alt((
            delimited(tag("|"), cut(ws0_then(Ident::parse)), tag("|")),
            ws0_then(Ident::parse),
        ))
    };

    context(
        fn_name!(),
        alt((
            delimited(
                ws0_then(tag("<var ignore>")),
                cut(ws0_then(ident())),
                ws0_then(tag("</var>")),
            ),
            ws0_then(ident()),
        )),
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

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct UnionBound {
    is_one_of: Vec<Ty>,
}

impl Display for UnionBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "is ")?;
        for (i, ty) in self.is_one_of.iter().enumerate() {
            let or = if i + 1 != self.is_one_of.len() {
                " | "
            } else {
                ""
            };
            write!(f, "{ty}{or}")?;
        }
        Ok(())
    }
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

impl Display for TraitBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "is a `{}`", self.is_a)
    }
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
    /// a bound described in plain text
    Prose(String),
}

impl Display for BoundKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundKind::Union(b) => write!(f, "{b}"),
            BoundKind::Trait(b) => write!(f, "{b}"),
            BoundKind::Prose(s) => write!(f, "\"{s}\""),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
    pub type_param: Ident,
    pub bound_kind: BoundKind,
}

impl Display for Bound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.type_param, self.bound_kind)
    }
}

impl Bound {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        let prose = take_until_matches(alt((tag("<br>"), tag("<td>"))));
        let prose = map(prose, |s: &str| {
            BoundKind::Prose(normalize_whitespace(s.trim()))
        });
        let prose = context(stringify!(BoundKind::Prose), prose);

        // the "a" | "an" word before a trait bound, e.g. "T is a [=texel format=]"
        let a_an = ws0_then(alt((tag("an"), tag("a"))));
        let is = || delimited(ws0, tag("is"), ws1);

        let parser = pair(
            ws0_then(parse_generic_arg),
            ws0_then(alt((
                preceded(
                    is(),
                    preceded(a_an, cut(map(TraitBound::parse, BoundKind::Trait))),
                ),
                preceded(is(), map(UnionBound::parse, BoundKind::Union)),
                prose,
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

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct Parametrization(pub Vec<Bound>);

impl Parametrization {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        let parser = many1(terminated(
            ws0_then(Bound::parse),
            many0(ws0_then(tag("<br>"))),
        ));
        map(context(fn_name!(), parser), Parametrization)(s)
    }
}

impl Display for Parametrization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, bound) in self.iter().enumerate() {
            let comma = if i + 1 != self.len() { "," } else { "" };
            writeln!(f, "    {bound}{comma}")?;
        }
        Ok(())
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

impl Display for OverloadRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[{}]", self.algorithm_attr);
        write!(f, "{}", self.fn_decl)?;
        writeln!(f, " where\n{};", self.parametrization)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_overload_row() {
        let str = "
        <var ignore>CF</var> depends on the storage texel format |F|.
        [See the texel format table](#storage-texel-formats) for the mapping of texel
        format to channel format.
        <td>";

        Parametrization::parse(str).report(str);

        let str = r#"<tr algorithm="textureSampleLevel 2d array">
    <td><var ignore>A</var> is [=i32=], or [=u32=]<br>
    <var ignore>X</var> is [=i32=] or [=u32=]
    <var ignore>Y</var> is [=i32=], [=u32=] or [=f32=]
    <var ignore>CF</var> depends on the storage texel format |F|.
        [See the texel format table](#storage-texel-formats) for the mapping of texel
        format to channel format.
    <td><xmp highlight=rust>
        fn textureSampleLevel(t: texture_2d_array<f32>,
                            s: sampler,
                            coords: vec2<f32>,
                            array_index: A,
                            level: f32) -> vec4<f32></xmp>"#;
        OverloadRow::parse(str).report(str);

        OverloadRow::parse(str).report(str);
    }
}
