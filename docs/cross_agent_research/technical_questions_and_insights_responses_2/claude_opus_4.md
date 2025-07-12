# 🛠️ Practical Implementation Solutions - Round 2

## 1. Testing Strategy Solutions

### GPU Testing Without Hardware

Here's a complete testing strategy using wgpu's software renderer:

```rust
// tests/common/gpu_test_utils.rs
use once_cell::sync::Lazy;
use std::sync::Arc;

pub struct TestGpuContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

static TEST_GPU: Lazy<TestGpuContext> = Lazy::new(|| {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true, // Force software rendering
                compatible_surface: None,
            })
            .await
            .expect("Failed to create test adapter");
            
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Test Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .expect("Failed to create test device");
            
        TestGpuContext {
            device: Arc::new(device),
            queue: Arc::new(queue),
        }
    })
});

// Trait for testable GPU operations
pub trait GpuOperations: Send + Sync {
    fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> wgpu::Buffer;
    fn create_shader_module(&self, desc: &wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule;
    fn submit(&self, commands: Vec<wgpu::CommandBuffer>);
}

// Real implementation
pub struct WgpuDevice {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

// Test implementation with memory tracking
pub struct MockGpuDevice {
    allocated_memory: AtomicU64,
    memory_limit: u64,
}

impl GpuOperations for MockGpuDevice {
    fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> wgpu::Buffer {
        let new_total = self.allocated_memory.fetch_add(desc.size, Ordering::SeqCst) + desc.size;
        if new_total > self.memory_limit {
            panic!("GPU memory limit exceeded in test");
        }
        // Return a real buffer from test context
        TEST_GPU.device.create_buffer(desc)
    }
}

// Example test
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_aggregation() {
        let gpu = MockGpuDevice {
            allocated_memory: AtomicU64::new(0),
            memory_limit: 1024 * 1024 * 1024, // 1GB limit for tests
        };
        
        let buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("test"),
            size: 1024,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        
        assert_eq!(gpu.allocated_memory.load(Ordering::SeqCst), 1024);
    }
}
```

### DuckDB Integration Testing

Efficient testing pattern with fixtures:

```rust
// tests/common/duckdb_fixtures.rs
use tempfile::TempDir;
use once_cell::sync::Lazy;

pub struct TestDatabase {
    _temp_dir: TempDir,
    conn: Connection,
}

impl TestDatabase {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        
        let conn = Connection::open(&db_path)?;
        
        // Configure for fast testing
        conn.execute_batch("
            SET threads = 1;
            SET memory_limit = '100MB';
            SET checkpoint_threshold = '1TB'; -- Disable checkpointing
        ")?;
        
        Ok(TestDatabase { _temp_dir: temp_dir, conn })
    }
    
    pub fn with_sample_data() -> Result<Self> {
        let mut db = Self::new()?;
        
        // Load pre-computed sample data
        db.conn.execute_batch(include_str!("../fixtures/sample_data.sql"))?;
        
        Ok(db)
    }
}

// Macro for async query testing with timeout
#[macro_export]
macro_rules! test_query_async {
    ($query:expr, $timeout_ms:expr) => {{
        tokio::time::timeout(
            Duration::from_millis($timeout_ms),
            tokio::task::spawn_blocking(move || {
                let db = TestDatabase::new()?;
                db.conn.query_arrow($query)
            })
        ).await
    }};
}

// Progress callback testing
pub struct MockProgressHandler {
    updates: Arc<Mutex<Vec<f64>>>,
}

impl ProgressHandler for MockProgressHandler {
    fn update(&self, progress: f64) {
        self.updates.lock().unwrap().push(progress);
    }
}
```

## 2. CLI User Experience

### Advanced Progress System

Using `indicatif` with custom multi-stage progress:

