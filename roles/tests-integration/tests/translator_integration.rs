// This file contains integration tests for the `TranslatorSv2` module.
//
// `TranslatorSv2` is a module that implements the Translator role in the Stratum V2 protocol.
//
// Note that it is enough to call `start_tracing()` once in the test suite to enable tracing for
// all tests. This is because tracing is a global setting.
use const_sv2::{MESSAGE_TYPE_SETUP_CONNECTION, MESSAGE_TYPE_SUBMIT_SHARES_EXTENDED};
use integration_tests_sv2::{sniffer::*, *};
use roles_logic_sv2::parsers::{CommonMessages, Mining, PoolMessages};

// This test runs an sv2 translator between an sv1 mining device and a pool. the connection between
// the translator and the pool is intercepted by a sniffer. The test checks if the translator and
// the pool exchange the correct messages upon connection. And that the miner is able to submit
// shares.
#[tokio::test]
async fn translate_sv1_to_sv2_successfully() {
    start_tracing();
    let (_tp, tp_addr) = start_template_provider(None);
    let (_pool, pool_addr) = start_pool(Some(tp_addr)).await;
    let (pool_translator_sniffer, pool_translator_sniffer_addr) =
        start_sniffer("0".to_string(), pool_addr, false, None).await;
    let (_, tproxy_addr) = start_sv2_translator(pool_translator_sniffer_addr).await;
    let _mining_device = start_mining_device_sv1(tproxy_addr, false, None).await;
    pool_translator_sniffer
        .wait_for_message_type(MessageDirection::ToUpstream, MESSAGE_TYPE_SETUP_CONNECTION)
        .await;
    assert_common_message!(
        &pool_translator_sniffer.next_message_from_downstream(),
        SetupConnection
    );
    assert_common_message!(
        &pool_translator_sniffer.next_message_from_upstream(),
        SetupConnectionSuccess
    );
    assert_mining_message!(
        &pool_translator_sniffer.next_message_from_downstream(),
        OpenExtendedMiningChannel
    );
    assert_mining_message!(
        &pool_translator_sniffer.next_message_from_upstream(),
        OpenExtendedMiningChannelSuccess
    );
    assert_mining_message!(
        &pool_translator_sniffer.next_message_from_upstream(),
        NewExtendedMiningJob
    );
    //panic!("Not yet implemented");
    tokio::task::spawn(async move {
        pool_translator_sniffer
            .wait_for_message_type(
                MessageDirection::ToUpstream,
                MESSAGE_TYPE_SUBMIT_SHARES_EXTENDED,
            )
            .await;
    });
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
}
