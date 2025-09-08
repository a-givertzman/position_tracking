use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};

///
/// Configuration parameters for template matching algorithm
/// 
/// Conf Example:
/// ```yaml
///     threshold: 0.8
///     method: TM_CCOEFF_NORMED    # TM_CCOEFF_NORMED or TM_CCORR_NORMED recomended, 
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateMatchConf {
    pub name: Name,
    pub template: String,
    pub method: opencv::imgproc::TemplateMatchModes,
    pub threshold: f64,
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
        let threshold: f64 = conf.get("threshold").expect(&format!("{dbg}.new | 'threshold' - not found or wrong configuration"));
        log::trace!("{}.new | threshold: {:?}", dbg, threshold);
        Self {
            name,
            template,
            method,
            threshold,
        }
    }
    ///
    /// Returns `TemplateMatchModes` parsed from string
    fn template_match_modes_from_str(method: &str) -> Result<opencv::imgproc::TemplateMatchModes, Error> {
        match method.to_lowercase().as_str() {
            "TM_SQDIFF" => Ok(opencv::imgproc::TemplateMatchModes::TM_SQDIFF),
            "TM_SQDIFF_NORMED" => Ok(opencv::imgproc::TemplateMatchModes::TM_SQDIFF_NORMED),
            "TM_CCORR" => Ok(opencv::imgproc::TemplateMatchModes::TM_CCORR),
            "TM_CCORR_NORMED" => Ok(opencv::imgproc::TemplateMatchModes::TM_CCORR_NORMED),
            "TM_CCOEFF" => Ok(opencv::imgproc::TemplateMatchModes::TM_CCOEFF),
            "TM_CCOEFF_NORMED" => Ok(opencv::imgproc::TemplateMatchModes::TM_CCOEFF_NORMED),
            _ => Err(Error::new("TemplateMatchConf", "template_match_modes_from_str").err(format!("Unknown method {}", method))),
        }
    }
}
