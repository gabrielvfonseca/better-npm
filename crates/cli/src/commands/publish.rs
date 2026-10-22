use crate::commands::PublishArgs;
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::{Context, Result};
use bnpm_manifest::schema::Manifest;
use bnpm_proto::DprClient;

pub async fn run(args: &PublishArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = if args.path.as_os_str().is_empty() {
        std::env::current_dir().context("failed to get current directory")?
    } else {
        args.path.clone()
    };

    let manifest_path = cwd.join("bnpm.toml");
    if !manifest_path.exists() {
        print_error("No bnpm.toml found in current directory");
        return Ok(ExitCode::Error);
    }

    let manifest = Manifest::from_toml_path(&manifest_path).context("failed to parse bnpm.toml")?;
    let pkg = manifest.package.as_ref().context("bnpm.toml missing [package] section")?;
    let name = &pkg.name;
    let version = &pkg.version;

    let files = collect_files(&cwd)?;
    let capsule = bnpm_capsule::Capsule::pack(files).context("failed to pack capsule")?;
    let capsule_bytes = capsule.to_bytes().context("failed to serialize capsule")?;
    let _cid = capsule.cid().to_string();

    let registry_url = args
        .registry
        .clone()
        .or_else(|| config.registry.first().cloned())
        .unwrap_or_else(|| "http://0.0.0.0:9481".to_string());

    let identity = bnpm_proto::dpr_v1::Identity {
        oidc_issuer: "https://bnpm.dev".to_string(),
        oidc_subject: name.clone(),
        key_id: "dev-key".to_string(),
        public_key: vec![],
    };

    let mut client = DprClient::connect(&registry_url)
        .await
        .map_err(|e| anyhow::anyhow!("failed to connect to registry at {registry_url}: {e}"))?;

    let resp = client
        .publish(capsule_bytes.into(), vec![], String::new(), identity)
        .await
        .context("publish failed")?;

    if config.json_output {
        print_json(&serde_json::json!({
            "cid": resp.cid,
            "version": resp.version,
            "logIndex": resp.log_index,
            "inclusionProof": resp.inclusion_proof.map(|p| serde_json::json!({
                "lemma": p.lemma,
                "rootHash": p.root_hash,
                "treeSize": p.tree_size,
            }))
        }));
    } else {
        print_success(&format!("Published {}@{}", name, version));
        print_kv("CID", &resp.cid);
        print_kv("Log Index", &resp.log_index.to_string());
        if let Some(ref proof) = resp.inclusion_proof {
            print_kv("Root Hash", &proof.root_hash);
            print_kv("Tree Size", &proof.tree_size.to_string());
        }
    }

    // Dual-publish to npm
    if args.dual {
        if config.json_output {
            print_json(&serde_json::json!({
                "dualPublish": "starting npm publish"
            }));
        } else {
            print_info("Dual-publishing to npm...");
        }
        match npm_publish(name, version, &capsule, &args.npm_tag) {
            Ok(_) => {
                if config.json_output {
                    print_json(&serde_json::json!({
                        "dualPublish": "published",
                        "npmName": name,
                        "npmVersion": version,
                    }));
                } else {
                    print_success(&format!("Dual-published {}@{} to npm", name, version));
                }
            },
            Err(e) => {
                if config.json_output {
                    print_json(&serde_json::json!({
                        "dualPublish": "failed",
                        "error": e.to_string(),
                    }));
                } else {
                    print_warning(&format!("Dual-publish to npm failed: {e}"));
                    print_info("Bnpm registry publish was successful regardless.");
                }
            },
        }
    }

    Ok(ExitCode::Success)
}

fn collect_files(cwd: &std::path::Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    collect_dir(cwd, cwd, &mut files)?;
    Ok(files)
}

fn collect_dir(
    base: &std::path::Path,
    dir: &std::path::Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.starts_with('.') || file_name == "node_modules" || file_name == "target" {
            continue;
        }

        if path.is_dir() {
            collect_dir(base, &path, files)?;
        } else if path.is_file() {
            let rel_path = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().to_string();
            let data = std::fs::read(&path)?;
            files.push((rel_path, data));
        }
    }
    Ok(())
}

fn npm_publish(
    name: &str,
    version: &str,
    capsule: &bnpm_capsule::Capsule,
    npm_tag: &Option<String>,
) -> Result<(), anyhow::Error> {
    let token = std::env::var("NPM_TOKEN")
        .or_else(|_| std::env::var("npm_token"))
        .unwrap_or_default();

    if token.is_empty() {
        anyhow::bail!("NPM_TOKEN environment variable not set");
    }

    let files = capsule.unpack().context("failed to unpack capsule")?;

    let mut tarball_buf = Vec::new();
    {
        let encoder =
            flate2::write::GzEncoder::new(&mut tarball_buf, flate2::Compression::default());
        let mut tar_builder = tar::Builder::new(encoder);

        for (path, data) in &files {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            let pkg_path = format!("package/{}", path);
            tar_builder
                .append_data(&mut header, &pkg_path, &data[..])
                .context("failed to write tarball entry")?;
        }

        let encoder = tar_builder.into_inner().context("failed to finalize tar")?;
        encoder.finish().context("failed to finish gzip")?;
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("bnpm-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to create npm publish client")?;

    let npm_registry =
        std::env::var("NPM_REGISTRY").unwrap_or_else(|_| "https://registry.npmjs.org".to_string());

    let tag = npm_tag.as_deref().unwrap_or("latest");

    let publish_url = format!("{}/{}", npm_registry.trim_end_matches('/'), name);

    let attachment_name = format!("{}-{}.tgz", name, version);
    let attachment_data = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&tarball_buf)
    };
    let attachment_len = tarball_buf.len();

    let mut dist_tags = serde_json::Map::new();
    dist_tags.insert(tag.to_string(), serde_json::Value::String(version.to_string()));

    let metadata = serde_json::json!({
        "name": name,
        "version": version,
        "_attachments": {
            attachment_name: {
                "content_type": "application/octet-stream",
                "data": attachment_data,
                "length": attachment_len,
            }
        },
        "dist-tags": dist_tags,
    });

    let resp = client
        .put(&publish_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&metadata)
        .send()
        .context("npm publish request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        anyhow::bail!("npm publish returned HTTP {}: {}", status, body);
    }

    Ok(())
}
