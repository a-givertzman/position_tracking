use std::time::Instant;

use frdm_tools::{ContextRead, ContextWrite, Eval, EvalResult, Image, ResultCtx};
use opencv::core::Mat;
use sal_core::error::Error;

///
/// # Detection of the template on the input frame
/// 
/// All of the template matching modes can be classified roughly as a dense (meaning pixel-wise) similarity metric,
/// or equivalently but inversely, a distance metric between images.
/// Generally, you will have two images and you want to compare them in some way. Off the bat, template matching doesn't directly
/// help you match things that are scaled, rotated, or warped. Template matching is strictly concerned with measuring the
/// similarity of two images exactly as they appear. However, the actual metrics used here are used everywhere in computer vision,
/// including finding transformations between images...just usually there's more complex steps going on in addition
/// (like gradient descent to find the optimal transformation parameters).
/// There are many choices for distance metrics, and they generally have pros and cons depending on the application.
/// 
/// ## Sum of absolute differences (SAD)
/// 
/// For a first start, the most basic distance metric is just the absolute difference between two values, i.e. d(x, y) = abs(x - y).
/// For images, an easy way to extend this from single values is just to sum all of these distances, pixel-wise, leading to the sum of absolute
/// differences (SAD) metric; it is also known as the Manhattan or the taxicab distance, and defines the L1 norm. Annoyingly, this isn't implemented
/// as one of OpenCV's template matching modes, but it's still important in this discussion as a comparison to SSD.
/// In the template matching scenario, you slide a template along multiple places and simply find where the smallest difference occurs.
/// It is the equivalent to asking what the index of the closest value to 5 is in the array [1, 4, 9]. You take the absolute difference of
/// each value in the array with 5, and index 1 has the smallest difference, so that's the location of the closest match. Of course in template
/// matching the value isn't 5 but an array, and the image is a larger array.
/// 
/// ## Sum of square differences (SSD): TM_SQDIFF
/// 
/// An interesting feature of the SAD metric is that it doesn't penalize really big differences any more than a bunch of really small differences.
/// Let's say we want to compute d(a, b) and d(a, c) with the following vectors:
/// 
/// a = [1, 2, 3]
/// b = [4, 5, 6]
/// c = [1, 2, 12]
/// 
/// Taking the sums of absolute differences element-wise, we see
/// 
/// SAD(a, b) = 3 + 3 + 3 = 9 = 0 + 0 + 9 = SAD(a, c)
/// 
/// In some applications, maybe that doesn't matter. But in other applications, you might want these two distances to actually be quite different.
/// Squaring the differences, instead of taking their absolute value, penalizes values that are further from what you expect---it makes the images
/// more distant as the difference in value grows. It maps more to how someone might explain an estimate as being way off, even if in value it's
/// not actually that distant. The sum of square differences (SSD) is equivalent to the squared Euclidean distance, the distance function for the
/// L2 norm. With SSD, we see our two distances are now quite different:
/// 
/// SSD(a, b) = 3^2 + 3^2 + 3^2 = 27 != 81 = 0^2 + 0^2 + 9^2 = SSD(a, c)
/// 
/// You may see that the L1 norm is sometimes called a robust norm. This is specifically because a single point of error won't grow the
/// distance more than the error itself. But of course with SSD, an outlier will make the distance much larger. So if your data is somewhat
/// prone to a few values that are very distant, note that SSD is probably not a good similarity metric for you. A good example might be
/// comparing images that may be overexposed. In some part of the image, you may just have white sky where the other is not white at all,
/// and you'll get a massive distance between images from that.
/// 
/// Both SAD and SSD have a minimum distance of 0, when the two images compared are identical. They're both always non-negative since the
/// absolute differences or square differences are always non-negative.
/// 
/// ## Cross correlation (CC): TM_CCORR
/// 
/// SAD and SSD are both generally discrete metrics---so they're a natural consideration for sampled signals, like images.
/// Cross correlation however is applicable as well to continuous, and therefore analog, signals, which is part of its ubiquity
/// in signal processing. With signals broadly, trying to detect the presence of a template inside a signal is known as a matched filter,
/// and you can basically think of it as the continuous analog of template matching.
/// Cross correlation just multiplies the two images together. You can imagine that if the two signals line up exactly, multiplying them
/// together will simply square the template. If they're not lined up just-so, then the product will be smaller. So, the location where
/// the product is maximized is where they line up the best. However, there is a problem with cross correlation in the case when you're
/// using it as a similarity metric of signals you're not sure are related, and that is usually shown in the following example.
/// Suppose you have three arrays:
/// 
/// a = [2, 600, 12]
/// b = [v, v, v]
/// c = [2v, 2v, 2v]
/// 
/// Broadly, there's no obvious correlation between a and b nor a and c. And generally, a shouldn't correlate any more to b than to c.
/// But, it's a product, and thus ccorr(a, c) = 2*ccorr(a, b). So, thats not ideal for trying to find a template inside a larger image.
/// And because we're dealing with discrete digital signals that have a defined maximum value (images), that means that a bright white
/// patch of the image will basically always have the maximum correlation. Because of this issues, TM_CCORR is not particularly useful
/// as a template matching method.
/// 
/// ## Mean shifted cross correlation (Pearson correlation coefficient): TM_CCOEFF
/// 
/// One simple way to solve the problem of correlating with bright patches is to simply subtract off the mean before comparing the signals.
/// That way, signals that are simply shifted have the same correlation as those that are unshifted. And this makes sense with our intuition---signals
/// that vary together are correlated.
/// 
/// ## Normalization: TM_SQDIFF_NORMED, TM_CCORR_NORMED, TM_CCOEFF_NORMED
/// 
/// All of the methods in OpenCV are normalized the same. The point of normalization is not to give a confidence/probability, but to give a metric
/// that you can compare against templates of different sizes or with values at different scales. For example, let's say we want to find if an object
/// is in an image, and we have two different templates of this object. The two different templates are different sizes. We could just normalize by
/// the number of pixels, which would work to compare templates of different sizes. However, say my templates are actually quite different in intensities,
/// like one has much higher variance of the pixel values than the other. Typically, what you'd do in this case is divide by the standard deviation
/// (square root of the sum of squared differences from the mean). OpenCV does do this with the TM_CCOEFF_NORMED method, since the squared sum of the
/// mean differences is the variance, but the other methods aren't mean shifted, so the scaling is just a measure of sum of the image values.
/// Either way, the result is similar, you want to scale by something that relates to the intensity of the image patches used.
/// 
/// ## Other metrics
/// 
/// There are other useful metrics that OpenCV does not provide. Matlab provides SAD, as well as the maximum absolute difference metric (MaxAD),
/// which is also known as the uniform distance metric and gives the L∞ norm. Basically, you take the max absolute difference instead of the sum of them.
/// Other metrics that are used are typically seen in optimization settings, for example the enhanced correlation coefficient which was first proposed
/// for stereo matching, and then later expanded for alignment in general. That method is used in OpenCV, but not for template matching; you'll find
/// the ECC metric in computeECC() and findTransformECC().
/// 
/// ## Which method to use?
/// 
/// Most often, you will see normed and un-normed SSD (TM_SQDIFF_NORMED, TM_SQDIFF), and zero-normalized cross-correlation / ZNCC (TM_CCOEFF_NORMED) used.
/// Sometimes you may see TM_CCORR_NORMED, but less often. According to some lecture notes I found online (some nice examples and intuition there on this topic!),
/// Trucco and Verri's CV book states that generally SSD works better than correlation, but I don't have T&V's book to see why they suggest that; presumably
/// the comparison is on real-world photographs. But despite that, SAD and SSD are definitely useful, especially on digital images.
/// 
/// I don't know of any definitive examples of one or the other being inherently better in most cases or something---I think it really depends on your
/// imagery and template. Generally I'd say: if you're looking for exact or very close to exact matches, use SSD. It is fast, and it definitely maps
/// to what you're trying to minimize (the difference between the template and image patch). There's no need to normalize in that case, it is just
/// added overhead. If you have similar requirements but need multiple templates to be comparable, then normalize the SSD. If you're looking for matches,
/// but you're working with real-world photographs that may have exposure or contrast differences,
/// the mean shifting and variance equalization from ZNCC will likely be the best.
/// 
/// As for picking the right threshold, the value from ZNCC or SSD is not a confidence or probability number at all. If you want to pick the right threshold,
/// you can measure the parameter in any number of typical ways. You can calculate ROC curves or PR curves for different thresholds. You can use regression
/// to find the optimal parameter. You'll need to label some data, but then at least you'll have measurements of how you're doing against some test set so
/// that your choice is not arbitrary. As usual with a data-filled field, you'll need to make sure your data is as close to real world examples as possible,
/// and that your test data covers your edge cases as well as your typical images.
/// 
pub struct TemplateMatch {
    method: opencv::imgproc::TemplateMatchModes,
    threshold: f64,
    template: Image,
    ctx: Box<dyn Eval<Image, EvalResult>>,
}
//
//
impl TemplateMatch {
    ///
    /// Returns [TemplateMatch] new instance
    /// - `threshold` - ...
    /// - `method` - TM_CCOEFF_NORMED or TM_CCORR_NORMED
    pub fn new(method: opencv::imgproc::TemplateMatchModes, threshold: f64, template: Image, ctx: impl Eval<Image, EvalResult> + 'static) -> Self {
        Self { 
            method,
            threshold,
            template,
            ctx: Box::new(ctx),
        }
    }
    ///
    /// Draws a bounding box around matched image segment
    fn draw_box(&self, mut frame: Image, rec: opencv::core::Rect) -> Result<Image, Error> {
        match opencv::imgproc::rectangle(
            &mut frame.mat,
            rec,
            opencv::core::VecN([0.0, 0.0, 255.0, 255.0]),
            3,
            opencv::imgproc::LineTypes::FILLED as i32,
            0,
        ) {
            Ok(_) => Ok(frame),
            Err(err) => Err(Error::new("me", "area").pass(err.to_string())),
        }
    }
}
//
//
impl Eval<Image, EvalResult> for TemplateMatch {
    fn eval(&self, src: Image) -> EvalResult {
        let error = Error::new("TemplateMatch", "eval");
        match self.ctx.eval(src.clone()) {
            Ok(ctx) => {
                let t = Instant::now();
                let result: &ResultCtx = ctx.read();
                let frame = &result.frame;
                let mut result = Mat::default();
                let mask = Mat::default();
                match opencv::imgproc::match_template(&frame.mat, &self.template.mat, &mut result, self.method as i32, &mask) {
                    Ok(_) => {
                        let mut min_val: f64 = 0.0;
                        let mut max_val: f64 = 0.0;
                        let mut min_loc = opencv::core::Point::new(0, 0);
                        let mut max_loc = opencv::core::Point::new(0, 0);
                        match opencv::core::min_max_loc(&result, Some(&mut min_val), Some(&mut max_val), Some(&mut min_loc), Some(&mut max_loc), &mask) {
                            Ok(_) => {
                                if max_val > self.threshold {
                                    let frame = self.draw_box(src, opencv::core::Rect::new(max_loc.x, max_loc.y, 30, 30))?;
                                    let result = ResultCtx { frame };
                                    log::debug!("TemplateMatch.eval | X: {}, Y: {}", max_loc.x, max_loc.y);
                                    log::debug!("TemplateMatch.eval | Elapsed: {:?}", t.elapsed());
                                    ctx.write(result)
                                } else {
                                    Err(error.err("Match not found"))
                                }
                            }
                            Err(err) => Err(error.pass(err.to_string())),
                        }
                    }
                    Err(err) => Err(error.pass(err.to_string())),
                }
            }
            Err(err) => Err(error.pass(err)),
        }
    }
}