```rust
// src/cli/progress.rs
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct PikaProgress {
    multi: MultiProgress,
    main_bar: ProgressBar,
    sub_bars: Vec<ProgressBar>,
    json_mode: bool,
}

impl PikaProgress {
    pub fn new(json_mode: bool) -> Self {
        if json_mode {
            // JSON output mode for scripts
            return Self {
                multi: MultiProgress::new(),
                main_bar: ProgressBar::hidden(),
                sub_bars: vec![],
                json_mode: true,
            };
        }
        
        let multi = MultiProgress::new();
        let main_bar = multi.add(ProgressBar::new(100));
        
        main_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner} {prefix} [{bar:40}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▓░"),
        );
        
        Self { multi, main_bar, sub_bars: vec![], json_mode }
    }
    
    pub fn add_stage(&mut self, name: &str, total: u64) -> ProgressHandle {
        if self.json_mode {
            println!("{}", serde_json::json!({
                "event": "stage_start",
                "name": name,
                "total": total,
            }));
            return ProgressHandle::Json { name: name.to_string() };
        }
        
        let bar = self.multi.add(ProgressBar::new(total));
        bar.set_prefix(name.to_string());
        self.sub_bars.push(bar.clone());
        
        ProgressHandle::Bar(bar)
    }
}

pub enum ProgressHandle {
    Bar(ProgressBar),
    Json { name: String },
}

impl ProgressHandle {
    pub fn inc(&self, delta: u64) {
        match self {
            Self::Bar(bar) => bar.inc(delta),
            Self::Json { name } => {
                println!("{}", serde_json::json!({
                    "event": "progress",
                    "stage": name,
                    "delta": delta,
                }));
            }
        }
    }
}

// Graceful cancellation
pub struct CancellationHandler {
    token: CancellationToken,
}

impl CancellationHandler {
    pub fn new() -> Self {
        let token = CancellationToken::new();
        let token_clone = token.clone();
        
        ctrlc::set_handler(move || {
            eprintln!("\nGracefully cancelling operation...");
            token_clone.cancel();
        }).expect("Failed to set Ctrl+C handler");
        
        Self { token }
    }
}
```

### Configuration Management

Hierarchical config with `config` crate:

```rust
// src/cli/config.rs
use config::{Config, Environment, File};
use directories::ProjectDirs;

#[derive(Debug, Deserialize, Serialize)]
pub struct PikaConfig {
    pub database: DatabaseConfig,
    pub gpu: GpuConfig,
    pub memory: MemoryConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemoryConfig {
    #[serde(with = "humansize_serde")]
    pub max_memory: u64,
    #[serde(with = "humansize_serde")]
    pub gpu_memory_limit: u64,
}

impl PikaConfig {
    pub fn load() -> Result<Self> {
        let config_dir = ProjectDirs::from("com", "pikaplot", "pika")
            .map(|dirs| dirs.config_dir().to_owned())
            .unwrap_or_else(|| PathBuf::from("."));
            
        let config = Config::builder()
            // 1. Start with defaults
            .set_default("memory.max_memory", "8GB")?
            .set_default("gpu.device_index", -1i32)?
            
            // 2. Global config
            .add_source(File::from(config_dir.join("config.toml")).required(false))
            
            // 3. Local project config
            .add_source(File::from("pika.toml").required(false))
            
            // 4. Environment variables (PIKA_MEMORY_MAX_MEMORY)
            .add_source(Environment::with_prefix("PIKA").separator("_"))
            
            // 5. Build and deserialize
            .build()?;
            
        config.try_deserialize()
    }
}

// Human-friendly size parsing
mod humansize_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        parse_size(&s).map_err(serde::de::Error::custom)
    }
    
    pub fn serialize<S>(bytes: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let human = format_size(*bytes);
        serializer.serialize_str(&human)
    }
    
    fn parse_size(s: &str) -> Result<u64> {
        // "4GB" -> 4294967296
        // "512MB" -> 536870912
        // "1.5TB" -> 1649267441664
        parse_size::parse_size(s).map_err(|e| anyhow!("Invalid size: {}", e))
    }
}
```

## 3. Error Handling and Recovery

### Automatic Fallback System

