use std::time::Instant;

use frdm_tools::{ContextRead, ContextWrite, Eval, EvalResult, Image, ResultCtx};
use opencv::{core::Mat, prelude::{DescriptorMatcherTrait, DescriptorMatcherTraitConst, Feature2DTrait}};
use sal_core::{dbg::Dbg, error::Error};

///
/// Brute Force Match
pub struct BfMatch {
    method: opencv::imgproc::TemplateMatchModes,
    threshold: f64,
    template: Image,
    ctx: Box<dyn Eval<Image, EvalResult>>,
    dbg: Dbg,
}
//
//
impl BfMatch {
    ///
    /// Returns [BfMatch] new instance
    /// - `threshold` - ...
    /// - `method` - TM_CCOEFF_NORMED or TM_CCORR_NORMED
    pub fn new(method: opencv::imgproc::TemplateMatchModes, threshold: f64, template: Image, ctx: impl Eval<Image, EvalResult> + 'static) -> Self {
        let dbg = Dbg::new("", "BfMatch");
        Self { 
            method,
            threshold,
            template,
            ctx: Box::new(ctx),
            dbg,
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
    ///
    /// ORB Matching
    fn orb_match(dbg: &Dbg, pattern_img: &opencv::core::Mat, dst_img: &mut opencv::core::Mat, match_ratio: f32) -> Result<(), Error> {
        let orb = opencv::features2d::ORB::create(
            500,
            1.2,
            8,
            31,
            0,
            2,
            opencv::features2d::ORB_ScoreType::HARRIS_SCORE, //FAST_SCORE
            31,
            20,        
            // nfeatures, scale_factor, nlevels, edge_threshold, first_level, wta_k, score_type, patch_size, fast_threshold
        );
        match orb {
            Ok(mut orb) => {
                let mut keypoints_pattern = opencv::core::Vector::default();
                let mut keypoints_dst_img = opencv::core::Vector::default();
                let mut desc_pattern = opencv::core::Mat::default();
                let mut desc_dst_img = opencv::core::Mat::default();
                if let Err(err) = orb.detect_and_compute(pattern_img, &opencv::core::Mat::default(), &mut keypoints_pattern, &mut desc_pattern, false) {
                    println!("orbMatch | detect_and_compute error:\n\t{:?}", err);
                }
                if let Err(err) = orb.detect_and_compute(dst_img, &opencv::core::Mat::default(), &mut keypoints_dst_img, &mut desc_dst_img, false) {
                    println!("orbMatch | detect_and_compute error:\n\t{:?}", err);
                }
                let mut bf_matches: opencv::core::Vector<opencv::core::DMatch> = opencv::core::Vector::default();    // opencv::core::Vector<opencv::core::Vector<opencv::core::DMatch>>
                match opencv::features2d::BFMatcher::create(opencv::core::NORM_HAMMING , true) {
                    Ok(bf) => {
                        bf.train_match(&desc_pattern, &desc_dst_img, &mut bf_matches, &opencv::core::Mat::default())
                            .map_err(|err| Error::new(dbg, "orb_match").pass(err.to_string()))?;
                        bf
                    },
                    Err(err) => panic!("orbMatch | BFMatcher create error: {:?}", err),
                };
                println!("orbMatch | matches: {:?}", bf_matches);
                // let mut good_matches = opencv::core::Vector::default();
                // for mm in bf_matches {
                //     let m0 = mm.get(0).unwrap();
                //     let m1 = mm.get(1).unwrap();
                //     println!("orbMatch | Match: {:?}", m0);
                //     if m0.distance < match_ratio * m1.distance {
                //         good_matches.push(m0);
                //     }
                // }
                // println!("orbMatch | good matches: {:?}", good_matches);
                let mut out = opencv::core::Mat::default();
                if let Err(err) = opencv::features2d::draw_matches(
                    pattern_img, 
                    &keypoints_pattern, 
                    dst_img,
                    &keypoints_dst_img, 
                    &bf_matches, 
                    &mut out, 
                    opencv::core::Scalar::new(0f64, 255f64, 0f64, 0f64), 
                    opencv::core::Scalar::new(0f64, 255f64, 0f64, 0f64), 
                    &opencv::core::Vector::default(), 
                    opencv::features2d::DrawMatchesFlags::NOT_DRAW_SINGLE_POINTS,
                ) {
                    println!("orbMatch | Error: {:?}", err);
                }
                dst_img.clone_from(&out);
                // features2d::draw_keypoints(
                //     patternImg,
                //     &keypoints,
                //     dstImg,
                //     core::VecN([0., 255., 0., 255.]),
                //     features2d::DrawMatchesFlags::DEFAULT,
                // )?;
                // imgproc::rectangle(
                //     dstImg,
                //     core::Rect::from_points(core::Point::new(0, 0), core::Point::new(50, 50)),
                //     core::VecN([255., 0., 0., 0.]),
                //     -1,
                //     imgproc::LINE_8,
                //     0,
                // )?;
                // // Use SIFT
                // let mut sift = features2d::SIFT::create(0, 3, 0.04, 10., 1.6)?;
                // let mut sift_keypoints = core::Vector::default();
                // let mut sift_desc = core::Mat::default();
                // sift.detect_and_compute(imgPattern, &mask, &mut sift_keypoints, &mut sift_desc, false)?;
                // features2d::draw_keypoints(
                //     &dstImg.clone(),
                //     &sift_keypoints,
                //     dstImg,
                //     core::VecN([0., 0., 255., 255.]),
                //     features2d::DrawMatchesFlags::DEFAULT,
                // )?;
            },
            Err(err) => {
                println!("orbMatch | creating ORB error:\n\t{:?}", err);
            },
        };
        Ok(())
    }
}
//
//
impl Eval<Image, EvalResult> for BfMatch {
    fn eval(&self, src: Image) -> EvalResult {
        let error = Error::new("BfMatch", "eval");
        match self.ctx.eval(src.clone()) {
            Ok(ctx) => {
                let t = Instant::now();
                let result: &ResultCtx = ctx.read();
                let mut frame = result.frame.clone();
                match Self::orb_match(&self.dbg, &self.template.mat, &mut frame.mat, 0.8) {
                    Ok(_) => {
                        let result = ResultCtx { frame: frame };
                        log::debug!("BfMatch.eval | Elapsed: {:?}", t.elapsed());
                        ctx.write(result)
                    }
                    Err(err) => Err(error.pass(err.to_string())),
                }
            }
            Err(err) => Err(error.pass(err)),
        }
    }
}
