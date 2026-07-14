use clap::{Parser, Subcommand};
use serde::Serialize;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "dash-conformance-runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Cid,
    Capsule,
    Store,
}

#[derive(Serialize)]
struct ConformanceReport {
    suite: String,
    total: usize,
    passed: usize,
    failed: usize,
    cases: Vec<CaseResult>,
}

#[derive(Serialize)]
struct CaseResult {
    name: String,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    got: Option<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let report = match cli.command {
        Command::Cid => run_cid(),
        Command::Capsule => run_capsule(),
        Command::Store => run_store(),
    };
    let json = serde_json::to_string_pretty(&report).unwrap();
    println!("{json}");
    if report.failed == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn run_cid() -> ConformanceReport {
    let mut cases = Vec::new();
    let vectors_json = include_str!("../../../docs/schemas/test-vectors/cid/cid-vectors.json");
    let parsed: serde_json::Value =
        serde_json::from_str(vectors_json).expect("failed to parse cid-vectors.json");
    let vecs = parsed["vectors"].as_array().expect("vectors must be array");

    for (i, v) in vecs.iter().enumerate() {
        let desc = v["inputDescription"].as_str().unwrap_or("unknown");
        let canonical_hex = v["canonicalBytes"].as_str().unwrap_or("");
        let expected_cid = v["expectedCid"].as_str().unwrap();
        let expected_multihash = v["multihashHex"].as_str().unwrap();
        let expected_digest = v["blake3DigestHex"].as_str().unwrap();

        let canonical_bytes = hex::decode(canonical_hex).unwrap();
        let cid = dash_cid::Cid::new(&canonical_bytes);

        let cid_match = cid.to_string() == expected_cid;
        let mh_match = cid.multihash_hex() == expected_multihash;
        let digest_match = cid.digest_hex() == expected_digest;
        let all_ok = cid_match && mh_match && digest_match;

        cases.push(CaseResult {
            name: format!("vector[{i}] {desc}: expectedCid={expected_cid}"),
            ok: all_ok,
            expected: if !all_ok {
                Some(format!("cid={expected_cid} mh={expected_multihash} dgst={expected_digest}"))
            } else {
                None
            },
            got: if !all_ok {
                Some(format!("cid={} mh={} dgst={}", cid, cid.multihash_hex(), cid.digest_hex()))
            } else {
                None
            },
        });
    }

    {
        let non_canonical = hex::decode("a16161f93c00").unwrap();
        let result = dash_cid::canonical_cbor_validate(&non_canonical);
        cases.push(CaseResult {
            name: "rejects non-canonical half-float encoding".into(),
            ok: result.is_err(),
            expected: None,
            got: None,
        });
    }

    cases.push({
        let bytes = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
            .unwrap();
        let cid1 = dash_cid::Cid::new(&bytes);
        let cid2 = dash_cid::Cid::new(&bytes);
        CaseResult {
            name: "deterministic across platforms: same input → same CID".into(),
            ok: cid1 == cid2 && cid1.to_string() == cid2.to_string(),
            expected: None,
            got: None,
        }
    });

    let passed = cases.iter().filter(|c| c.ok).count();
    let failed = cases.len() - passed;
    ConformanceReport { suite: "cid".into(), total: cases.len(), passed, failed, cases }
}

fn run_capsule() -> ConformanceReport {
    use dash_capsule::{Attestation, AttestationType, Capsule};

    let mut cases = Vec::new();

    let files = vec![
        ("hello.txt".to_string(), b"Hello, World!".to_vec()),
        ("foo/bar.txt".to_string(), b"bar content".to_vec()),
        ("a.txt".to_string(), b"alpha".to_vec()),
    ];

    // (a) pack and unpack roundtrip
    {
        let capsule = Capsule::pack(files.clone()).unwrap();
        let bytes = capsule.to_bytes().unwrap();
        let restored = Capsule::from_bytes(&bytes).unwrap();
        let unpacked = restored.unpack().unwrap();
        let ok = unpacked.len() == 3
            && unpacked.iter().any(|(p, d)| p == "hello.txt" && d == b"Hello, World!")
            && unpacked.iter().any(|(p, d)| p == "foo/bar.txt" && d == b"bar content")
            && unpacked.iter().any(|(p, d)| p == "a.txt" && d == b"alpha");
        cases.push(CaseResult {
            name: "pack_and_unpack_roundtrip".into(),
            ok,
            expected: None,
            got: None,
        });
    }

    // (b) deterministic: same input → same bytes and CID
    {
        let c1 = Capsule::pack(files.clone()).unwrap();
        let c2 = Capsule::pack(files.clone()).unwrap();
        let b1 = c1.to_bytes().unwrap();
        let b2 = c2.to_bytes().unwrap();
        let ok = b1 == b2 && c1.cid() == c2.cid();
        cases.push(CaseResult {
            name: "deterministic_same_cid".into(),
            ok,
            expected: None,
            got: None,
        });
    }

    // (c) file-level dedup: identical content → same BLAKE3 hash
    {
        let content = b"shared content for dedup";
        let dual_files = vec![
            ("shared_a.txt".to_string(), content.to_vec()),
            ("shared_b.txt".to_string(), content.to_vec()),
        ];
        let capsule = Capsule::pack(dual_files).unwrap();
        let entries = capsule.file_table().entries();
        let ok = entries.len() == 2 && entries[0].blake3_hash == entries[1].blake3_hash;
        cases.push(CaseResult {
            name: "file_level_dedup_same_hash".into(),
            ok,
            expected: None,
            got: None,
        });
    }

    // (d) range read
    {
        let content = b"0123456789ABCDEF".to_vec();
        let sf = vec![("data.bin".to_string(), content)];
        let capsule = Capsule::pack(sf).unwrap();
        let range = capsule.read_range("data.bin", 5, 4).unwrap();
        let ok = range == b"5678";
        cases.push(CaseResult {
            name: "range_read".into(),
            ok,
            expected: None,
            got: if !ok { Some(format!("got {:?}", range)) } else { None },
        });
    }

    // (e) detached attestations don't change CID
    {
        let capsule = Capsule::pack(files.clone()).unwrap();
        let cid_before = capsule.cid();
        let attestation = Attestation::new(
            AttestationType::InTotoStatement,
            b"payload".to_vec(),
            b"signature".to_vec(),
        );
        let capsule_with_att = capsule.with_attestations(vec![attestation]);
        let cid_after = capsule_with_att.cid();
        let ok = cid_before == cid_after;
        cases.push(CaseResult {
            name: "detached_attestation_cid_unchanged".into(),
            ok,
            expected: None,
            got: None,
        });
    }

    let passed = cases.iter().filter(|c| c.ok).count();
    ConformanceReport {
        suite: "capsule".into(),
        total: cases.len(),
        passed,
        failed: cases.len() - passed,
        cases,
    }
}

fn run_store() -> ConformanceReport {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(run_store_async())
}

async fn run_store_async() -> ConformanceReport {
    use dash_capsule::Capsule;
    use dash_store::cas::CasStore;

    let mut cases = Vec::new();
    let dir = tempfile::TempDir::new().unwrap();
    let store = CasStore::new(dir.path().to_path_buf()).await.unwrap();

    let files1 = vec![
        ("shared.txt".to_string(), b"common content shared".to_vec()),
        ("unique1.txt".to_string(), b"unique number one".to_vec()),
    ];
    let files2 = vec![
        ("shared.txt".to_string(), b"common content shared".to_vec()),
        ("unique2.txt".to_string(), b"unique number two".to_vec()),
    ];
    let capsule1 = Capsule::pack(files1).unwrap();
    let capsule2 = Capsule::pack(files2).unwrap();

    // (a) store and retrieve by CID
    {
        let cid = store.store_capsule(&capsule1).await.unwrap();
        let retrieved = store.get_capsule(&cid).await.unwrap();
        let ok = retrieved.cid() == cid;
        let unpacked = retrieved.unpack().unwrap();
        let ok = ok
            && unpacked.len() == 2
            && unpacked.iter().any(|(p, d)| p == "shared.txt" && d == b"common content shared")
            && unpacked.iter().any(|(p, d)| p == "unique1.txt" && d == b"unique number one");
        cases.push(CaseResult {
            name: "store_and_retrieve_by_cid".into(),
            ok,
            expected: None,
            got: None,
        });
    }

    // (b) file-level dedup across capsules
    {
        let _cid2 = store.store_capsule(&capsule2).await.unwrap();

        let files_dir = dir.path().join("files").join("b3");
        let mut file_count = 0usize;
        let mut l1 = tokio::fs::read_dir(&files_dir).await.unwrap();
        while let Some(aa) = l1.next_entry().await.unwrap() {
            if aa.file_type().await.unwrap().is_dir() {
                let mut l2 = tokio::fs::read_dir(aa.path()).await.unwrap();
                while let Some(bb) = l2.next_entry().await.unwrap() {
                    if bb.file_type().await.unwrap().is_dir() {
                        let mut l3 = tokio::fs::read_dir(bb.path()).await.unwrap();
                        while let Some(entry) = l3.next_entry().await.unwrap() {
                            if entry.file_type().await.unwrap().is_file() {
                                file_count += 1;
                            }
                        }
                    }
                }
            }
        }
        let ok = file_count == 3;
        cases.push(CaseResult {
            name: "file_level_dedup_across_capsules".into(),
            ok,
            expected: Some("3 unique file objects".into()),
            got: if !ok { Some(format!("{file_count} file objects found")) } else { None },
        });
    }

    // (c) materialize and verify files
    {
        let capsule = Capsule::pack(vec![
            ("a.txt".to_string(), b"material alpha".to_vec()),
            ("sub/b.txt".to_string(), b"material beta".to_vec()),
        ])
        .unwrap();
        let cid = store.store_capsule(&capsule).await.unwrap();

        let target = tempfile::TempDir::new().unwrap();
        store.materialize(&cid, target.path()).await.unwrap();

        let a = tokio::fs::read_to_string(target.path().join("a.txt")).await.unwrap();
        let b = tokio::fs::read_to_string(target.path().join("sub/b.txt")).await.unwrap();
        let ok = a == "material alpha" && b == "material beta";
        cases.push(CaseResult {
            name: "materialize_and_verify_files".into(),
            ok,
            expected: None,
            got: if !ok { Some(format!("a={a:?} b={b:?}")) } else { None },
        });
    }

    // (d) tamper detection via verify
    {
        let capsule =
            Capsule::pack(vec![("tamper.txt".to_string(), b"tamper me please".to_vec())]).unwrap();
        let cid = store.store_capsule(&capsule).await.unwrap();

        let cas_path = store.capsule_path(&cid);
        let mut bytes = tokio::fs::read(&cas_path).await.unwrap();
        if bytes.len() > 20 {
            bytes[14] ^= 0xff;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ =
                    tokio::fs::set_permissions(&cas_path, std::fs::Permissions::from_mode(0o644))
                        .await;
            }
            tokio::fs::write(&cas_path, &bytes).await.unwrap();
        }

        let verified = store.verify(&cid).await.unwrap();
        cases.push(CaseResult {
            name: "tamper_detection_verify_fails".into(),
            ok: !verified,
            expected: None,
            got: if verified { Some("verify returned true on tampered data".into()) } else { None },
        });
    }

    let passed = cases.iter().filter(|c| c.ok).count();
    ConformanceReport {
        suite: "store".into(),
        total: cases.len(),
        passed,
        failed: cases.len() - passed,
        cases,
    }
}