```rust
// src/engine/fallback.rs
use std::sync::Arc;

pub struct FallbackRenderer {
    gpu_renderer: Option<Arc<GpuRenderer>>,
    cpu_renderer: Arc<CpuRenderer>,
    metrics: RenderMetrics,
}

impl FallbackRenderer {
    pub async fn render_with_fallback(
        &mut self,
        data: &PlotData,
        viewport: ViewportBounds,
    ) -> Result<RenderedPlot> {
        // Try GPU first if available
        if let Some(gpu) = &self.gpu_renderer {
            match self.try_gpu_render(gpu, data, viewport).await {
                Ok(result) => {
                    self.metrics.record_gpu_success();
                    return Ok(result);
                }
                Err(e) => {
                    warn!("GPU render failed, falling back to CPU: {}", e);
                    self.metrics.record_gpu_failure();
                    
                    // Disable GPU for future calls if too many failures
                    if self.metrics.gpu_failure_rate() > 0.5 {
                        warn!("Disabling GPU rendering due to high failure rate");
                        self.gpu_renderer = None;
                    }
                }
            }
        }
        
        // CPU fallback
        self.render_cpu_with_progress(data, viewport).await
    }
    
    async fn try_gpu_render(
        &self,
        gpu: &GpuRenderer,
        data: &PlotData,
        viewport: ViewportBounds,
    ) -> Result<RenderedPlot> {
        // Timeout GPU operations to prevent hangs
        tokio::time::timeout(
            Duration::from_secs(5),
            gpu.render(data, viewport)
        ).await?
    }
    
    async fn render_cpu_with_progress(
        &self,
        data: &PlotData,
        viewport: ViewportBounds,
    ) -> Result<RenderedPlot> {
        // Show progress for slow CPU rendering
        let progress = ProgressBar::new(data.point_count() as u64);
        progress.set_message("CPU rendering (slower than GPU)");
        
        let result = self.cpu_renderer.render_with_callback(data, viewport, |done| {
            progress.set_position(done as u64);
        }).await?;
        
        progress.finish_with_message("CPU rendering complete");
        Ok(result)
    }
}

// Automatic retry with resource reduction
pub struct AdaptiveExecutor {
    memory_limit: AtomicU64,
    batch_size: AtomicU32,
}

impl AdaptiveExecutor {
    pub async fn execute_adaptive<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut(u64, u32) -> BoxFuture<'static, Result<T>>,
    {
        let mut attempts = 0;
        let mut memory = self.memory_limit.load(Ordering::Relaxed);
        let mut batch = self.batch_size.load(Ordering::Relaxed);
        
        loop {
            match operation(memory, batch).await {
                Ok(result) => return Ok(result),
                Err(e) if e.is_memory_error() && attempts < 3 => {
                    attempts += 1;
                    memory = (memory as f64 * 0.75) as u64;
                    batch = (batch as f64 * 0.5) as u32;
                    
                    warn!("Retrying with reduced resources: memory={}, batch={}", 
                          format_size(memory), batch);
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

## 4. Performance Benchmarking

### Automated Benchmark Suite

```rust
// benches/suite.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pprof::criterion::{Output, PProfProfiler};

pub fn bench_csv_import(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_import");
    
    // Test different file sizes
    for size_mb in [1, 10, 100, 1000] {
        let csv_path = generate_test_csv(size_mb);
        
        group.throughput(criterion::Throughput::Bytes((size_mb * 1024 * 1024) as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}MB", size_mb)),
            &csv_path,
            |b, path| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        let engine = StorageEngine::new_temp().await.unwrap();
                        engine.import_csv(path, Default::default()).await.unwrap();
                        black_box(engine);
                    });
            },
        );
    }
    
    group.finish();
}

pub fn bench_gpu_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_aggregation");
    
    // Skip if no GPU available
    let gpu = match GpuRenderer::new() {
        Ok(gpu) => gpu,
        Err(_) => {
            eprintln!("Skipping GPU benchmarks - no GPU available");
            return;
        }
    };
    
    for points in [10_000, 100_000, 1_000_000, 10_000_000] {
        let data = generate_scatter_data(points);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}pts", points)),
            &data,
            |b, data| {
                b.iter(|| {
                    let result = gpu.aggregate_points(data, ViewportBounds::default());
                    black_box(result);
                });
            },
        );
    }
}

