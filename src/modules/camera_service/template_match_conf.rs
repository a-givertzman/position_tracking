use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};

///
/// Configuration parameters for template matching algorithm
/// 
/// Conf Example:
/// ```yaml
///     match-ratio: 0.8
///     method: TM_CCOEFF_NORMED    # TM_CCOEFF_NORMED or TM_CCORR_NORMED recomended, 
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateMatchConf {
    pub name: Name,
    pub template: String,
    pub method: opencv::imgproc::TemplateMatchModes,
    pub match_ratio: f64,
}
//
// 
impl TemplateMatchConf {
    ///
    /// Returns [TemplateMatchConf] built from `ConfTree`:
    pub fn new(
        parent: impl Into<String>,
        conf: ConfTree,
    ) -> Self {
        let parent = parent.into();
        let me = "TemplateMatchConf";
        let dbg = Dbg::new(&parent, me);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let template: String = conf.get("template").expect(&format!("{dbg}.new | 'template' - not found or wrong configuration"));
        log::trace!("{}.new | template: {:?}", dbg, template);
        let method: String = conf.get("method").expect(&format!("{dbg}.new | 'method' - not found or wrong configuration"));
        let method = Self::template_match_modes_from_str(&method).expect(&format!("{dbg}.new | Unknown 'method' {method}"));
        log::trace!("{}.new | method: {:?}", dbg, method);
        let match_ratio: f64 = conf.get("match-ratio").expect(&format!("{dbg}.new | 'match-ratio' - not found or wrong configuration"));
        log::trace!("{}.new | match-ratio: {:?}", dbg, match_ratio);
        Self {
            name,
            template,
            method,
            match_ratio,
        }
    }
    ///
    /// Returns `TemplateMatchModes` parsed from string
    fn template_match_modes_from_str(method: &str) -> Result<opencv::imgproc::TemplateMatchModes, Error> {
        match method.to_lowercase().as_str() {
            "tm_sqdiff" => Ok(opencv::imgproc::TemplateMatchModes::TM_SQDIFF),
            "tm_sqdiff_normed" => Ok(opencv::imgproc::TemplateMatchModes::TM_SQDIFF_NORMED),
            "tm_ccorr" => Ok(opencv::imgproc::TemplateMatchModes::TM_CCORR),
            "tm_ccorr_normed" => Ok(opencv::imgproc::TemplateMatchModes::TM_CCORR_NORMED),
            "tm_ccoeff" => Ok(opencv::imgproc::TemplateMatchModes::TM_CCOEFF),
            "tm_ccoeff_normed" => Ok(opencv::imgproc::TemplateMatchModes::TM_CCOEFF_NORMED),
            _ => Err(Error::new("TemplateMatchConf", "template_match_modes_from_str").err(format!("Unknown method {}", method))),
        }
    }
}
