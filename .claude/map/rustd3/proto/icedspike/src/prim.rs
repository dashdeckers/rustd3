//! Compile check: a figure drawn by vello inside an
//! iced shader primitive, on iced's own wgpu device.
use iced_wgpu::graphics::Viewport;
use iced_wgpu::core::Rectangle;
use iced_wgpu::primitive::{Pipeline, Primitive};
use iced_wgpu::wgpu;
use std::sync::Arc;

#[derive(Debug)]
pub struct Figure {
    pub scene: Arc<SceneBox>,
}

pub struct SceneBox(pub vello::Scene);
impl std::fmt::Debug for SceneBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Scene")
    }
}

pub struct VelloPipeline {
    // vello::Renderer is Send but not Sync; iced pipelines must be Sync.
    renderer: std::sync::Mutex<vello::Renderer>,
    target: Option<(wgpu::Texture, wgpu::BindGroup, u32, u32)>,
    blit: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

const BLIT: &str = r#"
@group(0) @binding(0) var t: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;
struct V { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) i: u32) -> V {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return V(vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0), uv);
}
@fragment fn fs(v: V) -> @location(0) vec4<f32> { return textureSample(t, s, v.uv); }
"#;

impl Pipeline for VelloPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let renderer = vello::Renderer::new(device, vello::RendererOptions::default()).expect("vello");
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(BLIT.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let blit = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        VelloPipeline { renderer: std::sync::Mutex::new(renderer), target: None, blit, layout, sampler }
    }
}

impl Primitive for Figure {
    type Pipeline = VelloPipeline;

    fn prepare(
        &self,
        p: &mut VelloPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let s = viewport.scale_factor() as f32;
        let (w, h) = ((bounds.width * s) as u32, (bounds.height * s) as u32);
        if p.target.as_ref().is_none_or(|t| t.2 != w || t.3 != h) {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &p.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&p.sampler) },
                ],
            });
            p.target = Some((tex, bind, w, h));
        }
        let (tex, ..) = p.target.as_ref().unwrap();
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        // The scene is in logical px; render at device px.
        let mut scaled = vello::Scene::new();
        scaled.append(&self.scene.0, Some(vello::kurbo::Affine::scale(f64::from(s))));
        let params = vello::RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width: w,
            height: h,
            antialiasing_method: vello::AaConfig::Area,
        };
        p.renderer
            .get_mut()
            .expect("lock")
            .render_to_texture(device, queue, &scaled, &view, &params)
            .expect("render");
    }

    fn draw(&self, p: &VelloPipeline, pass: &mut wgpu::RenderPass<'_>) -> bool {
        let Some((_, bind, ..)) = &p.target else { return false };
        pass.set_pipeline(&p.blit);
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
        true
    }
}
