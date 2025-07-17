// Cargo.toml
/*
[package]
name = "wgpu-wasm-test"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
wgpu = { version = "26", features = ["webgl"] }
web-sys = "0.3"
console_error_panic_hook = "0.1"
bytemuck = "1.14"

[dependencies.web-sys]
version = "0.3"
features = ["console"]
*/

// src/lib.rs
use wasm_bindgen::prelude::*;
use wgpu;
use bytemuck;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct GpuContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

#[wasm_bindgen]
impl GpuContext {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<GpuContext, JsValue> {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();
        
        console_log!("Initializing wgpu instance...");
        
        // Create instance with WebGL backend
        // let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        //     backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
        //     ..Default::default()
        // });
        let instance = wgpu::Instance::default();

        console_log!("Requesting adapter...");
        
        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await;
        let adapter = match adapter {
            Ok(adapter) => adapter,
            Err(e) => return Err(JsValue::from_str("No suitable GPU adapter found")),
        };
        
        console_log!("Adapter found: {:?}", adapter.get_info());
        
        // Request device and queue
        console_log!("Requesting device...");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WASM Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                      .using_resolution(adapter.limits()),                    memory_hints: Default::default(),
                    trace: wgpu::Trace::default(),
                },
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create device: {:?}", e)))?;
        
        console_log!("Device created successfully!");
        
        Ok(GpuContext {
            instance,
            adapter,
            device,
            queue,
        })
    }
    
    #[wasm_bindgen]
    pub fn get_adapter_info(&self) -> String {
        let info = self.adapter.get_info();
        format!(
            "Adapter: {}\nVendor: {:?}\nDevice: {:?}\nDevice Type: {:?}\nDriver: {}\nDriver Info: {}\nBackend: {:?}",
            info.name,
            info.vendor,
            info.device,
            info.device_type,
            info.driver,
            info.driver_info,
            info.backend
        )
    }
    
    #[wasm_bindgen]
    pub fn get_device_features(&self) -> String {
        format!("{:?}", self.device.features())
    }
    
    #[wasm_bindgen]
    pub fn get_device_limits(&self) -> String {
        format!("{:#?}", self.device.limits())
    }
    
    // Simple compute shader test
    #[wasm_bindgen]
    pub async fn run_simple_compute(&self) -> Result<Vec<f32>, JsValue> {
        console_log!("Running simple compute shader...");
        
        // Create shader module
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        
        // Create buffer
        let size = 64;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Storage Buffer"),
            size: (size * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create staging buffer for reading results
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (size * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout and pipeline
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let compute_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        
        // Create command encoder and run compute pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });
        
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(size as u32 / 4, 1, 1);
        }
        
        // Copy buffer to staging buffer
        encoder.copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, (size * std::mem::size_of::<f32>()) as u64);
        
        // Submit commands
        self.queue.submit(Some(encoder.finish()));
        
        // Read results
        let buffer_slice = staging_buffer.slice(..);
        
        // We'll use futures::channel for the oneshot channel
        let (tx, rx) = futures::channel::oneshot::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        
        self.device.poll(wgpu::PollType::Wait);
        
        rx.await
            .expect("Failed to receive mapping result")
            .expect("Failed to map buffer");
        
        let data = buffer_slice.get_mapped_range();
        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        
        drop(data);
        staging_buffer.unmap();
        
        console_log!("Compute shader completed!");
        
        Ok(result)
    }
}

// src/shader.wgsl
/*
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    data[index] = f32(index) * 2.0;
}
*/