#[cfg(test)]
use std::time::Duration;

use std::{
    sync::Once, 
};
use sal_core::dbg::Dbg;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{
    DebugSession, 
    LogLevel, 
    Backtrace
};

use crate::modules::{Filter, FilterSmooth};
///
///
static INIT: Once = Once::new();
///
/// once called initialisation
fn init_once() {
    INIT.call_once(|| {
        // implement your initialisation code to be called only once for current test file
    })
}
///
/// returns:
///  - ...
fn init_each() -> () {}
///
/// Testing [FilterSmooth].add
#[test]
fn filter_u16() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    let dbg = Dbg::own("FilterSmooth-test-u16");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1.0,
            [
                (101, 001, Some(001)),
                (102, 002, Some(002)),
                (103, 003, Some(003)),
                (104, 004, Some(004)),
                (105, 012, Some(012)),
                (106, 006, Some(006)),
                (107, 008, Some(008)),
                (108, 009, Some(009)),
                (109, 010, Some(010)),
                (110, 011, Some(011)),
                (111, 012, Some(012)),
            ]
        ),
        (
            2.0,
            [
                (201, 001, Some(001)),
                (202, 002, Some(002)),
                (203, 003, Some(003)),
                (204, 004, Some(004)),
                (205, 012, Some(008)),
                (206, 006, Some(007)),
                (207, 008, Some(008)),
                (208, 009, Some(009)),
                (209, 010, Some(010)),
                (210, 011, Some(011)),
                (211, 012, Some(012)),
            ]
        ),
        (
            4.0,
            [
                (401, 001, Some(001)),
                (402, 002, Some(001)),
                (403, 003, Some(002)),
                (404, 004, Some(003)),
                (405, 012, Some(005)),
                (406, 006, Some(005)),
                (407, 008, Some(006)),
                (408, 009, Some(007)),
                (409, 010, Some(008)),
                (410, 011, Some(009)),
                (411, 012, Some(010)),
            ]
        ),
        (
            6.0,
            [
                (401, 001, Some(001)),
                (402, 002, Some(001)),
                (403, 003, Some(001)),
                (404, 004, Some(002)),
                (405, 012, Some(004)),
                (406, 006, Some(004)),
                (407, 008, Some(005)),
                (408, 009, Some(006)),
                (409, 010, Some(007)),
                (410, 011, Some(008)),
                (411, 012, Some(009)),
            ]
        ),
    ];
    for (factor, values) in test_data {
        log::debug!("factor: {factor}:");
        let mut filter: FilterSmooth<u16> = FilterSmooth::new(None, factor);
        for (step, val, target) in values {
            let result = filter.add(val);
            log::debug!("factor: {factor}  step {step}   val: {:?}  result: {:?}", val, result);
            assert!(result == target, "factor: {factor}  step {step}   \nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
    test_duration.exit();
}
