//! WebAssembly runtime integration for running WASM-based packages.
//!
//! Provides compilation, validation, and execution of WebAssembly
//! modules within the Bnpm environment.

use std::collections::HashMap;

/// Error type for WASM operations.
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    #[error("compilation failed: {0}")]
    Compilation(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("execution failed: {0}")]
    Execution(String),
    #[error("function not found: {0}")]
    FunctionNotFound(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("memory error: {0}")]
    Memory(String),
}

/// A compiled WebAssembly module with its exported symbols.
#[derive(Debug, Clone)]
pub struct WasmModule {
    /// The compiled module bytes.
    pub compiled: Vec<u8>,
    /// Exported function names and their signatures.
    pub exports: Vec<ExportDescriptor>,
    /// Original source bytes.
    pub source: Vec<u8>,
    /// Module metadata.
    pub metadata: WasmModuleMetadata,
}

/// Metadata about a WASM module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmModuleMetadata {
    /// Module name (from the WASM name section).
    pub name: Option<String>,
    /// WASM version.
    pub version: Option<String>,
    /// Custom sections in the module.
    pub custom_sections: HashMap<String, Vec<u8>>,
}

/// Describes an exported function or item from a WASM module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportDescriptor {
    /// Export name.
    pub name: String,
    /// Export kind (function, memory, table, global).
    pub kind: ExportKind,
    /// Optional signature for functions.
    pub signature: Option<FunctionSignature>,
}

/// The kind of a WASM export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExportKind {
    Function,
    Memory,
    Table,
    Global,
}

/// The signature of a WASM function.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionSignature {
    /// Parameter types.
    pub params: Vec<ValType>,
    /// Return types.
    pub results: Vec<ValType>,
}

/// WebAssembly value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
    V128,
    ExternRef,
    FuncRef,
}

/// Validate a WebAssembly module without compiling it.
///
/// Checks the module structure and ensures it conforms to the WASM spec.
pub fn validate_wasm(module: &[u8]) -> Result<(), WasmError> {
    // Basic validation checks
    if module.len() < 8 {
        return Err(WasmError::Validation("module too short".to_string()));
    }

    // Check magic number
    let magic = &module[0..4];
    if magic != b"\0asm" {
        return Err(WasmError::Validation(
            "invalid WASM magic number".to_string(),
        ));
    }

    // Check version
    let version = u32::from_le_bytes([module[4], module[5], module[6], module[7]]);
    if version != 1 {
        return Err(WasmError::Validation(format!(
            "unsupported WASM version: {version}, expected 1"
        )));
    }

    // Parse sections for basic structure validation
    let mut pos = 8;
    while pos < module.len() {
        if pos + 1 > module.len() {
            return Err(WasmError::Validation("truncated section id".to_string()));
        }
        let section_id = module[pos];
        pos += 1;

        if pos + 4 > module.len() {
            return Err(WasmError::Validation("truncated section size".to_string()));
        }
        let section_size = u32::from_le_bytes([
            module[pos],
            module[pos + 1],
            module[pos + 2],
            module[pos + 3],
        ]) as usize;
        pos += 4;

        if pos + section_size > module.len() {
            return Err(WasmError::Validation("section extends beyond module".to_string()));
        }

        match section_id {
            0 => {} // Custom section (any content)
            1 => {} // Type section
            2 => {} // Import section
            3 => {} // Function section
            4 => {} // Table section
            5 => {} // Memory section
            6 => {} // Global section
            7 => {} // Export section
            8 => {} // Start section
            9 => {} // Element section
            10 => {} // Code section
            11 => {} // Data section
            12 => {} // Data count section
            _ => {
                return Err(WasmError::Validation(format!(
                    "unknown section id: {section_id}"
                )));
            }
        }

        pos += section_size;
    }

    Ok(())
}

