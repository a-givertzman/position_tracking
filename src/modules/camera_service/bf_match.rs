use std::time::Instant;

use frdm_tools::{ContextRead, ContextWrite, Eval, EvalResult, Image, ResultCtx};
use opencv::{core::{DMatch, KeyPoint, KeyPointTraitConst, Mat, MatTraitConst, Point_, VecN, Vector}, imgproc::LineTypes, prelude::{DescriptorMatcherTraitConst, Feature2DTrait}};
use sal_core::{dbg::Dbg, error::Error};

///
/// Brute Force Match
pub struct BfMatch {
    method: opencv::imgproc::TemplateMatchModes,
    match_ratio: f32,
    deviation_ratio: f32,
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
    pub fn new(method: opencv::imgproc::TemplateMatchModes, match_ratio: f64, deviation_ratio: f64, template: Image, ctx: impl Eval<Image, EvalResult> + 'static) -> Self {
        let dbg = Dbg::new("", "BfMatch");
        Self { 
            method,
            match_ratio: match_ratio as f32,
            deviation_ratio: deviation_ratio as f32,
            template,
            ctx: Box::new(ctx),
            dbg,
        }
    }
    ///
    /// Draws a dot on the image
    fn draw_dot(img: &mut Mat, x: f32, y: f32) {
        let _ = opencv::imgproc::circle(
            img,
            Point_::new(x.round() as i32, y.round() as i32),
            12,
            VecN([0.0, 0.0, 255.0, 0.0]),
            -1,
            LineTypes::FILLED as i32,
            0,
        );
    }
    ///
    /// Draws a text on the image
    fn draw_text(img: &mut Mat, x: i32, y: i32, text: &str) {
        let _ = opencv::imgproc::put_text(
            img,
            text,
            Point_::new(x, y),
            opencv::imgproc::HersheyFonts::FONT_HERSHEY_SIMPLEX as i32,
            1.0,
            VecN([0.0, 0.0, 255.0, 0.0]),
            4,
            LineTypes::FILLED as i32,
            false,
        );
    }
    ///
    /// Draws a text on the image
    fn draw_matches_knn(img1: &Mat, keypoints1: &Vector<KeyPoint>, img2: &Mat, keypoints2: &Vector<KeyPoint>, matches: &Vector<Vector<DMatch>>) -> Mat {
        let mut out = Mat::default();
        let _ = opencv::features2d::draw_matches_knn(
            img1, 
            keypoints1, 
            img2,
            keypoints2,
            matches,
            &mut out, 
            opencv::core::Scalar::new(0f64, 255f64, 0f64, 0f64), 
            opencv::core::Scalar::new(0f64, 255f64, 0f64, 0f64), 
            &Vector::default(), 
            opencv::features2d::DrawMatchesFlags::NOT_DRAW_SINGLE_POINTS,
        );
        out
    }
    ///
    /// ORB Matching
    fn bf_match(dbg: &Dbg, template_img: &Mat, input_img: &mut Mat, match_ratio: f32, deviation_ratio: f32) -> Result<(u16, u16), Error> {
        let mut orb = opencv::features2d::SIFT::create(
            0,
            3,
            0.04,
            10.0,
            1.6,
        ).map_err(|err| Error::new(dbg, "SIFT::create error").pass(err.to_string()))?;
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
        // log::debug!("{dbg}.bf_match | Train matches: {:?}", bf_matches);
        let mut bf_matches: Vector<Vector<DMatch>> = Vector::default();
        let mask = unsafe { Mat::new_rows_cols(0, 0, opencv::core::CV_8UC1).unwrap() };
        bf.knn_train_match(&template_descr, &input_descr, &mut bf_matches, 3, &mask, false)
            .map_err(|err| Error::new(dbg, "knn_train_match").pass(err.to_string()))?;
        log::trace!("{dbg}.bf_match | KNN matches: {:?}", bf_matches);
        let bf_matches: Vec<Vector<DMatch>> = bf_matches.iter().filter(|mm| {
            let m0 = mm.get(0).unwrap();
            let m1 = mm.get(1).unwrap();
            log::trace!("{dbg}.bf_match | Match: {:?}", m0);
            m0.distance < match_ratio * m1.distance
        }).collect();
        // log::debug!("{dbg}.bf_match | good matches: {:?}", good_matches);
        let center = Self::center(
            dbg,
            deviation_ratio,
            &bf_matches,
            &input_keypoints,
        );
        if let Some((x, y)) = &center {
            log::debug!("{dbg}.bf_match | Center: {x}, {y}");
            Self::draw_dot(input_img, *x, *y);
            Self::draw_text(input_img, 10, input_img.rows() - 48, &format!("x: {}, y: {}", x, y));
        }
        input_img.clone_from(
            &Self::draw_matches_knn(template_img, &template_keypoints, input_img, &input_keypoints, &bf_matches.into())
        );
        match center {
            Some((x, y)) => Ok((x.round() as u16, y.round() as u16)),
            None => Err(Error::new(dbg, "bf_match").err(format!("Can't find center of {} keypoints", input_keypoints.len()))),
        }
    }
    ///
    /// Returns a geometrical center of the points collection
    fn center(dbg: &Dbg, deviation_ratio: f32, matches: &Vec<Vector<DMatch>>, keypoints: &Vector<KeyPoint>) -> Option<(f32, f32)> {
        let points: Vector<KeyPoint> = matches.iter().fold(Vector::new(), |mut acc, m| {
            acc.push(keypoints.get(m.get(0).unwrap().train_idx as usize).unwrap());
            acc.push(keypoints.get(m.get(1).unwrap().train_idx as usize).unwrap());
            acc
        });
        let len = points.len();
        if len >= 2 {
            log::debug!("{dbg}.center | Total Keypoints: {}", len);
            let (mut xa, mut ya) = (0.0, 0.0);
            for p in &points {
                let p = p.pt();
                xa += p.x;
                ya += p.y;
                log::trace!("{dbg}.center | x: {}, y: {}", p.x, p.y);
            }
            xa = xa / len as f32;
            ya = ya / len as f32;
            let (deviation_av, deviations) = points.iter().fold((0.0, vec![]), |(acc, mut deviations), p| {
                let p = p.pt();
                let deviation = ((p.x - xa).powi(2) + (p.y - ya).powi(2)).sqrt();
                log::trace!("{dbg}.center | deviation: {}", deviation);
                deviations.push(deviation);
                (
                    acc + deviation,
                    deviations
                )
            });
            let deviation_av = deviation_av / len as f32;
            let mut len = 0;
            let filtered = points.iter().enumerate().filter(|(i, _)| {
                if deviations[*i] <= deviation_av * deviation_ratio {
                    log::trace!("{dbg}.center | Filtered deviation: {}", deviations[*i]);
                    len += 1;
                    true
                } else {
                    false
                }
            });
            let (mut xa, mut ya) = (0.0, 0.0);
            for (_, kp) in filtered {
                let p = kp.pt();
                xa += p.x;
                ya += p.y;
                log::trace!("{dbg}.center | x: {}, y: {}", p.x, p.y);
            }
            log::debug!("{dbg}.center | Filtered Keypoints: {}", len);
            xa = xa / len as f32;
            ya = ya / len as f32;
            Some((xa, ya))
        } else {
            None
        }
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
                match Self::bf_match(&self.dbg, &self.template.mat, &mut frame.mat, self.match_ratio, self.deviation_ratio) {
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
