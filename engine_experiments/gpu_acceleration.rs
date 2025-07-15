use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use wgpu::*;
use bytemuck::{Pod, Zeroable};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use polars::prelude::*;

/// GPU-accelerated data processing engine
pub struct GpuAccelerationEngine {
    device: Device,
    queue: Queue,
    compute_pipelines: HashMap<String, ComputePipeline>,
    buffer_pool: BufferPool,
    shader_cache: ShaderCache,
    performance_monitor: PerformanceMonitor,
}

/// Buffer pool for efficient GPU memory management
pub struct BufferPool {
    available_buffers: Vec<Buffer>,
    used_buffers: Vec<Buffer>,
    buffer_size: u64,
}

/// Shader cache for compiled compute shaders
pub struct ShaderCache {
    shaders: HashMap<String, ShaderModule>,
    source_cache: HashMap<String, String>,
}

/// Performance monitoring for GPU operations
pub struct PerformanceMonitor {
    operation_times: HashMap<String, Vec<f64>>,
    memory_usage: HashMap<String, u64>,
    gpu_utilization: f64,
}

/// GPU compute operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuComputeConfig {
    pub workgroup_size: (u32, u32, u32),
    pub dispatch_size: (u32, u32, u32),
    pub buffer_size: u64,
    pub use_shared_memory: bool,
    pub precision: GpuPrecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuPrecision {
    Float16,
    Float32,
    Float64,
    Int32,
    Int64,
}

/// GPU-accelerated statistical operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuOperation {
    // Basic statistics
    Sum,
    Mean,
    Variance,
    StandardDeviation,
    Min,
    Max,
    Median,
    Quantiles(Vec<f64>),
    
    // Advanced statistics
    Correlation,
    Covariance,
    LinearRegression,
    PolynomialRegression(u32),
    
    // Data transformations
    Normalize,
    Standardize,
    Scale(f64, f64),
    Log,
    Exp,
    Power(f64),
    
    // Aggregations
    GroupBy(String),
    Pivot,
    Histogram(u32),
    
    // Machine learning
    KMeans(u32),
    PCA(u32),
    SVD,
    
    // Signal processing
    FFT,
    IFFT,
    Convolution,
    CrossCorrelation,
    
    // Image processing
    Blur,
    EdgeDetection,
    Resize,
    
    // Custom compute
    Custom(String),
}

/// GPU buffer wrapper with type information
#[derive(Debug)]
pub struct GpuBuffer {
    buffer: Buffer,
    size: u64,
    data_type: GpuDataType,
    usage: BufferUsages,
}

#[derive(Debug, Clone, Copy)]
pub enum GpuDataType {
    F32,
    F64,
    I32,
    I64,
    U32,
    U64,
}

/// Result of GPU computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuComputeResult {
    pub operation: GpuOperation,
    pub result_data: Vec<f64>,
    pub metadata: ComputeMetadata,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMetadata {
    pub input_size: usize,
    pub output_size: usize,
    pub workgroup_size: (u32, u32, u32),
    pub dispatch_count: u32,
    pub memory_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub compute_time_ms: f64,
    pub memory_transfer_time_ms: f64,
    pub total_time_ms: f64,
    pub throughput_gbs: f64,
    pub gpu_utilization: f64,
}

