use std::time::Instant;

use frdm_tools::{ContextRead, ContextWrite, Eval, EvalResult, Image, ResultCtx};
use opencv::{core::{DMatch, Mat, Vector}, prelude::{DescriptorMatcherTraitConst, Feature2DTrait}};
use sal_core::{dbg::Dbg, error::Error};

///
/// Brute Force Match
pub struct BfMatch {
    method: opencv::imgproc::TemplateMatchModes,
    match_ratio: f64,
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
    pub fn new(method: opencv::imgproc::TemplateMatchModes, match_ratio: f64, template: Image, ctx: impl Eval<Image, EvalResult> + 'static) -> Self {
        let dbg = Dbg::new("", "BfMatch");
        Self { 
            method,
            match_ratio,
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
            Err(err) => Err(Error::new(&self.dbg, "area").pass(err.to_string())),
        }
    }
    ///
    /// ORB Matching
    fn bf_match(dbg: &Dbg, template_img: &Mat, input_img: &mut Mat, match_ratio: f32) -> Result<(), Error> {
        let mut orb = opencv::features2d::SIFT::create(
            0,
            3,
            0.04,
            10.0,
            1.6,
        ).map_err(|err| Error::new(dbg, "SIFT::create error").pass(err.to_string()))?;
        // opencv::features2d::ORB::create(
        //     500,
        //     1.2,
        //     8,
        //     31,
        //     0,
        //     2,
        //     opencv::features2d::ORB_ScoreType::HARRIS_SCORE, //FAST_SCORE
        //     31,
        //     20,        
        //     // nfeatures, scale_factor, nlevels, edge_threshold, first_level, wta_k, score_type, patch_size, fast_threshold
        // )
            // .map_err(|err| Error::new(dbg, "RB::create error").pass(err.to_string()))?;
        let mask = Mat::default();
        let mut template_keypoints = Vector::default();
        let mut input_keypoints = Vector::default();
        let mut template_descr = Mat::default();
        let mut input_descr = Mat::default();
        orb.detect_and_compute(template_img, &mask, &mut template_keypoints, &mut template_descr, false)
            .map_err(|err| Error::new(dbg, "detect_and_compute template_img error").pass(err.to_string()))?;
        orb.detect_and_compute(input_img, &mask, &mut input_keypoints, &mut input_descr, false)
            .map_err(|err| Error::new(dbg, "detect_and_compute input_img error").pass(err.to_string()))?;
        let bf = opencv::features2d::FlannBasedMatcher::create()    //opencv::core::NORM_L2 , true
            .map_err(|err| Error::new(dbg, "BFMatcher::create error").pass(err.to_string()))?;
        // let mut bf_matches: Vector<DMatch> = Vector::default();
        // bf.train_match(&template_descr, &input_descr, &mut bf_matches, &mask)
        //     .map_err(|err| Error::new(dbg, "train_match").pass(err.to_string()))?;
        // let mut bf_matches = bf_matches.to_vec();
        // bf_matches.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        // let mut bf_matches: Vec<DMatch> = bf_matches.iter().filter(|m| m.distance < 40.0).cloned().collect();
        // let bf_matches = match bf_matches.get(..10) {
        //     Some(m) => m.to_vec(),
        //     None => vec![],
        // };
        // log::debug!("{dbg}.bf_match | Train matches: {:?}", bf_matches);
        let mut bf_matches: Vector<Vector<DMatch>> = Vector::default();
        let mask = unsafe { Mat::new_rows_cols(0, 0, opencv::core::CV_8UC1).unwrap() };
        bf.knn_train_match(&template_descr, &input_descr, &mut bf_matches, 3, &mask, false)
            .map_err(|err| Error::new(dbg, "knn_train_match").pass(err.to_string()))?;
        log::trace!("{dbg}.bf_match | KNN matches: {:?}", bf_matches);
        // let mut good_matches = Vector::default();
        let bf_matches: Vec<Vector<DMatch>> = bf_matches.iter().filter(|mm| {
            let m0 = mm.get(0).unwrap();
            let m1 = mm.get(1).unwrap();
            log::trace!("{dbg}.bf_match | Match: {:?}", m0);
            m0.distance < match_ratio * m1.distance
        }).collect();
        // log::debug!("{dbg}.bf_match | good matches: {:?}", good_matches);
        let mut out = Mat::default();
        opencv::features2d::draw_matches_knn(
            template_img, 
            &template_keypoints, 
            input_img,
            &input_keypoints, 
            &bf_matches.into(),
            &mut out, 
            opencv::core::Scalar::new(0f64, 255f64, 0f64, 0f64), 
            opencv::core::Scalar::new(0f64, 255f64, 0f64, 0f64), 
            &Vector::default(), 
            opencv::features2d::DrawMatchesFlags::NOT_DRAW_SINGLE_POINTS,
        ).map_err(|err| Error::new(dbg, "Can't draw matches").pass(err.to_string()))?;
        input_img.clone_from(&out);
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
                match Self::bf_match(&self.dbg, &self.template.mat, &mut frame.mat, self.match_ratio as f32) {
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
