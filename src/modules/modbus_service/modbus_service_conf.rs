use std::{str::FromStr, time::Duration};
use frdm_tools::camera::CameraConf;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfCustomKeywd, ConfTree, ConfTreeGet}, entity::Name};

use crate::modules::TemplateMatchConf;

///
/// Modbus Function Code u8
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionCode(pub u8);
impl FunctionCode {
    pub fn be_bytes(&self) -> [u8; 1] {
        self.0.to_be_bytes()
    }
    pub fn le_bytes(&self) -> [u8; 1] {
        self.0.to_le_bytes()
    }
}
///
/// Modbus Register address u16
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Register(pub u16);
impl Register {
    pub fn be_bytes(&self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
    pub fn le_bytes(&self) -> [u8; 2] {
        self.0.to_le_bytes()
    }
}
///
/// ## The configuration parameters for the `RopeDefect`
/// 
/// ### Conf example
/// ```yaml
/// service ModbusService:
///     wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///     unit 01:
///         address: 192.168.100.1:502
///         function-x 03: 101
///         function-y 03: 103
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ModbusServiceConf {
    pub name: Name,
    /// Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    pub unit: u8,
    pub addr: String,
    pub register_x: (FunctionCode, Register),
    pub register_y: (FunctionCode, Register),
}
//
// 
impl ModbusServiceConf {
    ///
    /// Returns [ModbusServiceConf] built from `ConfTree`:
    pub fn new(
        parent: impl Into<String>,
        conf: ConfTree,
    ) -> Self {
        let parent = parent.into();
        let me = "ModbusServiceConf";
        let dbg = Dbg::new(&parent, me);
        let name = Name::new(parent, me);
        // log::debug!("{dbg}.new | conf: {:#?}", conf);
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let (unit, addr, register_x, register_y) = conf.nodes()
            .filter_map(|node| {
                log::debug!("{dbg}.new | node: {:#?}", node);
                match ConfCustomKeywd::from_str(&node.key) {
                    Ok(keywd) => {
                        if keywd.name() == "unit" {
                            let unit = keywd.title().parse().expect(&format!("{dbg}.new | 'unit number' - not found or wrong configuration"));
                            let addr: String = node.get("address").expect(&format!("{dbg}.new | 'unit {}: address' - not found or wrong configuration", keywd.title()));
                            let (x_function, x_register) = node.get_by_custom_keywd("", "function-x").map(|(keywd, node)| {
                                (
                                    FunctionCode(keywd.title().parse().expect(&format!("{dbg}.new | 'unit {}: function-x code' - not found or wrong configuration", keywd.title()))),
                                    Register(node.conf.as_u64().expect(&format!("{dbg}.new | 'unit {}: function-x register' - not found or wrong configuration", keywd.title())) as u16)
                                )
                            }).expect(&format!("{dbg}.new | 'unit {}: function-x' - not found or wrong configuration", keywd.title()));
                            let (y_function, y_register) = node.get_by_custom_keywd("", "function-y").map(|(keywd, node)| {
                                (
                                    FunctionCode(keywd.title().parse().expect(&format!("{dbg}.new | 'unit {}: function-y code' - not found or wrong configuration", keywd.title()))),
                                    Register(node.conf.as_u64().expect(&format!("{dbg}.new | 'unit {}: function-y register' - not found or wrong configuration", keywd.title())) as u16)
                                )
                            }).expect(&format!("{dbg}.new | 'unit {}: function-y' - not found or wrong configuration", keywd.title()));
                            Some((unit, addr, (x_function, x_register), (y_function, y_register)))
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            })
            .next()
            .expect(&format!("{dbg}.new | 'unit' - not found or wrong configuration"));
        log::debug!("{}.new | unit: {:?}", dbg, unit);
        log::debug!("{}.new | \taddr: {:?}", dbg, addr);
        log::debug!("{}.new | \tregister_x: {:?}", dbg, register_x);
        log::debug!("{}.new | \tregister_y: {:?}", dbg, register_y);
        Self {
            name,
            wait_started,
            unit,
            addr,
            register_x,
            register_y,
        }
    }
}
///
/// Camera unique identifier to be used in the sql database and folder name
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraId(pub usize);
// Implement the Default trait to provide a default value
impl Default for CameraId {
    fn default() -> Self {
        CameraId(0) // Default value for the wrapped usize
    }
}

// Implement Deref to allow immutable dereferencing to usize
impl std::ops::Deref for CameraId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0 // Dereference to the inner usize
    }
}

// Implement DerefMut to allow mutable dereferencing to usize
impl std::ops::DerefMut for CameraId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0 // Mutably dereference to the inner usize
    }
}