// GPU-compatible data structures
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuVector2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuVector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuMatrix4 {
    pub data: [f32; 16],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct StatisticsParams {
    pub count: u32,
    pub mean: f32,
    pub variance: f32,
    pub min_val: f32,
    pub max_val: f32,
    pub sum: f32,
    pub sum_squares: f32,
    pub _padding: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HistogramParams {
    pub bin_count: u32,
    pub min_value: f32,
    pub max_value: f32,
    pub bin_width: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct KMeansParams {
    pub k: u32,
    pub dimensions: u32,
    pub max_iterations: u32,
    pub tolerance: f32,
}

impl GpuAccelerationEngine {
    pub async fn new() -> Result<Self> {
        // Initialize WGPU
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("GPU Acceleration Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: MemoryHints::Performance,
                },
                None,
            )
            .await?;

        let buffer_pool = BufferPool {
            available_buffers: Vec::new(),
            used_buffers: Vec::new(),
            buffer_size: 1024 * 1024 * 64, // 64MB default
        };

        let shader_cache = ShaderCache {
            shaders: HashMap::new(),
            source_cache: HashMap::new(),
        };

        let performance_monitor = PerformanceMonitor {
            operation_times: HashMap::new(),
            memory_usage: HashMap::new(),
            gpu_utilization: 0.0,
        };

        let mut engine = Self {
            device,
            queue,
            compute_pipelines: HashMap::new(),
            buffer_pool,
            shader_cache,
            performance_monitor,
        };

        // Initialize standard compute shaders
        engine.initialize_standard_shaders().await?;

        Ok(engine)
    }

    /// Initialize standard compute shaders
    async fn initialize_standard_shaders(&mut self) -> Result<()> {
        // Basic statistics shader
        let stats_shader = self.create_statistics_shader();
        self.add_shader("statistics", &stats_shader).await?;

        // Histogram shader
        let histogram_shader = self.create_histogram_shader();
        self.add_shader("histogram", &histogram_shader).await?;

        // Matrix operations shader
        let matrix_shader = self.create_matrix_shader();
        self.add_shader("matrix", &matrix_shader).await?;

        // K-means clustering shader
        let kmeans_shader = self.create_kmeans_shader();
        self.add_shader("kmeans", &kmeans_shader).await?;

        // FFT shader
        let fft_shader = self.create_fft_shader();
        self.add_shader("fft", &fft_shader).await?;

        // Convolution shader
        let convolution_shader = self.create_convolution_shader();
        self.add_shader("convolution", &convolution_shader).await?;

        Ok(())
    }

    /// Add a compute shader to the cache
    async fn add_shader(&mut self, name: &str, source: &str) -> Result<()> {
        let shader_module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(name),
            source: ShaderSource::Wgsl(source.into()),
        });

        self.shader_cache.shaders.insert(name.to_string(), shader_module);
        self.shader_cache.source_cache.insert(name.to_string(), source.to_string());

        Ok(())
    }

    /// Perform GPU-accelerated computation on DataFrame
    pub async fn compute(&mut self, 
        data: &DataFrame, 
        operation: GpuOperation, 
        config: GpuComputeConfig
    ) -> Result<GpuComputeResult> {
        let start_time = std::time::Instant::now();

        // Convert DataFrame to GPU-compatible format
        let gpu_data = self.dataframe_to_gpu_data(data)?;

        // Create GPU buffers
        let input_buffer = self.create_buffer(&gpu_data, BufferUsages::STORAGE | BufferUsages::COPY_DST)?;
        let output_buffer = self.create_output_buffer(&operation, gpu_data.len(), &config)?;

        // Execute GPU computation
        let result = match operation {
            GpuOperation::Sum => self.compute_sum(&input_buffer, &output_buffer, &config).await?,
            GpuOperation::Mean => self.compute_mean(&input_buffer, &output_buffer, &config).await?,
            GpuOperation::Variance => self.compute_variance(&input_buffer, &output_buffer, &config).await?,
            GpuOperation::StandardDeviation => self.compute_std_dev(&input_buffer, &output_buffer, &config).await?,
            GpuOperation::Min => self.compute_min(&input_buffer, &output_buffer, &config).await?,
            GpuOperation::Max => self.compute_max(&input_buffer, &output_buffer, &config).await?,
            GpuOperation::Histogram(bins) => self.compute_histogram(&input_buffer, &output_buffer, bins, &config).await?,
            GpuOperation::Correlation => self.compute_correlation(&input_buffer, &output_buffer, &config).await?,
            GpuOperation::KMeans(k) => self.compute_kmeans(&input_buffer, &output_buffer, k, &config).await?,
            GpuOperation::FFT => self.compute_fft(&input_buffer, &output_buffer, &config).await?,
            GpuOperation::Normalize => self.compute_normalize(&input_buffer, &output_buffer, &config).await?,
            _ => return Err(anyhow::anyhow!("Operation not implemented")),
        };

        // Read result back from GPU
        let result_data = self.read_buffer(&output_buffer).await?;

        let total_time = start_time.elapsed();

        let performance = PerformanceMetrics {
            compute_time_ms: total_time.as_millis() as f64,
            memory_transfer_time_ms: 0.0, // TODO: Track separately
            total_time_ms: total_time.as_millis() as f64,
            throughput_gbs: (gpu_data.len() * 4) as f64 / total_time.as_secs_f64() / 1e9,
            gpu_utilization: self.performance_monitor.gpu_utilization,
        };

        let metadata = ComputeMetadata {
            input_size: gpu_data.len(),
            output_size: result_data.len(),
            workgroup_size: config.workgroup_size,
            dispatch_count: 1,
            memory_used: input_buffer.size() + output_buffer.size(),
        };

        Ok(GpuComputeResult {
            operation,
            result_data,
            metadata,
            performance,
        })
    }

    /// Convert DataFrame to GPU-compatible data
    fn dataframe_to_gpu_data(&self, df: &DataFrame) -> Result<Vec<f32>> {
        let mut data = Vec::new();
        
        for column in df.get_columns() {
            match column.dtype() {
                DataType::Float32 => {
                    let values = column.f32()?;
                    for value in values {
                        data.push(value.unwrap_or(0.0));
                    }
                },
                DataType::Float64 => {
                    let values = column.f64()?;
                    for value in values {
                        data.push(value.unwrap_or(0.0) as f32);
                    }
                },
                DataType::Int32 => {
                    let values = column.i32()?;
                    for value in values {
                        data.push(value.unwrap_or(0) as f32);
                    }
                },
                DataType::Int64 => {
                    let values = column.i64()?;
                    for value in values {
                        data.push(value.unwrap_or(0) as f32);
                    }
                },
                _ => return Err(anyhow::anyhow!("Unsupported data type for GPU computation")),
            }
        }

        Ok(data)
    }

    /// Create a GPU buffer from data
    fn create_buffer(&self, data: &[f32], usage: BufferUsages) -> Result<Buffer> {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(data),
            usage,
        });

        Ok(buffer)
    }

    /// Create output buffer based on operation
    fn create_output_buffer(&self, operation: &GpuOperation, input_size: usize, config: &GpuComputeConfig) -> Result<Buffer> {
        let output_size = match operation {
            GpuOperation::Sum | GpuOperation::Mean | GpuOperation::Variance | 
            GpuOperation::StandardDeviation | GpuOperation::Min | GpuOperation::Max => 4,
            GpuOperation::Histogram(bins) => *bins as u64 * 4,
            GpuOperation::Correlation => input_size as u64 * input_size as u64 * 4,
            GpuOperation::KMeans(k) => *k as u64 * 4,
            GpuOperation::FFT => input_size as u64 * 8, // Complex numbers
            GpuOperation::Normalize => input_size as u64 * 4,
            _ => input_size as u64 * 4,
        };

        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Ok(buffer)
    }

    /// Read data back from GPU buffer
    async fn read_buffer(&self, buffer: &Buffer) -> Result<Vec<f64>> {
        let buffer_slice = buffer.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();

        buffer_slice.map_async(MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        self.device.poll(Maintain::Wait);
        receiver.await??;

        let data = buffer_slice.get_mapped_range();
        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        buffer.unmap();

        Ok(result.into_iter().map(|x| x as f64).collect())
    }

    /// Compute sum using GPU
    async fn compute_sum(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        let shader = self.shader_cache.shaders.get("statistics")
            .ok_or_else(|| anyhow::anyhow!("Statistics shader not found"))?;

        let bind_group_layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Sum Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Sum Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: input.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: output.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Sum Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = self.device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Sum Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: "sum_main",
            compilation_options: Default::default(),
            cache: None,
        });

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Sum Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Sum Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(
                config.dispatch_size.0,
                config.dispatch_size.1,
                config.dispatch_size.2,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    /// Compute mean using GPU
    async fn compute_mean(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        // Similar to sum but with division by count
        self.compute_sum(input, output, config).await?;
        // TODO: Add division step
        Ok(())
    }

    /// Compute variance using GPU
    async fn compute_variance(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        // Two-pass algorithm: first compute mean, then variance
        // TODO: Implement two-pass variance computation
        Ok(())
    }

    /// Compute standard deviation using GPU
    async fn compute_std_dev(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        // Compute variance then take square root
        self.compute_variance(input, output, config).await?;
        // TODO: Add square root step
        Ok(())
    }

    /// Compute minimum using GPU
    async fn compute_min(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        // Parallel reduction to find minimum
        // TODO: Implement parallel min reduction
        Ok(())
    }

    /// Compute maximum using GPU
    async fn compute_max(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        // Parallel reduction to find maximum
        // TODO: Implement parallel max reduction
        Ok(())
    }

    /// Compute histogram using GPU
    async fn compute_histogram(&self, input: &Buffer, output: &Buffer, bins: u32, config: &GpuComputeConfig) -> Result<()> {
        // GPU-accelerated histogram computation
        // TODO: Implement GPU histogram
        Ok(())
    }

    /// Compute correlation using GPU
    async fn compute_correlation(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        // GPU-accelerated correlation matrix
        // TODO: Implement GPU correlation
        Ok(())
    }

    /// Compute K-means clustering using GPU
    async fn compute_kmeans(&self, input: &Buffer, output: &Buffer, k: u32, config: &GpuComputeConfig) -> Result<()> {
        // GPU-accelerated K-means clustering
        // TODO: Implement GPU K-means
        Ok(())
    }

    /// Compute FFT using GPU
    async fn compute_fft(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        // GPU-accelerated Fast Fourier Transform
        // TODO: Implement GPU FFT
        Ok(())
    }

    /// Compute normalization using GPU
    async fn compute_normalize(&self, input: &Buffer, output: &Buffer, config: &GpuComputeConfig) -> Result<()> {
        // GPU-accelerated data normalization
        // TODO: Implement GPU normalization
        Ok(())
    }

    /// Create statistics compute shader
    fn create_statistics_shader(&self) -> String {
        r#"
        @group(0) @binding(0) var<storage, read> input_data: array<f32>;
        @group(0) @binding(1) var<storage, read_write> output_data: array<f32>;

        @compute @workgroup_size(256)
        fn sum_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let index = global_id.x;
            if (index >= arrayLength(&input_data)) {
                return;
            }
            
            // Parallel sum reduction
            var sum = 0.0;
            for (var i = 0u; i < arrayLength(&input_data); i++) {
                sum += input_data[i];
            }
            
            if (index == 0u) {
                output_data[0] = sum;
            }
        }

        @compute @workgroup_size(256)
        fn mean_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let index = global_id.x;
            if (index >= arrayLength(&input_data)) {
                return;
            }
            
            var sum = 0.0;
            let count = f32(arrayLength(&input_data));
            
            for (var i = 0u; i < arrayLength(&input_data); i++) {
                sum += input_data[i];
            }
            
            if (index == 0u) {
                output_data[0] = sum / count;
            }
        }

        @compute @workgroup_size(256)
        fn variance_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let index = global_id.x;
            if (index >= arrayLength(&input_data)) {
                return;
            }
            
            // Two-pass variance computation
            var sum = 0.0;
            let count = f32(arrayLength(&input_data));
            
            // First pass: compute mean
            for (var i = 0u; i < arrayLength(&input_data); i++) {
                sum += input_data[i];
            }
            let mean = sum / count;
            
            // Second pass: compute variance
            var variance = 0.0;
            for (var i = 0u; i < arrayLength(&input_data); i++) {
                let diff = input_data[i] - mean;
                variance += diff * diff;
            }
            
            if (index == 0u) {
                output_data[0] = variance / count;
            }
        }
        "#.to_string()
    }

    /// Create histogram compute shader
    fn create_histogram_shader(&self) -> String {
        r#"
        @group(0) @binding(0) var<storage, read> input_data: array<f32>;
        @group(0) @binding(1) var<storage, read_write> histogram: array<u32>;
        @group(0) @binding(2) var<uniform> params: HistogramParams;

        struct HistogramParams {
            bin_count: u32,
            min_value: f32,
            max_value: f32,
            bin_width: f32,
        }

        @compute @workgroup_size(256)
        fn histogram_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let index = global_id.x;
            if (index >= arrayLength(&input_data)) {
                return;
            }
            
            let value = input_data[index];
            if (value >= params.min_value && value <= params.max_value) {
                let bin_index = u32((value - params.min_value) / params.bin_width);
                let clamped_bin = min(bin_index, params.bin_count - 1u);
                atomicAdd(&histogram[clamped_bin], 1u);
            }
        }
        "#.to_string()
    }

    /// Create matrix operations compute shader
    fn create_matrix_shader(&self) -> String {
        r#"
        @group(0) @binding(0) var<storage, read> matrix_a: array<f32>;
        @group(0) @binding(1) var<storage, read> matrix_b: array<f32>;
        @group(0) @binding(2) var<storage, read_write> matrix_c: array<f32>;
        @group(0) @binding(3) var<uniform> dimensions: vec3<u32>;

        @compute @workgroup_size(16, 16)
        fn matrix_multiply(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let row = global_id.x;
            let col = global_id.y;
            
            if (row >= dimensions.x || col >= dimensions.y) {
                return;
            }
            
            var sum = 0.0;
            for (var k = 0u; k < dimensions.z; k++) {
                sum += matrix_a[row * dimensions.z + k] * matrix_b[k * dimensions.y + col];
            }
            
            matrix_c[row * dimensions.y + col] = sum;
        }
        "#.to_string()
    }

    /// Create K-means compute shader
    fn create_kmeans_shader(&self) -> String {
        r#"
        @group(0) @binding(0) var<storage, read> data_points: array<f32>;
        @group(0) @binding(1) var<storage, read_write> centroids: array<f32>;
        @group(0) @binding(2) var<storage, read_write> assignments: array<u32>;
        @group(0) @binding(3) var<uniform> params: KMeansParams;

        struct KMeansParams {
            k: u32,
            dimensions: u32,
            max_iterations: u32,
            tolerance: f32,
        }

        @compute @workgroup_size(256)
        fn kmeans_assign(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let point_index = global_id.x;
            let total_points = arrayLength(&data_points) / params.dimensions;
            
            if (point_index >= total_points) {
                return;
            }
            
            var min_distance = 1e30;
            var best_cluster = 0u;
            
            for (var cluster = 0u; cluster < params.k; cluster++) {
                var distance = 0.0;
                for (var dim = 0u; dim < params.dimensions; dim++) {
                    let point_val = data_points[point_index * params.dimensions + dim];
                    let centroid_val = centroids[cluster * params.dimensions + dim];
                    let diff = point_val - centroid_val;
                    distance += diff * diff;
                }
                
                if (distance < min_distance) {
                    min_distance = distance;
                    best_cluster = cluster;
                }
            }
            
            assignments[point_index] = best_cluster;
        }
        "#.to_string()
    }

    /// Create FFT compute shader
    fn create_fft_shader(&self) -> String {
        r#"
        @group(0) @binding(0) var<storage, read> input_real: array<f32>;
        @group(0) @binding(1) var<storage, read> input_imag: array<f32>;
        @group(0) @binding(2) var<storage, read_write> output_real: array<f32>;
        @group(0) @binding(3) var<storage, read_write> output_imag: array<f32>;

        const PI = 3.14159265359;

        @compute @workgroup_size(256)
        fn fft_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let index = global_id.x;
            let n = arrayLength(&input_real);
            
            if (index >= n) {
                return;
            }
            
            var sum_real = 0.0;
            var sum_imag = 0.0;
            
            for (var k = 0u; k < n; k++) {
                let angle = -2.0 * PI * f32(index * k) / f32(n);
                let cos_val = cos(angle);
                let sin_val = sin(angle);
                
                sum_real += input_real[k] * cos_val - input_imag[k] * sin_val;
                sum_imag += input_real[k] * sin_val + input_imag[k] * cos_val;
            }
            
            output_real[index] = sum_real;
            output_imag[index] = sum_imag;
        }
        "#.to_string()
    }

    /// Create convolution compute shader
    fn create_convolution_shader(&self) -> String {
        r#"
        @group(0) @binding(0) var<storage, read> signal: array<f32>;
        @group(0) @binding(1) var<storage, read> kernel: array<f32>;
        @group(0) @binding(2) var<storage, read_write> output: array<f32>;
        @group(0) @binding(3) var<uniform> params: ConvolutionParams;

        struct ConvolutionParams {
            signal_length: u32,
            kernel_length: u32,
            output_length: u32,
            padding: u32,
        }

        @compute @workgroup_size(256)
        fn convolution_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let output_index = global_id.x;
            
            if (output_index >= params.output_length) {
                return;
            }
            
            var sum = 0.0;
            for (var k = 0u; k < params.kernel_length; k++) {
                let signal_index = i32(output_index) - i32(k);
                if (signal_index >= 0 && signal_index < i32(params.signal_length)) {
                    sum += signal[signal_index] * kernel[k];
                }
            }
            
            output[output_index] = sum;
        }
        "#.to_string()
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> &PerformanceMonitor {
        &self.performance_monitor
    }

    /// Clear performance statistics
    pub fn clear_performance_stats(&mut self) {
        self.performance_monitor.operation_times.clear();
        self.performance_monitor.memory_usage.clear();
        self.performance_monitor.gpu_utilization = 0.0;
    }
}

