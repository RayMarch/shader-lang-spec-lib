use crate::nom_prelude::*;
use std::error::Error;

use self::{parametrization::OverloadRow, primitives::FnDecl};

pub mod parametrization;
pub mod primitives;

pub struct WgslSpec {
    pub text: String,
    pub fns: Vec<FnDecl>,
    pub overloads: Vec<OverloadRow>,
}

impl WgslSpec {
    pub fn from_download() -> Result<Self, Box<dyn Error>> {
        Self::from_bs_url("https://raw.githubusercontent.com/gpuweb/gpuweb/main/wgsl/index.bs")
    }

    pub fn from_bs_url(bs_url: &str) -> Result<Self, Box<dyn Error>> {
        let text = crate::misc::download_text(bs_url)?;
        let (_, spec) = WgslSpec::parse_bs(&text).map_err(|x| x.report_into_string(&text))?;
        Ok(spec)
    }

    pub fn parse_bs(i: &str) -> NomResult<&str, Self> {
        let text = i.to_string();
        let (s, fns) = many0(preceded(take_until_matches(FnDecl::parse), FnDecl::parse))(i)?;
        let (s, overloads) = many0(preceded(
            take_until_matches(OverloadRow::parse),
            OverloadRow::parse,
        ))(i)?;
        Ok((
            s,
            WgslSpec {
                text,
                overloads,
                fns,
            },
        ))
    }
}
