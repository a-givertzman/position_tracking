use frdm_tools::Image;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{conf::ConfTree, Service}, thread_pool::ThreadPool};

use crate::modules::{CameraServiceConf, CameraService};

mod modules;
///
/// Application entry point
fn main() -> Result<(), Error>{
    let dbg = Dbg::own("position-tracking");
    let thread_pool = ThreadPool::new(&dbg, Some(8));
    let conf = ConfTree::empty();
    let conf = CameraServiceConf::new(&dbg, conf);
    let template = Image::load(&conf.template_match.template)?;
    let camera_service = CameraService::new(&dbg, conf, template, thread_pool.scheduler());
    camera_service.run()?;
    camera_service.wait()?;
    Ok(())
}
