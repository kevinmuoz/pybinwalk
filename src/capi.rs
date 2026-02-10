use binwalk::Binwalk;
use serde::Serialize;
use std::collections::HashSet;
use std::ffi::{c_char, c_uchar, CStr, CString};

#[derive(Serialize)]
struct ScanResult {
    offset: usize,
    id: String,
    size: usize,
    name: String,
    confidence: u8,
    description: String,
    always_display: bool,
    extraction_declined: bool,
}

#[derive(Serialize)]
struct ScanResponse {
    success: bool,
    error: Option<String>,
    file_map: Vec<ScanResult>,
}

#[no_mangle]
pub extern "C" fn bw_scan(data: *const c_uchar, len: usize) -> *mut c_char {
    let response = match std::panic::catch_unwind(|| {
        let slice = checked_slice(data, len)?;
        let bw = Binwalk::new();
        Ok(scan_with(&bw, slice))
    }) {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => ScanResponse {
            success: false,
            error: Some(e),
            file_map: vec![],
        },
        Err(_) => ScanResponse {
            success: false,
            error: Some("panic during scan".into()),
            file_map: vec![],
        },
    };

    to_json_ptr(&response)
}

#[no_mangle]
pub extern "C" fn bw_scan_with_options(
    data: *const c_uchar,
    len: usize,
    options_json: *const c_char,
) -> *mut c_char {
    let response = match std::panic::catch_unwind(|| {
        let slice = checked_slice(data, len)?;
        let options = parse_options(options_json)?;

        let bw = Binwalk::configure(
            None,
            None,
            options.include,
            options.exclude,
            None,
            options.search_all,
        )
        .map_err(|_| "configure failed".to_string())?;

        Ok(scan_with(&bw, slice))
    }) {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => ScanResponse {
            success: false,
            error: Some(e),
            file_map: vec![],
        },
        Err(_) => ScanResponse {
            success: false,
            error: Some("panic during scan".into()),
            file_map: vec![],
        },
    };

    to_json_ptr(&response)
}

#[no_mangle]
pub extern "C" fn bw_version() -> *mut c_char {
    #[derive(Serialize)]
    struct VersionResponse {
        success: bool,
        error: Option<String>,
        version: &'static str,
        binwalk_version: &'static str,
    }

    let v = VersionResponse {
        success: true,
        error: None,
        version: env!("CARGO_PKG_VERSION"),
        binwalk_version: "3.1.0",
    };

    to_json_ptr(&v)
}

#[no_mangle]
pub extern "C" fn bw_list_signatures() -> *mut c_char {
    #[derive(Serialize)]
    struct SignatureInfo {
        name: String,
        short: bool,
    }

    #[derive(Serialize)]
    struct SignatureListResponse {
        success: bool,
        error: Option<String>,
        count: usize,
        signatures: Vec<SignatureInfo>,
    }

    let response = match std::panic::catch_unwind(|| {
        let bw = Binwalk::new();
        let short_names: HashSet<String> =
            bw.short_signatures.iter().map(|s| s.name.clone()).collect();

        let mut signatures: Vec<SignatureInfo> = bw
            .extractor_lookup_table
            .keys()
            .map(|name| SignatureInfo {
                name: name.clone(),
                short: short_names.contains(name),
            })
            .collect();

        signatures.sort_by(|a, b| a.name.cmp(&b.name));

        SignatureListResponse {
            success: true,
            error: None,
            count: signatures.len(),
            signatures,
        }
    }) {
        Ok(r) => r,
        Err(_) => SignatureListResponse {
            success: false,
            error: Some("panic during signature enumeration".into()),
            count: 0,
            signatures: vec![],
        },
    };

    to_json_ptr(&response)
}

#[no_mangle]
pub extern "C" fn bw_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

fn to_json_ptr<T: Serialize>(value: &T) -> *mut c_char {
    let json = serde_json::to_string(value)
        .unwrap_or_else(|_| r#"{"success":false,"error":"serialization failed"}"#.into());
    CString::new(json)
        .unwrap_or_else(|_| {
            CString::new(r#"{"success":false,"error":"null byte in json"}"#).unwrap()
        })
        .into_raw()
}
fn checked_slice<'a>(data: *const c_uchar, len: usize) -> Result<&'a [u8], String> {
    if data.is_null() || len == 0 {
        return Err("invalid input: nulll pointer or zero length".to_string());
    }
    Ok(unsafe { std::slice::from_raw_parts(data, len) })
}

fn scan_with(bw: &Binwalk, slice: &[u8]) -> ScanResponse {
    let mut file_map: Vec<ScanResult> = bw
        .scan(slice)
        .into_iter()
        .map(|r| ScanResult {
            offset: r.offset,
            id: r.id,
            size: r.size,
            name: r.name,
            confidence: r.confidence,
            description: r.description,
            always_display: r.always_display,
            extraction_declined: r.extraction_declined,
        })
        .collect();

    file_map.sort_by_key(|r| r.offset);

    ScanResponse {
        success: true,
        error: None,
        file_map,
    }
}

fn parse_options(options_json: *const c_char) -> Result<Options, String> {
    if options_json.is_null() {
        return Ok(Options::default());
    }
    let opts_str = unsafe { CStr::from_ptr(options_json) }
        .to_str()
        .map_err(|_| "invalid options_json: not valid UTF-8".to_string())?;
    serde_json::from_str(opts_str).map_err(|_| "invalid options_json".to_string())
}

#[derive(serde::Deserialize, Default)]
struct Options {
    #[serde(default)]
    include: Option<Vec<String>>,
    #[serde(default)]
    exclude: Option<Vec<String>>,
    #[serde(default)]
    search_all: bool,
}