// Track performance over time
pub fn save_benchmark_results(results: &BenchmarkResults) {
    let history_path = "benchmarks/history.json";
    
    let mut history: Vec<HistoricalBenchmark> = 
        std::fs::read_to_string(&history_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
            
    history.push(HistoricalBenchmark {
        timestamp: Utc::now(),
        commit: std::env::var("GITHUB_SHA").ok(),
        results: results.clone(),
        system_info: SystemInfo::current(),
    });
    
    // Keep last 100 runs
    if history.len() > 100 {
        history.drain(0..history.len() - 100);
    }
    
    std::fs::write(&history_path, serde_json::to_string_pretty(&history).unwrap()).unwrap();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_csv_import, bench_gpu_aggregation
}
```

## 5. Data Streaming Architecture

### Streaming Iterator Implementation

```rust
// src/engine/streaming.rs
use futures::Stream;
use pin_project::pin_project;

#[pin_project]
pub struct ArrowStream {
    #[pin]
    inner: ReceiverStream<Result<RecordBatch>>,
    schema: Arc<Schema>,
    estimated_rows: Option<u64>,
}

impl ArrowStream {
    pub fn new(conn: Connection, query: String) -> Self {
        let (tx, rx) = mpsc::channel(4); // Small buffer for backpressure
        
        // Spawn background query executor
        tokio::task::spawn_blocking(move || {
            let mut result = conn.prepare(&query)?.query_arrow([])?;
            
            while let Some(batch) = result.next()? {
                if tx.blocking_send(Ok(batch)).is_err() {
                    break; // Receiver dropped
                }
            }
            
            Ok::<_, Error>(())
        });
        
        Self {
            inner: ReceiverStream::new(rx),
            schema,
            estimated_rows: None,
        }
    }
}

impl Stream for ArrowStream {
    type Item = Result<RecordBatch>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}

// Backpressure-aware GPU streaming
pub struct GpuStreamProcessor {
    gpu_queue: Arc<Mutex<GpuQueue>>,
    max_buffers: usize,
}

impl GpuStreamProcessor {
    pub async fn process_stream(
        &self,
        mut stream: ArrowStream,
        viewport: ViewportBounds,
    ) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(self.max_buffers));
        
        while let Some(batch) = stream.next().await {
            let batch = batch?;
            let permit = semaphore.clone().acquire_owned().await?;
            
            // Process batch on GPU
            let gpu_queue = self.gpu_queue.clone();
            tokio::spawn(async move {
                let _permit = permit; // Keep permit alive
                gpu_queue.lock().await.process_batch(batch, viewport).await;
            });
        }
        
        Ok(())
    }
}
```

## 6. Windows-Specific Robustness

### Path Handling

```rust
// src/platform/windows.rs
use std::os::windows::ffi::OsStrExt;
use winapi::um::fileapi::GetLongPathNameW;

pub fn normalize_windows_path(path: &Path) -> PathBuf {
    // Handle UNC and long paths
    let path_str = path.to_string_lossy();
    
    if path_str.starts_with(r"\\?\") {
        return path.to_owned();
    }
    
    if path_str.len() > 260 {
        // Convert to long path format
        return PathBuf::from(format!(r"\\?\{}", path.display()));
    }
    
    path.to_owned()
}

// Handle file locking
pub async fn try_open_exclusive(path: &Path) -> Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE};
    
    let path = normalize_windows_path(path);
    
    // Try multiple times with backoff
    let mut attempts = 0;
    loop {
        match OpenOptions::new()
            .read(true)
            .share_mode(0) // Exclusive access
            .open(&path)
        {
            Ok(file) => return Ok(file),
            Err(e) if attempts < 3 && e.kind() == io::ErrorKind::PermissionDenied => {
                attempts += 1;
                warn!("File locked, retrying in {}ms...", attempts * 100);
                tokio::time::sleep(Duration::from_millis(attempts * 100)).await;
            }
            Err(e) => {
                return Err(anyhow!("Cannot open file '{}': {} (file may be open in Excel)", 
                                   path.display(), e));
            }
        }
    }
}

