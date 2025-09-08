use std::time::Duration;
use frdm_tools::camera::CameraConf;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};

use crate::modules::{ImageConf, TemplateMatchConf};

///
/// ## The configuration parameters for the `RopeDefect`
/// 
/// ### Conf example
/// ```yaml
/// service CameraService:
///     wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///     camera Camera1:
///         fps: Max                    # Max / Min / 30.0
///         resolution: 
///             width: 1200
///             height: 800
///         index: 0
///         # address: 192.168.10.12:2020
///         # Mono8/10/12/16, Bayer8/10/12/16, RGB8, BGR8, YCbCr8, YCbCr411, YUV422, YUV411 | Default and fastest BayerRG8
///         # pixel-format:  Mono8
///         # pixel-format:  BayerRG8
///         # pixel-format:  QOI_Mono8
///         pixel-format:  QOI_BayerRG8
///         exposure:
///             auto: Off                   # Off / Continuous
///             time: 26000                   # microseconds
///         auto-packet-size: true          # StreamAutoNegotiatePacketSize
///         channel-packet-size: Max        # Maximizing packet size increases frame rate
///         resend-packet: true             # StreamPacketResendEnable
///     image:
///         cropping:
///             x: 10           # new left edge
///             width: 1900     # new image width
///             y: 10           # new top edge
///             height: 1180    # new image height
///         gamma:
///             factor: 95.0            # percent of influence of [AutoGamma] algorythm bigger the value more the effect of [AutoGamma] algorythm, %
///         brightness-contrast:
///             hist-clip-left: 1.0     # optional histogram clipping from right, default = 0.0 %
///             hist-clip-right: 1.0    # optional histogram clipping from right, default = 0.0 %
///         gausian:
///             blur-size:              # blur radius
///                 width: 3
///                 height: 3
///             sigma-x: 0.0
///             sigma-y: 0.0
///         sobel:
///             kernel-size: 3
///             scale: 1.0
///             delta: 0.0
///         overlay:
///             src1-weight: 0.5
///             src2-weight: 0.5
///             gamma: 0.0
///     template-match:
///         threshold: 0.8
///         method: TM_CCOEFF_NORMED    # TM_CCOEFF_NORMED or TM_CCORR_NORMED recomended, 
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CameraServiceConf {
    pub name: Name,
    /// Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    pub image: ImageConf,
    pub template_match: TemplateMatchConf,
    pub camera: CameraConf,
}
//
// 
impl CameraServiceConf {
    ///
    /// Returns [CameraServiceConf] built from `ConfTree`:
    pub fn new(
        parent: impl Into<String>,
        conf: ConfTree,
    ) -> Self {
        let parent = parent.into();
        let me = "CameraServiceConf";
        let dbg = Dbg::new(&parent, me);
        let name = Name::new(parent, me);
        // log::debug!("{dbg}.new | conf: {:#?}", conf);
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let image = conf.get("image").expect(&format!("{dbg}.new | 'image' - not found or wrong configuration"));
        let image = ImageConf::new(&dbg, image);
        log::trace!("{}.new | image: {:?}", dbg, image);
        let template_match = conf.get("template-match").expect(&format!("{dbg}.new | 'template-match' - not found or wrong configuration"));
        let template_match = TemplateMatchConf::new(&dbg, template_match);
        log::trace!("{}.new | template-match: {:?}", dbg, template_match);
        let camera = conf.get("camera").expect(&format!("{dbg}.new | 'camera' - not found or wrong configuration"));
        let camera = CameraConf::new(&name, &camera);
        log::trace!("{dbg}.new | camera: {:#?}", camera);
        Self {
            name,
            wait_started,
            image,
            template_match,
            camera
        }
    }
}
///
/// Camera unique identifier to be used in the sql database and folder name
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraId(pub usize);
// Implement the Default trait to provide a default value
impl Default for CameraId {
    fn default() -> Self {
        CameraId(0) // Default value for the wrapped usize
    }
}

// Implement Deref to allow immutable dereferencing to usize
impl std::ops::Deref for CameraId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0 // Dereference to the inner usize
    }
}

// Implement DerefMut to allow mutable dereferencing to usize
impl std::ops::DerefMut for CameraId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0 // Mutably dereference to the inner usize
    }
}