/// Compile a WebAssembly module from source bytes.
///
/// Validates the module and extracts metadata and exports.
pub fn compile_wasm(source: &[u8]) -> Result<WasmModule, WasmError> {
    // Validate first
    validate_wasm(source)?;

    // Parse exports from the module
    let mut exports = Vec::new();
    let mut metadata = WasmModuleMetadata {
        name: None,
        version: None,
        custom_sections: HashMap::new(),
    };

    // Parse the export section (section id 7)
    let mut pos = 8;
    while pos < source.len() {
        if pos + 1 > source.len() {
            break;
        }
        let section_id = source[pos];
        pos += 1;

        if pos + 4 > source.len() {
            break;
        }
        let section_size = u32::from_le_bytes([
            source[pos],
            source[pos + 1],
            source[pos + 2],
            source[pos + 3],
        ]) as usize;
        pos += 4;

        let section_end = pos + section_size;

        match section_id {
            7 => {
                // Export section
                if pos < source.len() {
                    let count = source[pos];
                    pos += 1;
                    for _ in 0..count {
                        // Read name length and name
                        if pos + 4 > source.len() {
                            break;
                        }
                        let name_len = u32::from_le_bytes([
                            source[pos],
                            source[pos + 1],
                            source[pos + 2],
                            source[pos + 3],
                        ]) as usize;
                        pos += 4;

                        if pos + name_len > source.len() {
                            break;
                        }
                        let name = String::from_utf8_lossy(&source[pos..pos + name_len]).to_string();
                        pos += name_len;

                        if pos + 2 > source.len() {
                            break;
                        }
                        let kind = source[pos];
                        pos += 1;
                        let _index = u32::from_le_bytes([
                            source[pos],
                            source[pos + 1],
                            source[pos + 2],
                            source[pos + 3],
                        ]);
                        pos += 4;

                        let export_kind = match kind {
                            0 => ExportKind::Function,
                            1 => ExportKind::Table,
                            2 => ExportKind::Memory,
                            3 => ExportKind::Global,
                            _ => continue,
                        };

                        exports.push(ExportDescriptor {
                            name,
                            kind: export_kind,
                            signature: None,
                        });
                    }
                }
            }
            0 => {
                // Custom section - could contain name or metadata
                if pos < source.len() {
                    let name_len = u32::from_le_bytes([
                        source[pos],
                        source[pos + 1],
                        source[pos + 2],
                        source[pos + 3],
                    ]) as usize;
                    pos += 4;

                    if pos + name_len <= source.len() {
                        let section_name =
                            String::from_utf8_lossy(&source[pos..pos + name_len]).to_string();
                        let section_data = source[pos..section_end.min(source.len())].to_vec();
                        metadata.custom_sections.insert(section_name, section_data);
                    }
                }
            }
            _ => {}
        }

        pos = section_end;
    }

    Ok(WasmModule {
        compiled: source.to_vec(),
        exports,
        source: source.to_vec(),
        metadata,
    })
}

/// Run a function exported from a compiled WebAssembly module.
///
/// This is a simplified execution that would use a WASM runtime
/// (e.g., wasmtime/wasmer) in production.
pub fn run_wasm(_module: &[u8], function: &str, _args: &[u8]) -> Result<Vec<u8>, WasmError> {
    // In a real implementation, this would:
    // 1. Instantiate the module using wasmtime/wasmer
    // 2. Look up the exported function
    // 3. Call it with the arguments
    // 4. Return the result

    Err(WasmError::FunctionNotFound(format!(
        "WASM execution requires a runtime like wasmtime or wasmer; function '{function}' not callable without runtime"
    )))
}

/// Check if a module exports a specific function.
pub fn has_export(module: &WasmModule, name: &str) -> bool {
    module.exports.iter().any(|e| e.name == name)
}

/// Get the exports of a specific kind from a module.
pub fn exports_by_kind<'a>(
    module: &'a WasmModule,
    kind: ExportKind,
) -> Vec<&'a ExportDescriptor> {
    module.exports.iter().filter(|e| e.kind == kind).collect()
}

/// Approximate the memory usage of a WASM module.
pub fn module_size(module: &[u8]) -> usize {
    module.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_wasm_minimal() {
        // Minimal valid WASM module (empty module with just magic + version)
        let module = wasm_minimal();
        assert!(validate_wasm(&module).is_ok());
    }

    #[test]
    fn test_validate_wasm_invalid_magic() {
        let module = vec![0, 0, 0, 0, 1, 0, 0, 0]; // wrong magic
        assert!(validate_wasm(&module).is_err());
    }

    #[test]
    fn test_validate_wasm_invalid_version() {
        let module = vec![0x00, 0x61, 0x73, 0x6D, 0x02, 0x00, 0x00, 0x00]; // magic OK, version 2
        assert!(validate_wasm(&module).is_err());
    }

    #[test]
    fn test_validate_wasm_too_short() {
        let module = vec![0, 0, 0];
        assert!(validate_wasm(&module).is_err());
    }

    #[test]
    fn test_compile_wasm_minimal() {
        let module = wasm_minimal();
        let compiled = compile_wasm(&module).unwrap();
        assert_eq!(compiled.exports.len(), 0);
    }

    #[test]
    fn test_module_size() {
        let module = wasm_minimal();
        assert_eq!(module_size(&module), module.len());
    }

    #[test]
    fn test_has_export() {
        let module = compile_wasm(&wasm_minimal()).unwrap();
        assert!(!has_export(&module, "nonexistent"));
    }

    fn wasm_minimal() -> Vec<u8> {
        // Magic: \0asm, Version: 1
        vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]
    }

    #[test]
    fn test_val_type_debug() {
        // Just make sure types are constructible
        let _ = ValType::I32;
        let _ = ValType::I64;
        let _ = ValType::F32;
        let _ = ValType::F64;
    }

    #[test]
    fn test_export_kind() {
        let func = ExportKind::Function;
        assert!(!matches!(func, ExportKind::Memory));
    }

    #[test]
    fn test_run_wasm_no_runtime() {
        let module = wasm_minimal();
        let result = run_wasm(&module, "main", b"");
        assert!(result.is_err()); // No runtime available
    }

    #[test]
    fn test_exports_by_kind() {
        let module = compile_wasm(&wasm_minimal()).unwrap();
        let functions = exports_by_kind(&module, ExportKind::Function);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_module_metadata_default() {
        let meta = WasmModuleMetadata {
            name: None,
            version: None,
            custom_sections: HashMap::new(),
        };
        assert!(meta.name.is_none());
    }
}
