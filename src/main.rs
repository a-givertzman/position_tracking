use std::{fs::OpenOptions, str::FromStr};

use frdm_tools::Image;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{conf::{ConfKeywd, ConfKind, ConfTree}, Service}, thread_pool::ThreadPool};

use crate::modules::{CameraService, CameraServiceConf, ModbusService, ModbusServiceConf};


mod modules;
///
/// Application entry point
fn main() -> Result<(), Error>{
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let dbg = Dbg::own("position-tracking");
    let thread_pool = ThreadPool::new(&dbg, Some(8));
    let (position_send, position_recv) = kanal::unbounded();
    let mut position_recv = vec![position_recv];
    let conf = "config.yaml";
    let file = OpenOptions::new().read(true).open(conf).map_err(|err| Error::new(&dbg, "main").pass(err.to_string()))?;
    let conf = serde_yaml::from_reader(file).map_err(|err| Error::new(&dbg, "main").pass(err.to_string()))?;
    // log::debug!("{dbg}.main | conf: {:#?}", conf);
    let mut services: Vec<Box<dyn Service>> = vec![];
    let conf = ConfTree::new_root(conf);
    for node in conf.nodes() {
        if let Ok(keywd) = ConfKeywd::from_str(&node.key) {
            if keywd.kind() == ConfKind::Service.to_string() {
                match keywd.kind().as_str() {
                    "CameraService" => {
                        let conf = CameraServiceConf::new(&dbg, node);
                        let template = Image::load(&conf.template_match.template)?;
                        let service = CameraService::new(&dbg, conf, template, position_send.clone(), thread_pool.scheduler());
                        services.push(Box::new(service));
                    }
                    "ModbusService" => {
                        let conf = ModbusServiceConf::new(&dbg, node);
                        let service = ModbusService::new(&dbg, conf, position_recv.pop().unwrap(), thread_pool.scheduler());
                        services.push(Box::new(service));
                    }
                    _ => log::debug!("{dbg}.main | Unknown Service '{} {} {}' in the configuration", keywd.kind(), keywd.name(), keywd.title()),
                }
            }
        }
    }
    for service in &services {
        service.run()?;
    }
    for service in &services {
        service.wait()?;
    }
    Ok(())
}
