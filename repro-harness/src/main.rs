use std::process;

use bnpm_capsule::Capsule;
use bnpm_cid::Cid;
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct TestVector {
    #[serde(rename = "inputDescription")]
    input_description: String,
    #[serde(rename = "canonicalBytes")]
    canonical_bytes: String,
    #[serde(rename = "blake3DigestHex")]
    blake3_digest_hex: String,
    #[serde(rename = "multihashHex")]
    multihash_hex: String,
    #[serde(rename = "expectedCid")]
    expected_cid: String,
}

#[derive(Deserialize)]
struct TestVectors {
    description: String,
    vectors: Vec<TestVector>,
}

fn main() {
    let verbose = std::env::args().any(|a| a == "--verbose");

    let manifest = include_str!("../../docs/schemas/test-vectors/cid/cid-vectors.json");
    let vectors: TestVectors =
        serde_json::from_str(manifest).expect("failed to parse test vectors");

    println!("=== Bnpm Reproducibility Harness ===");
    println!("Suite: {}", vectors.description);
    println!("Vectors: {}\n", vectors.vectors.len());

    let mut passed = 0;
    let mut failed = 0;

    for (i, v) in vectors.vectors.iter().enumerate() {
        let canonical_bytes = hex::decode(&v.canonical_bytes).unwrap_or_default();
        let cid = Cid::new(&canonical_bytes);
        let cid_str = cid.to_string();

        let ok = cid_str == v.expected_cid;

        if ok {
            passed += 1;
            if verbose {
                println!("  PASS [{}] {}: {}", i + 1, v.input_description, cid_str);
            }
        } else {
            failed += 1;
            println!(
                "  FAIL [{}] {}:\n    expected: {}\n    got:      {}",
                i + 1,
                v.input_description,
                v.expected_cid,
                cid_str
            );
        }
    }

    // ── Determinism tests ──────────────────────────────────────────────
    println!("\n--- Determinism Tests ---");

    // Cid::new() 100 times on same input
    let input = b"determinism test input";
    let mut cids = Vec::with_capacity(100);
    for _ in 0..100 {
        cids.push(Cid::new(input));
    }
    let all_identical = cids.windows(2).all(|w| w[0] == w[1]);
    if all_identical {
        println!("  PASS Cid determinism: 100 invocations → identical");
        passed += 1;
    } else {
        println!("  FAIL Cid determinism: 100 invocations differ");
        failed += 1;
    }

    // Capsule::pack() twice → byte-identical
    let files = vec![
        ("hello.txt".to_string(), b"Hello, World!".to_vec()),
        ("foo/bar.txt".to_string(), b"bar content".to_vec()),
    ];
    let capsule1 = Capsule::pack(files.clone()).expect("pack 1");
    let capsule2 = Capsule::pack(files).expect("pack 2");
    let bytes1 = capsule1.to_bytes().expect("to_bytes 1");
    let bytes2 = capsule2.to_bytes().expect("to_bytes 2");
    if bytes1 == bytes2 {
        println!("  PASS Capsule determinism: 2 pack() calls → byte-identical");
        passed += 1;
    } else {
        println!("  FAIL Capsule determinism: byte output differs");
        failed += 1;
    }

    // CBOR canonicalization test: pack same content with sorted/unsorted file order
    let files_unsorted = vec![
        ("z_file.txt".to_string(), b"content".to_vec()),
        ("a_file.txt".to_string(), b"content".to_vec()),
    ];
    let files_sorted = vec![
        ("a_file.txt".to_string(), b"content".to_vec()),
        ("z_file.txt".to_string(), b"content".to_vec()),
    ];
    let c_unsorted = Capsule::pack(files_unsorted)
        .expect("pack unsorted")
        .to_bytes()
        .expect("to_bytes unsorted");
    let c_sorted = Capsule::pack(files_sorted)
        .expect("pack sorted")
        .to_bytes()
        .expect("to_bytes sorted");
    if c_unsorted == c_sorted {
        println!("  PASS CBOR canonicalization: file order normalized");
        passed += 1;
    } else {
        println!("  FAIL CBOR canonicalization: file order not normalized");
        failed += 1;
    }

    // ── Report ────────────────────────────────────────────────────────
    println!("\n=== Reproducibility Report ===");
    println!("  Total checks: {}", passed + failed);
    println!("  Passed:       {}", passed);
    println!("  Failed:       {}", failed);

    if failed > 0 {
        process::exit(1);
    }
}