impl Default for GpuAccelerationEngine {
    fn default() -> Self {
        block_on(Self::new()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[tokio::test]
    async fn test_gpu_engine_creation() {
        let engine = GpuAccelerationEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_gpu_sum_computation() {
        let mut engine = GpuAccelerationEngine::new().await.unwrap();
        
        let df = df! {
            "values" => [1.0, 2.0, 3.0, 4.0, 5.0],
        }.unwrap();

        let config = GpuComputeConfig {
            workgroup_size: (256, 1, 1),
            dispatch_size: (1, 1, 1),
            buffer_size: 1024,
            use_shared_memory: false,
            precision: GpuPrecision::Float32,
        };

        let result = engine.compute(&df, GpuOperation::Sum, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dataframe_conversion() {
        let engine = GpuAccelerationEngine::new().await.unwrap();
        
        let df = df! {
            "a" => [1.0, 2.0, 3.0],
            "b" => [4.0, 5.0, 6.0],
        }.unwrap();

        let gpu_data = engine.dataframe_to_gpu_data(&df);
        assert!(gpu_data.is_ok());
        
        let data = gpu_data.unwrap();
        assert_eq!(data.len(), 6);
        assert_eq!(data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[tokio::test]
    async fn test_buffer_operations() {
        let engine = GpuAccelerationEngine::new().await.unwrap();
        
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let buffer = engine.create_buffer(&data, BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        assert!(buffer.is_ok());
        
        let buffer = buffer.unwrap();
        assert_eq!(buffer.size(), 20); // 5 * 4 bytes
    }
} 