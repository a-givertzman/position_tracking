use std::time::Instant;
use sal_core::error::Error;
use frdm_tools::{
    ContextWrite, EvalResult,
    ResultCtx, ContextRead,
    Eval, Image,
};
///
/// Takes source [Image]
/// Return gray scale [Image]
pub struct GrayScale {
    ctx: Box<dyn Eval<Image, EvalResult>>,
}
impl GrayScale {
    ///
    /// Returns [GrayScale] new instance
    pub fn new(ctx: impl Eval<Image, EvalResult> + 'static) -> Self {
        Self { 
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Image, EvalResult> for GrayScale {
    fn eval(&self, frame: Image) -> EvalResult {
        let error = Error::new("GrayScale", "eval");
        match self.ctx.eval(frame) {
            Ok(ctx) => {
                // build a lookup table mapping the pixel values [0, 255] to
                // their adjusted gamma values
                let t = Instant::now();
                let result: &ResultCtx = ctx.read();
                let frame = &result.frame;
                let mut gray = opencv::core::Mat::default();
                match opencv::imgproc::cvt_color(&frame.mat, &mut gray, opencv::imgproc::COLOR_BGR2GRAY, 0) {
                    Ok(_) => {
                        let frame = Image {
                            width: frame.width,
                            height: frame.height,
                            timestamp: frame.timestamp,
                            mat: gray,
                            bytes: frame.bytes,
                        };
                        let result = ResultCtx { frame };
                        log::debug!("GrayScale.eval | Elapsed: {:?}", t.elapsed());
                        ctx.write(result)
                    }
                    Err(err) => Err(error.pass(err.to_string())),
                }
            }
            Err(err) => Err(error.pass(err)),
        }
    }
}
