use splora_api::Network;
use splora_web::{
    ClientError, FAIL_CLOSED, MissingSigner, Nip98Signer, UnsignedNip98, fetch_indexer,
    signer_from_presence,
};

#[test]
fn missing_signer_fails_closed() {
    let absent = signer_from_presence(false);
    assert!(absent.is_err(), "absent signer must be a typed error");
    assert!(matches!(absent.as_ref(), Err(ClientError::MissingSigner)));

    let missing = MissingSigner;
    assert!(matches!(
        missing.public_key_hex(),
        Err(ClientError::MissingSigner)
    ));

    let mut fetched = false;
    let mut authorization_seen: Option<String> = None;
    let result = fetch_indexer(
        None,
        "https://splora.surmount.systems",
        Network::Testnet3,
        "/tx/abc",
        1_700_000_000,
        &mut |url, authorization| {
            fetched = true;
            authorization_seen = Some(authorization.to_string());
            let _ = url;
            Ok(Vec::new())
        },
    );
    let err = result.expect_err("missing signer must not fetch");
    assert!(matches!(err, ClientError::MissingSigner));
    assert!(err.authorization().is_none());
    assert!(!err.fetched());
    assert!(!fetched, "indexer must not be called");
    assert!(
        authorization_seen.is_none(),
        "no Authorization value is produced, saw {authorization_seen:?}"
    );

    let unsigned = UnsignedNip98 {
        pubkey: "11".repeat(32),
        created_at: 1_700_000_000,
        kind: 27235,
        tags: vec![
            vec!["u".into(), "https://splora.surmount.systems/tx/abc".into()],
            vec!["method".into(), "GET".into()],
        ],
        content: String::new(),
    };
    assert!(missing.sign_event_json(&unsigned).is_err());
    assert!(!FAIL_CLOSED.to_ascii_lowercase().contains("nsec"));
    assert!(FAIL_CLOSED.contains("NIP-07"));
    assert!(FAIL_CLOSED.contains("was not called"));
}
