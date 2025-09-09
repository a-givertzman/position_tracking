use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use frdm_tools::{camera::Camera, AutoBrightnessAndContrast, AutoBrightnessAndContrastCtx, AutoGamma, AutoGammaCtx, ContextRead, Cropping, Eval, EvalResult, Image, Initial, InitialCtx, ResultCtx};
use opencv::prelude::{DescriptorMatcherTrait, Feature2DTrait};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object}, Service, ServiceWaiting, RECV_TIMEOUT}, sync::Handles, thread_pool::Scheduler};

use crate::modules::{BfMatch, CameraServiceConf, GrayScale, TemplateMatch};

/// 
/// Dects defect on the frames coming from the camera
pub struct CameraService {
    name: Name,
    conf: CameraServiceConf,
    template: Image,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl CameraService {
    ///
    /// Crteates [CameraService] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: CameraServiceConf,
        template: Image,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "CameraService");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            template,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Create opencv Ui windows
    fn setup_opencv_windows(dbg: &Dbg, keys: Vec<impl Into<String>>) {
        for key in keys {
            if let Err(err) = opencv::highgui::named_window(&key.into(), opencv::highgui::WINDOW_NORMAL) {
                log::warn!("{}.stream | Create Window Error: {}", dbg, err);
            }
        }
        // opencv::highgui::wait_key(1).unwrap();
    }
    ///
    /// Processing an image
    fn process(dbg: &Dbg, window: &str, window_src: &str, window_gamma: &str, window_abc: &str, templ_match: &impl Eval<Image, EvalResult>, frame: &Image) {
        log::info!("{dbg}.process | Source frame...");
        opencv::highgui::imshow(window_src, &frame.mat).unwrap();
        opencv::highgui::wait_key(1).unwrap();
        log::info!("{dbg}.process | Calculations...");
        match templ_match.eval(frame.clone()) {
            Ok(ctx) => {
                log::info!("{dbg}.process | Calculations - Ok");
                
                let gamma: &AutoGammaCtx = ctx.read();
                log::info!("{dbg}.process | Gamma frame...");
                opencv::highgui::imshow(window_gamma, &gamma.result.mat).unwrap();
                opencv::highgui::wait_key(1).unwrap();

                let abc: &AutoBrightnessAndContrastCtx = ctx.read();
                log::info!("{dbg}.process | ABC frame...");
                opencv::highgui::imshow(window_abc, &abc.result.mat).unwrap();
                opencv::highgui::wait_key(1).unwrap();
                
                let result: &ResultCtx = ctx.read();
                log::info!("{dbg}.process | Result frame...");
                opencv::highgui::imshow(window, &result.frame.mat).unwrap();
                opencv::highgui::wait_key(1).unwrap();
            }
            Err(err) => log::info!("{dbg}.run | Template match error: {:?}", err),
        };
    }
}
//
//
impl Object for CameraService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for CameraService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CameraService")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
// 
impl Service for CameraService {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let template = self.template.clone();
        let window = format!("Matching result");
        let window_src = format!("Source frame");
        let window_gamma = format!("Auto gamma frame");
        let window_abc = format!("BrightnessContrast");
        let exit = self.exit.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let handles_clone = self.handles.clone();
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            Self::setup_opencv_windows(&dbg, vec![&window, &window_src, &window_gamma, &window_abc]);
            // let mut template_gray = opencv::core::Mat::default();
            // opencv::imgproc::cvt_color(&template.mat, &mut template_gray, opencv::imgproc::COLOR_BGR2GRAY, 0)
            //     .map_err(|err| Error::new(dbg, "template to gray error").pass(err.to_string()))?;
            let templ_match = BfMatch::new(
                conf.template_match.method,
                conf.template_match.threshold,
                template,
                AutoBrightnessAndContrast::new(
                    conf.image.brightness_contrast.hist_clip_left,
                    conf.image.brightness_contrast.hist_clip_right,
                    AutoGamma::new(
                        conf.image.gamma.factor,
                        // Cropping::new(
                        //     conf.image.cropping.x,
                        //     conf.image.cropping.width,
                        //     conf.image.cropping.y,
                        //     conf.image.cropping.height,
                        // )
                        Initial::new(
                            InitialCtx::new(),
                        ),
                    ),
                ),
                // GrayScale::new(
                // ),
            );
            let mut camera = Camera::new(conf.camera.clone());
            match &conf.camera.from_path {
                Some(path) => {
                    log::info!("{dbg}.run | Starting camera from path '{path}'...");
                    let frames = camera.from_images(path).unwrap();
                    service_release.add(Ok(()));
                    for frame in frames {
                        Self::process(&dbg, &window, &window_src, &window_gamma, &window_abc, &templ_match, &frame);
                        // match templ_match.eval(frame) {
                        //     Err(err) => log::info!("{dbg}.run | Template match error: {:?}", err),
                        // }
                        std::thread::sleep(Duration::from_millis(2000));
                    }
                }
                None => {
                    let camera_stream = camera.stream();
                    service_release.add(Ok(()));
                    'main: loop {
                        log::debug!("{dbg}.run | Starting camera...");
                        match camera.read() {
                            Ok(handle) => {
                                log::debug!("{dbg}.run | Starting camera - Ok");
                                handles_clone.push(handle);
                                log::debug!("{dbg}.run | Receiving frames from camera...");
                                'camera: loop {
                                    match camera_stream.recv_timeout(RECV_TIMEOUT) {
                                        Ok(frame) => Self::process(&dbg, &window, &window_src, &window_gamma, &window_abc, &templ_match, &frame),
                                        Err(err) => {
                                            match err {
                                                kanal::ReceiveErrorTimeout::Timeout => {}
                                                _ => {
                                                    break 'camera;
                                                }
                                            }
                                        }
                                    }
                                    if exit.load(Ordering::Acquire) {
                                        break 'main;
                                    }
                                }
                                camera.exit();
                            }
                            Err(err) => log::info!("{dbg}.run | Camera error: {:?}", err),
                        }
                    }
                    camera.exit();
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        });
        match handle {
            Ok(handle) => {
                self.handles.push(handle);
                let r = match conf.wait_started {
                    Some(_) => {
                        log::info!("{}.run | Waiting while starting...", self.dbg);
                        service_waiting.wait()
                    }
                    None => Ok(()),
                };
                log::info!("{}.run | Starting - ok", self.dbg);
                r
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }
}
