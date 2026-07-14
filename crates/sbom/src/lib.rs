use std::collections::BTreeMap;

use dash_lockfile::Lockfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbomFormat {
    Spdx,
    CycloneDx,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpdxLicense {
    pub id: String,
    pub name: String,
    pub is_osi_approved: bool,
    pub is_fsf_libre: bool,
    pub reference: String,
}

impl SpdxLicense {
    pub fn unknown() -> Self {
        Self {
            id: "NOASSERTION".to_string(),
            name: "Unknown".to_string(),
            is_osi_approved: false,
            is_fsf_libre: false,
            reference: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicenseChange {
    pub package: String,
    pub version: String,
    pub old_license: Option<String>,
    pub new_license: String,
}

pub fn generate_sbom(lockfile: &Lockfile, format: SbomFormat) -> String {
    match format {
        SbomFormat::Spdx => generate_spdx(lockfile),
        SbomFormat::CycloneDx => generate_cyclonedx(lockfile),
    }
}

fn generate_spdx(lockfile: &Lockfile) -> String {
    let created = chrono::Utc::now().to_rfc3339();
    let mut packages = Vec::new();
    let mut pkg_ids = Vec::new();

    for (spdx_idx, pkg) in lockfile.package.iter().enumerate() {
        let spdx_id = format!("SPDXRef-Package-{}", spdx_idx);
        let license = resolve_spdx_license(
            pkg.provenance.as_ref().map(|p| &*p.publisher).unwrap_or("NOASSERTION"),
        )
        .id;
        pkg_ids.push(spdx_id.clone());
        packages.push(serde_json::json!({
            "name": pkg.name,
            "versionInfo": pkg.version,
            "SPDXID": spdx_id,
            "downloadLocation": format!("cid:{}", pkg.cid),
            "licenseConcluded": license,
            "checksums": [{
                "algorithm": "BLAKE3",
                "checksumValue": pkg.cid.strip_prefix("b3:").unwrap_or(&pkg.cid)
            }],
            "supplier": format!("Organization: {}", pkg.registry),
            "filesAnalyzed": false
        }));
    }

    let mut relationships = Vec::new();
    for (spdx_idx, pkg) in lockfile.package.iter().enumerate() {
        let spdx_id = format!("SPDXRef-Package-{}", spdx_idx);
        if pkg.name
            == lockfile
                .root
                .strip_suffix("@0.0.0")
                .unwrap_or(&lockfile.root)
                .trim_end_matches('@')
        {
            continue;
        }
        relationships.push(serde_json::json!({
            "spdxElementId": "SPDXRef-DOCUMENT",
            "relatedSpdxElement": spdx_id,
            "relationshipType": "DEPENDS_ON"
        }));
    }

    let doc = serde_json::json!({
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": format!("dash.lock - {}", lockfile.root),
        "creationInfo": {
            "created": created,
            "creators": ["Tool: dash-sbom-1.0.0"]
        },
        "documentNamespace": format!("https://dash.dev/sbom/{}", lockfile.lock_digest),
        "packages": packages,
        "relationships": relationships
    });

    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

fn generate_cyclonedx(lockfile: &Lockfile) -> String {
    let created = chrono::Utc::now().to_rfc3339();
    let mut components = Vec::new();

    for pkg in &lockfile.package {
        let lic = resolve_spdx_license(
            pkg.provenance.as_ref().map(|p| &*p.publisher).unwrap_or("NOASSERTION"),
        );
        let licenses = if lic.id == "NOASSERTION" {
            vec![]
        } else {
            vec![serde_json::json!({
                "license": {
                    "id": lic.id,
                    "name": lic.name
                }
            })]
        };

        components.push(serde_json::json!({
            "type": "library",
            "name": pkg.name,
            "version": pkg.version,
            "bom-ref": format!("pkg:dash/{}{}", pkg.name, if pkg.version.starts_with('@') { "" } else { "@" }),
            "hashes": [{
                "alg": "BLAKE3",
                "content": pkg.cid.strip_prefix("b3:").unwrap_or(&pkg.cid)
            }],
            "licenses": licenses,
            "publisher": pkg.registry,
            "supplier": {
                "name": pkg.registry
            },
            "evidence": {
                "identity": {
                    "field": "purl",
                    "confidence": 1.0,
                    "methods": [{
                        "technique": "manifest-analysis",
                        "value": format!("{}@{}", pkg.name, pkg.version)
                    }]
                }
            }
        }));
    }

    let doc = serde_json::json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "serialNumber": format!("urn:uuid:{}", lockfile.lock_digest),
        "version": 1,
        "metadata": {
            "timestamp": created,
            "tools": [{
                "vendor": "Dash",
                "name": "dash-sbom",
                "version": "1.0.0"
            }],
            "component": {
                "type": "application",
                "name": lockfile.root.clone(),
                "version": "1.0.0"
            }
        },
        "components": components
    });

    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

pub fn resolve_spdx_license(expression: &str) -> SpdxLicense {
    let known = [
        ("MIT", "MIT License", true, true),
        ("Apache-2.0", "Apache License 2.0", true, true),
        ("BSD-2-Clause", "BSD 2-Clause \"Simplified\" License", true, true),
        ("BSD-3-Clause", "BSD 3-Clause \"New\" or \"Revised\" License", true, true),
        ("ISC", "ISC License", true, true),
        ("Unlicense", "The Unlicense", true, true),
        ("Zlib", "zlib License", true, true),
        ("MPL-2.0", "Mozilla Public License 2.0", true, true),
        ("LGPL-2.1-only", "GNU Lesser General Public License v2.1 only", false, true),
        ("LGPL-2.1-or-later", "GNU Lesser General Public License v2.1 or later", false, true),
        ("LGPL-3.0-only", "GNU Lesser General Public License v3.0 only", false, true),
        ("LGPL-3.0-or-later", "GNU Lesser General Public License v3.0 or later", false, true),
        ("GPL-2.0-only", "GNU General Public License v2.0 only", false, true),
        ("GPL-2.0-or-later", "GNU General Public License v2.0 or later", false, true),
        ("GPL-3.0-only", "GNU General Public License v3.0 only", false, true),
        ("GPL-3.0-or-later", "GNU General Public License v3.0 or later", false, true),
        ("AGPL-3.0-only", "GNU Affero General Public License v3.0 only", false, true),
        ("AGPL-3.0-or-later", "GNU Affero General Public License v3.0 or later", false, true),
        ("LGPL-2.0-only", "GNU Library General Public License v2 only", false, true),
        ("LGPL-2.0-or-later", "GNU Library General Public License v2 or later", false, true),
        ("MPL-1.1", "Mozilla Public License 1.1", true, true),
        ("CDDL-1.0", "Common Development and Distribution License 1.0", true, false),
        ("EPL-2.0", "Eclipse Public License 2.0", true, true),
        ("CC0-1.0", "Creative Commons Zero v1.0 Universal", false, true),
        ("CC-BY-4.0", "Creative Commons Attribution 4.0 International", false, true),
        ("CC-BY-SA-4.0", "Creative Commons Attribution Share Alike 4.0 International", false, true),
        ("0BSD", "BSD Zero Clause License", true, true),
        ("PostgreSQL", "PostgreSQL License", true, true),
        ("WTFPL", "Do What The F*ck You Want To Public License", true, false),
        ("Artistic-2.0", "Artistic License 2.0", true, true),
        ("Python-2.0", "Python License 2.0", false, true),
        ("BSL-1.0", "Boost Software License 1.0", true, true),
    ];

    let cleaned = expression.trim();
    for (id, name, osi, fsf) in &known {
        if cleaned == *id || cleaned.starts_with(id) || cleaned.contains(id) {
            return SpdxLicense {
                id: id.to_string(),
                name: name.to_string(),
                is_osi_approved: *osi,
                is_fsf_libre: *fsf,
                reference: format!("https://spdx.org/licenses/{}.html", id),
            };
        }
    }
    SpdxLicense::unknown()
}

pub fn detect_license_changes(
    old_lockfile: &Lockfile,
    new_lockfile: &Lockfile,
) -> Vec<LicenseChange> {
    let mut changes = Vec::new();
    let old_map: BTreeMap<&str, &dash_lockfile::LockPackage> =
        old_lockfile.package.iter().map(|p| (p.name.as_str(), p)).collect();

    for new_pkg in &new_lockfile.package {
        if let Some(old_pkg) = old_map.get(new_pkg.name.as_str()) {
            let old_lic = old_pkg
                .provenance
                .as_ref()
                .map(|_| "NOASSERTION".to_string())
                .unwrap_or_else(|| "NOASSERTION".to_string());
            let new_lic = new_pkg
                .provenance
                .as_ref()
                .map(|_| "NOASSERTION".to_string())
                .unwrap_or_else(|| "NOASSERTION".to_string());

            if old_pkg.cid != new_pkg.cid && old_lic != new_lic {
                changes.push(LicenseChange {
                    package: new_pkg.name.clone(),
                    version: new_pkg.version.clone(),
                    old_license: if old_pkg.cid != new_pkg.cid { Some(old_lic) } else { None },
                    new_license: new_lic,
                });
            }
        } else {
            changes.push(LicenseChange {
                package: new_pkg.name.clone(),
                version: new_pkg.version.clone(),
                old_license: None,
                new_license: "NOASSERTION".to_string(),
            });
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use dash_lockfile::{LockBuild, LockPackage, Provenance};

    fn fixture_lockfile() -> Lockfile {
        let mut lf = Lockfile::with_root("@acme/app@0.0.0");
        lf.insert(LockPackage {
            name: "zod".into(),
            version: "3.23.8".into(),
            cid: "b3:qm7f2x3a4b5c6d7e8f9g0h1i2j3k4l5m6n7o8p9q0r1s2t3u4v5w6x7y8z".into(),
            resolved_from: "^3.23".into(),
            registry: "dash:default".into(),
            provenance: Some(Provenance {
                log_index: 1,
                slsa_level: 3,
                publisher: "did:dash:zod".into(),
            }),
            capabilities: None,
            build: None,
        });
        lf.insert(LockPackage {
            name: "express".into(),
            version: "4.21.0".into(),
            cid: "b3:aa11bb22cc33dd44ee55ff66gg77hh88ii99jj00kk11ll22mm33nn44oo55pp".into(),
            resolved_from: "^4.21".into(),
            registry: "dash:default".into(),
            provenance: Some(Provenance {
                log_index: 2,
                slsa_level: 2,
                publisher: "did:dash:express".into(),
            }),
            capabilities: None,
            build: Some(LockBuild {
                tool: "dash:build@1.0.0".into(),
                output_cid: "b3:out123".into(),
                hermetic: true,
            }),
        });
        lf.graph
            .insert("@acme/app@0.0.0".into(), vec!["zod@3.23.8".into(), "express@4.21.0".into()]);
        lf.graph.insert("zod@3.23.8".into(), vec![]);
        lf.graph.insert("express@4.21.0".into(), vec![]);
        lf.finalize();
        lf
    }

    #[test]
    fn test_generate_spdx() {
        let lf = fixture_lockfile();
        let sbom = generate_sbom(&lf, SbomFormat::Spdx);
        assert!(sbom.contains("SPDX-2.3"));
        assert!(sbom.contains("zod"));
        assert!(sbom.contains("express"));
        assert!(sbom.contains("SPDXRef-Package-0"));
        assert!(sbom.contains("cid:"));
        assert!(sbom.contains("BLAKE3"));
    }

    #[test]
    fn test_generate_cyclonedx() {
        let lf = fixture_lockfile();
        let sbom = generate_sbom(&lf, SbomFormat::CycloneDx);
        assert!(sbom.contains("CycloneDX"));
        assert!(sbom.contains("1.6"));
        assert!(sbom.contains("zod"));
        assert!(sbom.contains("express"));
        assert!(sbom.contains("bom-ref"));
        assert!(sbom.contains("BLAKE3"));
    }

    #[test]
    fn test_resolve_spdx_license_known() {
        let lic = resolve_spdx_license("MIT");
        assert_eq!(lic.id, "MIT");
        assert!(lic.is_osi_approved);
    }

    #[test]
    fn test_resolve_spdx_license_unknown() {
        let lic = resolve_spdx_license("PROPRIETARY-1.0");
        assert_eq!(lic.id, "NOASSERTION");
    }

    #[test]
    fn test_resolve_spdx_license_apache() {
        let lic = resolve_spdx_license("Apache-2.0");
        assert_eq!(lic.id, "Apache-2.0");
    }

    #[test]
    fn test_resolve_spdx_license_gpl() {
        let lic = resolve_spdx_license("GPL-3.0-only");
        assert_eq!(lic.id, "GPL-3.0-only");
        assert!(!lic.is_osi_approved);
    }

    #[test]
    fn test_generate_sbom_contains_correct_packages() {
        let lf = fixture_lockfile();
        let spdx = generate_sbom(&lf, SbomFormat::Spdx);
        let parsed: serde_json::Value = serde_json::from_str(&spdx).unwrap();
        let packages = parsed["packages"].as_array().unwrap();
        assert_eq!(packages.len(), 2);
        let names: Vec<&str> = packages.iter().map(|p| p["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"zod"));
        assert!(names.contains(&"express"));
    }

    #[test]
    fn test_detect_license_changes_new_package() {
        let old = Lockfile::with_root("@app@0.0.0");
        let mut new = Lockfile::with_root("@app@0.0.0");
        new.insert(LockPackage {
            name: "new-pkg".into(),
            version: "1.0.0".into(),
            cid: "b3:new123".into(),
            resolved_from: "^1.0".into(),
            registry: "dash:default".into(),
            provenance: None,
            capabilities: None,
            build: None,
        });
        new.finalize();
        let changes = detect_license_changes(&old, &new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].package, "new-pkg");
    }
}