// GPU driver detection
pub fn detect_gpu_info() -> Result<GpuInfo> {
    use wmi::{COMLibrary, WMIConnection};
    
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;
    
    #[derive(Deserialize)]
    struct Win32_VideoController {
        Name: String,
        DriverVersion: String,
        AdapterRAM: u64,
    }
    
    let results: Vec<Win32_VideoController> = wmi_con.query()?;
    
    // Find discrete GPU
    let gpu = results.into_iter()
        .filter(|g| g.AdapterRAM > 1024 * 1024 * 1024) // > 1GB
        .max_by_key(|g| g.AdapterRAM)
        .ok_or_else(|| anyhow!("No discrete GPU found"))?;
        
    Ok(GpuInfo {
        name: gpu.Name,
        driver_version: gpu.DriverVersion,
        vram_bytes: gpu.AdapterRAM,
    })
}
```

## 7. Real-World Data Handling

### Robust CSV Parser

```rust
// src/import/csv_parser.rs
use encoding_rs::Encoding;
use csv::ByteRecord;

pub struct SmartCsvReader {
    reader: csv::Reader<Box<dyn Read>>,
    encoding: &'static Encoding,
    line_count: u64,
}

impl SmartCsvReader {
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        
        // Detect encoding
        let mut buffer = vec![0; 8192];
        let bytes_read = file.read(&mut buffer)?;
        file.seek(SeekFrom::Start(0))?;
        
        let (encoding, _) = encoding_rs::Encoding::for_bom(&buffer[..bytes_read])
            .unwrap_or_else(|| {
                // Try to detect encoding
                if let Some(enc) = chardet::detect(&buffer[..bytes_read]) {
                    encoding_rs::Encoding::for_label(enc.encoding.as_bytes())
                        .unwrap_or(encoding_rs::UTF_8)
                } else {
                    encoding_rs::UTF_8
                }
            });
            
        // Handle BOM
        let reader: Box<dyn Read> = if encoding == encoding_rs::UTF_8 {
            // Strip BOM if present
            let mut reader = BufReader::new(file);
            let mut bom = [0; 3];
            reader.read_exact(&mut bom)?;
            if &bom != b"\xef\xbb\xbf" {
                reader.seek(SeekFrom::Start(0))?;
            }
            Box::new(reader)
        } else {
            // Transcode to UTF-8
            Box::new(encoding_rs_io::DecodeReaderBytes::new(file, encoding))
        };
        
        let csv_reader = csv::ReaderBuilder::new()
            .flexible(true) // Allow variable column counts
            .comment(Some(b'#'))
            .build(reader);
            
        Ok(Self {
            reader: csv_reader,
            encoding,
            line_count: 0,
        })
    }
    
    pub fn read_with_recovery(&mut self) -> impl Iterator<Item = Result<ByteRecord>> + '_ {
        std::iter::from_fn(move || {
            loop {
                self.line_count += 1;
                
                match self.reader.read_byte_record() {
                    Ok(record) => return Some(Ok(record)),
                    Err(e) => {
                        if let csv::ErrorKind::UnequalLengths { .. } = e.kind() {
                            warn!("Line {}: Inconsistent column count, skipping", self.line_count);
                            continue;
                        }
                        
                        if self.line_count > 1_000_000 {
                            error!("Too many errors, aborting at line {}", self.line_count);
                            return None;
                        }
                        
                        return Some(Err(e.into()));
                    }
                }
            }
        })
    }
}

// Type inference with confidence
pub struct TypeInferencer {
    samples: Vec<String>,
    null_values: HashSet<String>,
}

impl TypeInferencer {
    pub fn infer_with_confidence(&self) -> (DataType, f64) {
        let mut type_votes = HashMap::new();
        let total_samples = self.samples.len() as f64;
        
        for sample in &self.samples {
            if self.null_values.contains(sample) {
                continue;
            }
            
            let detected_type = self.detect_type(sample);
            *type_votes.entry(detected_type).or_insert(0) += 1;
        }
        
        let (best_type, votes) = type_votes.into_iter()
            .max_by_key(|(_, v)| *v)
            .unwrap_or((DataType::Utf8, 0));
            
        let confidence = votes as f64 / total_samples;
        
        (best_type, confidence)
    }
}
```

This comprehensive set of solutions addresses the practical challenges of building a production-ready tool. The focus is on robustness, user experience, and real-world data handling. Let me know if you need more detail on any specific area!