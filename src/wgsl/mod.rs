use nom::IResult;
use std::error::Error;

mod primitives;

pub struct WgslSpec {}

impl WgslSpec {
    pub fn from_download() -> Result<Self, Box<dyn Error>> {
        Self::from_bs_url("https://raw.githubusercontent.com/gpuweb/gpuweb/main/wgsl/index.bs")
    }

    pub fn from_bs_url(bs_url: &str) -> Result<Self, Box<dyn Error>> {
        let text = crate::misc::download_text(bs_url)?;
        let (_, spec) = WgslSpec::parse_bs(&text).map_err(|x| x.to_owned())?;
        Ok(spec)
    }

    pub fn parse_bs(bs: &str) -> IResult<&str, Self> {
        todo!()
    }
}
