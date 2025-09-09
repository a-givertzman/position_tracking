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
    let conf = ConfTree::new_root(conf);
    let services: Vec<Box<dyn Service>> = conf.nodes()
        .filter_map(|node| {
            match ConfKeywd::from_str(&node.key) {
                Ok(keywd) => match keywd.kind() == ConfKind::Service.to_string() {
                    true => Some((keywd, node)),
                    false => None,
                }
                Err(_) => None,
            }
        })
        .filter_map::<Box<dyn Service>, _>(|(keywd, node)| {
            match keywd.name().as_str() {
                "CameraService" => {
                    let conf = CameraServiceConf::new(&dbg, node);
                    match Image::load(&conf.template_match.template) {
                        Ok(template) => {
                            let service = CameraService::new(&dbg, conf, template, position_send.clone(), thread_pool.scheduler());
                            Some(Box::new(service))
                        }
                        Err(err) => {
                            log::debug!("{dbg}.main | Can't read template: {:?}", err);
                            None
                        }
                    }
                }
                "ModbusService" => {
                    log::debug!("{dbg}.main | ModbusService conf: {:#?}", node);
                    let conf = ModbusServiceConf::new(&dbg, node);
                    let service = ModbusService::new(&dbg, conf, position_recv.pop().unwrap(), thread_pool.scheduler());
                    Some(Box::new(service))
                }
                _ => {
                    log::debug!("{dbg}.main | Unknown Service '{} {} {}' in the configuration", keywd.kind(), keywd.name(), keywd.title());
                    None
                }
            }
        })
        .collect();
    for service in &services {
        service.run()?;
    }
    for service in &services {
        service.wait()?;
    }
    Ok(())
}
