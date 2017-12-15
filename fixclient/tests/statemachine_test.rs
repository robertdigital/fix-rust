extern crate fixclient;
extern crate fix;

use std::rc::Rc;

use fixclient::{FixSessionConfig, FixDictionary};
//use fixclient::connector::*;
use fixclient::connector::statemachine::*;
use fixclient::connector::memstore::*;
use fixclient::builder;
use fix::fixmessagegen::*;

#[test]
fn test_initial_state() {
    let store = create_store(|_| {});
    let sm = FixSyncState::new( store );

    assert_eq!("us connected - them connected", format!("{}", sm));
}

#[test]
fn test_logon_sent_state() {
    let store = create_store(|_| {});
    let mut sm = FixSyncState::new( store );

    let res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");

    assert_eq!("us logon - them logon", format!("{}", sm));
}

#[test]
fn test_logon_handshake_state() {
    let store = create_store(|_| {});
    let mut sm = FixSyncState::new( store );

    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
    let recv_res = sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");

    assert_eq!("us operational - them operational", format!("{}", sm));
    match recv_res {
        TransitionAction::None => {
            // expected
        },
        _ => {
             panic!("expecting None, but got {:?}", recv_res);
        }
    }
}

#[test]
fn test_server_sending_a_resend_request() {
    let store = create_store(|_| {});
    let mut sm = FixSyncState::new( store );

    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
    sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");
    let recv_res = sm.register_recv( &builder::build_resend_req( 2, 1, 10 ) ).expect("success");

    assert_eq!("us resync - them operational", format!("{}", sm));
    match recv_res {
        TransitionAction::ResendRange( range ) => {
            assert_eq!( (1,10), range );
            // expected
        },
        _ => {
            panic!("expecting None, but got {:?}", recv_res);
        }
    }
}

#[test]
fn test_responding_to_a_resend_request_with_a_full_gap_fill() {
    let store = create_store(|_| {});
    let mut sm = FixSyncState::new( store );

    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
    sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");
    sm.register_recv( &builder::build_resend_req( 2, 1, 10 ) ).expect("success");
    sm.register_sent( &builder::build_sequence_reset(2, 11, Some(true) ) );

    assert_eq!("us operational - them operational", format!("{}", sm));
}

#[test]
fn test_responding_to_a_resend_request_with_individual_messages() {
    let store = create_store(|_| {});
    let mut sm = FixSyncState::new( store );

    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
    sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");
    sm.register_recv( &builder::build_resend_req( 2, 2, 5 ) ).expect("success");
    sm.register_sent( &builder::build_new_order_single(2, true, "cl1", "AAPL", 100.0, 594.2, FieldSideEnum::Buy, FieldOrdTypeEnum::Limit ) );
    sm.register_sent( &builder::build_new_order_single(3, true, "cl2", "AAPL", 100.0, 594.2, FieldSideEnum::Buy, FieldOrdTypeEnum::Limit ) );
    sm.register_sent( &builder::build_new_order_single(4, true, "cl3", "AAPL", 100.0, 594.2, FieldSideEnum::Buy, FieldOrdTypeEnum::Limit ) );

    assert_eq!("us resync - them operational", format!("{}", sm));
}

#[test]
fn test_responding_to_a_resend_request_with_individual_messages_complete() {
    let store = create_store(|_| {});
    let mut sm = FixSyncState::new( store );

    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
    sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");
    sm.register_recv( &builder::build_resend_req( 2, 2, 5 ) ).expect("success");
    sm.register_sent( &builder::build_new_order_single(2, true, "cl1", "AAPL", 100.0, 594.2, FieldSideEnum::Buy, FieldOrdTypeEnum::Limit ) );
    sm.register_sent( &builder::build_new_order_single(3, true, "cl2", "AAPL", 100.0, 594.2, FieldSideEnum::Buy, FieldOrdTypeEnum::Limit ) );
    sm.register_sent( &builder::build_new_order_single(4, true, "cl3", "AAPL", 100.0, 594.2, FieldSideEnum::Buy, FieldOrdTypeEnum::Limit ) );
    sm.register_sent( &builder::build_new_order_single(5, true, "cl4", "AAPL", 100.0, 594.2, FieldSideEnum::Buy, FieldOrdTypeEnum::Limit ) );

    assert_eq!("us operational - them operational", format!("{}", sm));
}

//#[test]
//fn test_sending_a_resend_req() {
//    let store = create_store(|_| {});
//    let mut sm = FixSyncState::new( store );
//
//    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
//    sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");
//    sm.register_recv( &builder::build_resend_req( 2, 2, 5 ) ).expect("success");
//    sm.register_sent( &builder::build_new_order_single(2, 11, Some(true) ) );
//
//    assert_eq!("us operational - them operational", format!("{}", sm));
//}
//
//#[test]
//fn test_processing_resends_not_complete() {
//    let store = create_store(|_| {});
//    let mut sm = FixSyncState::new( store );
//
//    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
//    sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");
//    sm.register_recv( &builder::build_resend_req( 2, 2, 5 ) ).expect("success");
//    sm.register_sent( &builder::build_new_order_single(2, 11, Some(true) ) );
//
//    assert_eq!("us operational - them operational", format!("{}", sm));
//}
//
//#[test]
//fn test_processing_resends_complete() {
//    let store = create_store(|_| {});
//    let mut sm = FixSyncState::new( store );
//
//    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
//    sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");
//    sm.register_recv( &builder::build_resend_req( 2, 2, 5 ) ).expect("success");
//    sm.register_sent( &builder::build_new_order_single(2, 11, Some(true) ) );
//
//    assert_eq!("us operational - them operational", format!("{}", sm));
//}
//
//#[test]
//fn test_processing_resends_via_sequence_reset() {
//    let store = create_store(|_| {});
//    let mut sm = FixSyncState::new( store );
//
//    let send_res = sm.register_sent( &builder::build_logon( 1, None ) ).expect("success");
//    sm.register_recv( &builder::build_logon( 1, None ) ).expect("success");
//    sm.register_recv( &builder::build_resend_req( 2, 2, 5 ) ).expect("success");
//    sm.register_sent( &builder::build_new_order_single(2, 11, Some(true) ) );
//
//    assert_eq!("us operational - them operational", format!("{}", sm));
//}


fn create_store<F>( f : F ) -> Rc<MemoryMessageStore>
    where F : FnOnce(&mut MemoryMessageStore) -> () {

    let cfg = FixSessionConfig::new( "qualifier", "sender", "target", "hostname",
                                     1234, 60, "log", "store", FixDictionary::Fix42 );
    let mut store = MemoryMessageStore::new( &cfg ).unwrap();
    f(&mut store);
    let store = Rc::new( store );
    store
}



