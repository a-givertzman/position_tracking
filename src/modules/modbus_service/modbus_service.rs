use std::{io::Write, net::TcpStream, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object}, Service, ServiceWaiting, RECV_TIMEOUT}, sync::{Handles, Owner}, thread_pool::Scheduler};

use crate::modules::{FunctionCode, ModbusServiceConf, Register};

/// 
/// Communication with the Modbus device over TCP/IP
/// 
/// ## Message format
/// 
/// ```ignore
///  Transaction ID | Protocol ID | Length Field |  Unit ID | Function Code | Data
///  ---            | ---         | ---          | ---      | ---           | ---
///   2 Bytes       | 2 Bytes     | 2 Bytes      | 1 Bytes  | 1 Byte        | Vec<u8>
/// ```
/// 
/// ## Example | Write u16 value
/// 
/// Например устройства 00 (для Modbus TCP не используется, как я понял, только для шлюзов)
/// Функция 06, регистр 11, пишем число 1234
/// 
/// - Header (7 bytes):
/// - Transaction ID: u16, [0x00, 0x00] (любое число)
/// - Protocol ID: u16, [0x00, 0x00]
/// - Length Field: u16, [0x00, 0x04] (4 bytes)
/// - Unit ID: u8 0x00
/// - PDU:
/// - Function Code: u8 0x06
/// - Data: 
///    - Register: [0x00,0x0B]
///    - Value: [0x04,0xD2]
/// 
/// Посылка должна выглядеть так:
/// 
/// [0x00,0x00, 0x00,0x00, 0x00,0x04, 0x00, 0x06, 0x00,0x0B, 0x04,0xD2]
///

pub struct ModbusService {
    name: Name,
    conf: ModbusServiceConf,
    position: Owner<kanal::Receiver<(u16, u16)>>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl ModbusService {
    ///
    /// Crteates [ModbusService] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: ModbusServiceConf,
        position: kanal::Receiver<(u16, u16)>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "ModbusService");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            position: Owner::new(position),
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Build a Modbus command message
    fn build_modbus_cmd_message(tr_id: u16, pr_id: u16, unit: u8, function: u8, register: u16, value: u16) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(tr_id.to_be_bytes());
        bytes.extend(pr_id.to_be_bytes());
        bytes.extend([0x00, 0x04]);
        bytes.push(unit);
        bytes.push(function);
        bytes.extend(register.to_be_bytes());
        bytes.extend(value.to_be_bytes());
        bytes
    }
}
//
//
impl Object for ModbusService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for ModbusService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModbusService")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
// 
impl Service for ModbusService {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let (FunctionCode(function_code_x), Register(register_x)) = (conf.register_x.0, conf.register_x.1);
        let (FunctionCode(function_code_y), Register(register_y)) = (conf.register_y.0, conf.register_y.1);
        let position = self.position.take().unwrap();
        let exit = self.exit.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        log::debug!("{dbg}.run | Preparing thread...");
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            service_release.add(Ok(()));
            loop {
                match TcpStream::connect(&conf.addr) {
                    Ok(mut stream) => {
                        log::debug!("{dbg}.run | Connected to {:?}", conf.addr);
                        let mut buf = vec![];
                        loop {
                            match position.recv_timeout(RECV_TIMEOUT) {
                                Ok((pos_x, pos_y)) => {
                                    buf = Self::build_modbus_cmd_message(0, 0, conf.unit, function_code_x, register_x, pos_x);
                                    if let Err(err) = stream.write_all(&mut buf) {
                                        log::debug!("{dbg}.run | TcpStream write error {:?}", err);
                                    }
                                    buf = Self::build_modbus_cmd_message(0, 0, conf.unit, function_code_y, register_y, pos_y);
                                    if let Err(err) = stream.write_all(&mut buf) {
                                        log::debug!("{dbg}.run | TcpStream write error {:?}", err);
                                    }
                                }
                                Err(_) => {},
                            }
                        }
                    }
                    Err(err) => {
                        log::debug!("{dbg}.run | Can't connected to {:?}, \n\terror: {:?}", conf.addr, err);
                        std::thread::sleep(Duration::from_millis(3000));
                    }
                }
                if exit.load(Ordering::Acquire) {
                    break;
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        });
        match handle {
            Ok(handle) => {
                self.handles.push(handle);
                let r = match conf.wait_started {
                    Some(_) => {
                        log::info!("{}.run | Waiting while starting...", self.dbg);
                        service_waiting.wait()
                    }
                    None => Ok(()),
                };
                log::info!("{}.run | Starting - ok", self.dbg);
                r
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }
}
