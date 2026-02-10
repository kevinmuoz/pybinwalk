use binwalk::extractors::common::{
    Extractor as ExtractorInner, ExtractorType as ExtractorTypeInner,
};
use binwalk::signatures::common::SignatureResult as SignatureResultInner;
use binwalk::{AnalysisResults as AnalysisResultsInner, Binwalk as BinwalkInner};
use pyo3::exceptions::{PyIOError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyBytesMethods};
use std::collections::HashMap;


#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct ExtractorType {
    inner: ExtractorTypeInner,
}

#[pymethods]
impl ExtractorType {
    #[getter]
    fn kind(&self) -> String {
        match &self.inner {
            ExtractorTypeInner::External(_) => "external".to_string(),
            ExtractorTypeInner::Internal(_) => "internal".to_string(),
            ExtractorTypeInner::None => "none".to_string(),
        }
    }

    #[getter]
    fn value(&self) -> Option<String> {
        match &self.inner {
            ExtractorTypeInner::External(s) => Some(s.clone()),
            ExtractorTypeInner::Internal(_) => Some(format!("{:?}", self.inner)),
            ExtractorTypeInner::None => None,
        }
    }

    fn __repr__(&self) -> String {
        match &self.inner {
            ExtractorTypeInner::External(cmd) => {
                format!("ExtractorType(kind='external', value='{}')", cmd)
            }
            ExtractorTypeInner::Internal(_) => {
                format!("ExtractorType(kind='internal', value='{:?}')", self.inner)
            }
            ExtractorTypeInner::None => "ExtractorType(kind='none', value=None)".to_string(),
        }
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct Extractor {
    inner: ExtractorInner,
}

#[pymethods]
impl Extractor {
    #[getter]
    fn utility<'py>(&self, py: Python<'py>) -> PyResult<Py<ExtractorType>> {
        extractor_type_to_py(py, &self.inner.utility)
    }

    #[getter]
    fn extension(&self) -> String {
        self.inner.extension.clone()
    }

    #[getter]
    fn arguments(&self) -> Vec<String> {
        self.inner.arguments.clone()
    }

    #[getter]
    fn exit_codes(&self) -> Vec<i32> {
        self.inner.exit_codes.clone()
    }

    #[getter]
    fn do_not_recurse(&self) -> bool {
        self.inner.do_not_recurse
    }

    fn __repr__(&self) -> String {
        format!(
            "Extractor(extension='{}', arguments={}, exit_codes={}, do_not_recurse={})",
            self.inner.extension,
            format!("{:?}", self.inner.arguments),
            format!("{:?}", self.inner.exit_codes),
            self.inner.do_not_recurse
        )
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct SignatureResult {
    inner: SignatureResultInner,
}

#[pymethods]
impl SignatureResult {
    #[getter]
    fn offset(&self) -> u64 {
        self.inner.offset as u64
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn size(&self) -> u64 {
        self.inner.size as u64
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn confidence(&self) -> u8 {
        self.inner.confidence
    }

    #[getter]
    fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    fn always_display(&self) -> bool {
        self.inner.always_display
    }

    #[getter]
    fn extraction_declined(&self) -> bool {
        self.inner.extraction_declined
    }

    #[getter]
    fn preferred_extractor<'py>(&self, py: Python<'py>) -> PyResult<Option<Py<Extractor>>> {
        match &self.inner.preferred_extractor {
            Some(extractor) => Ok(Some(extractor_to_py(py, extractor)?)),
            None => Ok(None),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "SignatureResult(offset={}, size={}, name='{}', confidence={})",
            self.inner.offset, self.inner.size, self.inner.name, self.inner.confidence
        )
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct ExtractionResult {
    inner: binwalk::extractors::common::ExtractionResult,
}

#[pymethods]
impl ExtractionResult {
    #[getter]
    fn size(&self) -> Option<u64> {
        self.inner.size.map(|s| s as u64)
    }

    #[getter]
    fn success(&self) -> bool {
        self.inner.success
    }

    #[getter]
    fn extractor(&self) -> String {
        self.inner.extractor.clone()
    }

    #[getter]
    fn do_not_recurse(&self) -> bool {
        self.inner.do_not_recurse
    }

    #[getter]
    fn output_directory(&self) -> String {
        self.inner.output_directory.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "ExtractionResult(success={}, extractor='{}', output_directory='{}')",
            self.inner.success, self.inner.extractor, self.inner.output_directory
        )
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct AnalysisResults {
    inner: AnalysisResultsInner,
}

#[pymethods]
impl AnalysisResults {
    #[getter]
    fn file_path(&self) -> String {
        self.inner.file_path.clone()
    }

    #[getter]
    fn file_map<'py>(&self, py: Python<'py>) -> PyResult<Vec<Py<SignatureResult>>> {
        self.inner
            .file_map
            .iter()
            .cloned()
            .map(|r| Py::new(py, SignatureResult { inner: r }))
            .collect()
    }

    #[getter]
    fn extractions<'py>(&self, py: Python<'py>) -> PyResult<HashMap<String, Py<ExtractionResult>>> {
        let mut map = HashMap::new();
        for (k, v) in &self.inner.extractions {
            map.insert(
                k.clone(),
                Py::new(py, ExtractionResult { inner: v.clone() })?,
            );
        }
        Ok(map)
    }

    fn __repr__(&self) -> String {
        format!(
            "AnalysisResults(file_path='{}', file_map_len={}, extractions_len={})",
            self.inner.file_path,
            self.inner.file_map.len(),
            self.inner.extractions.len()
        )
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct Binwalk {
    inner: BinwalkInner,
}

#[pymethods]
impl Binwalk {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(Self {
            inner: BinwalkInner::new(),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (
        target_file_name = None,
        output_directory = None,
        include = None,
        exclude = None,
        full_search = false
    ))]
    fn configure(
        target_file_name: Option<String>,
        output_directory: Option<String>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        full_search: bool,
    ) -> PyResult<Self> {
        let inner = BinwalkInner::configure(
            target_file_name,
            output_directory,
            include,
            exclude,
            None,
            full_search,
        )
        .map_err(|e| PyRuntimeError::new_err(format!("binwalk configure failed: debug={e:?}")))?;

        Ok(Self { inner })
    }

    fn scan_bytes<'py>(
        &self,
        py: Python<'py>,
        data: &Bound<'py, PyBytes>,
    ) -> PyResult<Vec<Py<SignatureResult>>> {
        // We copy here because we release the GIL below.
        let bytes = data.as_bytes().to_vec();
        let inner = self.inner.clone();
        let results = release_gil(py, move || inner.scan(&bytes));
        results
            .into_iter()
            .map(|r| Py::new(py, SignatureResult { inner: r }))
            .collect()
    }

    fn scan_path<'py>(
        &self,
        py: Python<'py>,
        file_path: String,
    ) -> PyResult<Vec<Py<SignatureResult>>> {
        let inner = self.inner.clone();
        let results = release_gil(py, move || {
            std::fs::read(&file_path).map(|data| inner.scan(&data))
        })
        .map_err(|e| PyIOError::new_err(format!("failed to read file: {e}")))?;

        results
            .into_iter()
            .map(|r| Py::new(py, SignatureResult { inner: r }))
            .collect()
    }

    fn extract_bytes<'py>(
        &self,
        py: Python<'py>,
        data: &Bound<'py, PyBytes>,
        file_path: String,
        file_map: Vec<Py<SignatureResult>>,
    ) -> PyResult<HashMap<String, Py<ExtractionResult>>> {
        let file_map_inner: Vec<SignatureResultInner> = file_map
            .iter()
            .map(|sr| sr.bind(py).borrow().inner.clone())
            .collect();

        let bytes = data.as_bytes().to_vec();
        let inner = self.inner.clone();
        let results = release_gil(py, move || {
            inner.extract(&bytes, &file_path, &file_map_inner)
        });

        let mut map = HashMap::new();
        for (k, v) in results {
            map.insert(k, Py::new(py, ExtractionResult { inner: v })?);
        }
        Ok(map)
    }

    fn analyze_path<'py>(
        &self,
        py: Python<'py>,
        target_file: String,
        do_extraction: bool,
    ) -> PyResult<AnalysisResults> {
        let inner = self.inner.clone();
        let results = release_gil(py, move || inner.analyze(&target_file, do_extraction));
        Ok(AnalysisResults { inner: results })
    }

    #[getter]
    fn signature_count(&self) -> u64 {
        self.inner.signature_count as u64
    }

    #[getter]
    fn base_target_file(&self) -> String {
        self.inner.base_target_file.clone()
    }

    #[getter]
    fn base_output_directory(&self) -> String {
        self.inner.base_output_directory.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "Binwalk(signature_count={}, base_target_file='{}')",
            self.inner.signature_count, self.inner.base_target_file
        )
    }
}

#[pyfunction]
fn version() -> PyResult<String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

fn extractor_type_to_py<'py>(
    py: Python<'py>,
    extractor_type: &ExtractorTypeInner,
) -> PyResult<Py<ExtractorType>> {
    Py::new(
        py,
        ExtractorType {
            inner: extractor_type.clone(),
        },
    )
}

fn extractor_to_py<'py>(py: Python<'py>, extractor: &ExtractorInner) -> PyResult<Py<Extractor>> {
    Py::new(
        py,
        Extractor {
            inner: extractor.clone(),
        },
    )
}

fn release_gil<T, F>(py: Python<'_>, f: F) -> T
where
    F: pyo3::marker::Ungil + FnOnce() -> T,
    T: pyo3::marker::Ungil,
{
    py.detach(f)
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Binwalk>()?;
    m.add_class::<SignatureResult>()?;
    m.add_class::<Extractor>()?;
    m.add_class::<ExtractorType>()?;
    m.add_class::<ExtractionResult>()?;
    m.add_class::<AnalysisResults>()?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
