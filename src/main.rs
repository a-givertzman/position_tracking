use std::fs::OpenOptions;

use frdm_tools::Image;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{conf::ConfTree, Service}, thread_pool::ThreadPool};

use crate::modules::{CameraService, CameraServiceConf};


mod modules;
///
/// Application entry point
fn main() -> Result<(), Error>{
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let dbg = Dbg::own("position-tracking");
    let thread_pool = ThreadPool::new(&dbg, Some(8));
    let conf = "config.yaml";
    let file = OpenOptions::new().read(true).open(conf).map_err(|err| Error::new(&dbg, "main").pass(err.to_string()))?;
    let conf = serde_yaml::from_reader(file).map_err(|err| Error::new(&dbg, "main").pass(err.to_string()))?;
    // log::debug!("{dbg}.main | conf: {:#?}", conf);
    let conf = ConfTree::new_root(conf);
    let (_, conf) = conf.get_by_keywd("", "service").unwrap();
    let conf = CameraServiceConf::new(&dbg, conf);
    let template = Image::load(&conf.template_match.template)?;
    let camera_service = CameraService::new(&dbg, conf, template, thread_pool.scheduler());
    camera_service.run()?;
    camera_service.wait()?;
    Ok(())
}
