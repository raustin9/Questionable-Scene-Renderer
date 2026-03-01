use std::{any::{Any, TypeId}, borrow::Cow, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, num::NonZero};

use wgpu::util::DeviceExt;

use crate::{Texture, shader::UniformBuffer, geometry::Mesh};

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct ResourceId(u64);

impl ResourceId {
    pub fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);

        ResourceId(
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    }
}

pub enum ResourceKind {
    Texture,
    UniformBuffer,
    Mesh,
}

pub enum ResourceAccess {
    Read,
    ReadWrite,
    Write
}

pub enum ResourceData {
    Texture(Texture),
    UniformBuffer(UniformBuffer),
    Mesh(Mesh),
}

pub struct ResourceHandle<'a> {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub resource: &'a ResourceData,
}

pub struct BufferResource {
    pub buffer: wgpu::Buffer,
    pub usages: wgpu::BufferUsages,
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct BufferHandle(u64);

impl BufferHandle {
    pub fn new(hash: u64) -> Self {
        Self(hash)
    }
}

pub struct BufferRegistry {
    buffers: HashMap<BufferHandle, BufferResource>,
    hasher: DefaultHasher,
}

impl BufferRegistry {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            hasher: DefaultHasher::new()
        }
    }

    /// Create a buffer and return its handle
    pub fn create_buffer(&mut self, device: &wgpu::Device, usages: wgpu::BufferUsages, data: &[u8]) -> BufferHandle {
        data.hash(&mut self.hasher);
        let handle = BufferHandle::new(self.hasher.finish());

        if self.buffers.contains_key(&handle) {
            return handle;
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("buffer"),
            contents: data,
            usage: usages
        });

        let resource = BufferResource {
            buffer,
            usages
        };

        self.buffers.insert(handle, resource);
        handle
    }

    /// Write data to a buffer
    pub fn write_buffer(&self, handle: BufferHandle, queue: &wgpu::Queue, offset: u64, data: &[u8]) {
        let resource = self.buffers.get(&handle)
            .expect("Attempted to retrieve buffer with invalid handle");
        queue.write_buffer(&resource.buffer, offset, data);
    }

    /// Attempt to get a held buffer with a handle.
    /// Returns `None` if no resource is found.
    pub fn get_buffer(&self, handle: BufferHandle) -> Option<&wgpu::Buffer> {
        self.buffers.get(&handle).map(|b| &b.buffer)
    }
    
    /// Attempt to get a held buffer's usages with a handle.
    /// Returns `None` if no resource is found.
    pub fn get_usages(&self, handle: BufferHandle) -> Option<&wgpu::BufferUsages> {
        self.buffers.get(&handle).map(|b| &b.usages)
    }
}

pub enum TextureSize {
    /// Set the texture to a fixed size that will not be updated 
    /// corresponding to screen resizes
    Fixed(u32, u32),

    /// Set the texture size to fill the screen.
    /// When the `TextureRegistry` is resized to 
    /// a new size, textures with this size will
    /// resize accordingly.
    Full,
}

impl TextureSize {
    /// Return whether or not a texture's size is relative to the screen.
    /// `Fixed` textures do not.
    pub fn screen_relative(&self) -> bool {
        match self {
            Self::Fixed(_, _) => false,
            Self::Full => true,
        }
    }
}

pub enum SamplerRepeat {
    Repeat,
    Clamp,
    MirrorRepeat,
    Border(wgpu::Color)
}

impl From<SamplerRepeat> for wgpu::AddressMode {
    fn from(value: SamplerRepeat) -> Self {
        match value {
            SamplerRepeat::Clamp => wgpu::AddressMode::ClampToEdge,
            SamplerRepeat::Repeat => wgpu::AddressMode::Repeat,
            SamplerRepeat::Border(_color) => wgpu::AddressMode::ClampToBorder,
            SamplerRepeat::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        }
    }
}

pub struct SamplerDescriptor {
    pub address_mode: SamplerRepeat,
}

pub struct TextureDescriptor {
    pub label: String,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub size: TextureSize,
}

/// Resource for a texture to be used in the renderer
pub struct TextureResource {
    /// The wgpu texture
    pub texture: wgpu::Texture,

    /// The wpgu texture view
    pub view: wgpu::TextureView,

    pub sampler: Option<wgpu::Sampler>,

    /// Description of the texture used to create it.
    /// Can be used to find other relevant information 
    /// about the texture like size, format, etc.
    pub descriptor: TextureDescriptor
}

/// Handle to a `TextureResource` held in the `TextureRegistry`
pub type TextureHandle = ResourceId;

/// Registry and manager of texture resources.
/// Resources can be retrieved with a `TextureHandle`
pub struct TextureRegistry {
    /// Storage for the textures
    textures: HashMap<TextureHandle, TextureResource>,

    /// The current width to keep the textures
    current_width: u32,

    /// The current height to keep the textures
    current_height: u32,
}

impl TextureRegistry {
    pub fn new(current_width: u32, current_height: u32) -> Self {
        Self {
            textures: HashMap::new(),
            current_width,
            current_height,
        }
    }

    /// Create and store a texture resource.
    /// Returns a handle to the resource to be retrieved
    pub fn create_texture(&mut self, device: &wgpu::Device, descriptor: TextureDescriptor, sampler_options: Option<SamplerDescriptor>) -> TextureHandle {
        let handle = TextureHandle::new();

        let (width, height) = self.resolve_size(&descriptor.size);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&descriptor.label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1
            },
            usage: descriptor.usage,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
            format: descriptor.format
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });

        let sampler = match sampler_options {
            None => None,
            Some(options) => {
                let address_mode = options.address_mode.into();
                
                Some(device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("sampler"),
                    address_mode_u: address_mode,
                    address_mode_v: address_mode,
                    address_mode_w: address_mode,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                    ..Default::default()
                }))
            }
        };

        let resource = TextureResource {
            texture,
            view,
            sampler,
            descriptor
        };

        self.textures.insert(handle, resource);

        handle
    }

    /// The `wgpu::TexureView` for the texture resource corresponding to the input handle
    pub fn get_view(&self, handle: TextureHandle) -> Option<&wgpu::TextureView> {
        self.textures.get(&handle).map(|t| &t.view)
    }

    /// The `wgpu::Texure` for the texture resource corresponding to the input handle
    pub fn get_texture(&self, handle: TextureHandle) -> Option<&wgpu::Texture> {
        self.textures.get(&handle).map(|t| &t.texture)
    }

    pub fn get_sampler(&self, handle: TextureHandle) -> Option<&wgpu::Sampler> {
        match self.textures.get(&handle) {
            Some(texture) => {
                match &texture.sampler {
                    Some(sampler) => Some(sampler),
                    None => None,
                }
            },
            None => None
        }
    }

    /// Get the resolved width and height from a `TextureSize` enum value
    pub fn resolve_size(&self, size: &TextureSize) -> (u32, u32) {
        match size {
            TextureSize::Fixed(width, height) => (*width, *height),
            TextureSize::Full => (self.current_width, self.current_height)
        }
    }

    /// Set the texture registry to a new size (normally to adjust to updated screen size)
    /// and update all the screen-relative textures to fit this new size
    pub fn resize_textures(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.current_width = width;
        self.current_height = height;

        let to_update = self.textures
            .iter()
            .filter(|(_, resource)| resource.descriptor.size.screen_relative())
            .map(|(handle, resource)| {
                let (w, h) = self.resolve_size(&resource.descriptor.size);
                (*handle, w, h)
            })
        .collect::<Vec<_>>();

        for (handle, width, height) in to_update {
            if let Some(resource) = self.textures.get_mut(&handle) {
                let updated_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&resource.descriptor.label),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    format: resource.descriptor.format,
                    usage: resource.descriptor.usage,
                    dimension: wgpu::TextureDimension::D2,
                    view_formats: &[],
                });

                let updated_view = updated_texture.create_view(&wgpu::TextureViewDescriptor::default());

                resource.texture = updated_texture;
                resource.view = updated_view;
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct PipelineHandle(u64);

impl PipelineHandle {
    pub fn new(hash: u64) -> Self {
        Self(hash)
    }
}

pub struct PipelineResource {
    pipeline: wgpu::RenderPipeline,
    color_targets: Vec<wgpu::ColorTargetState>,
    depth_target: Option<wgpu::DepthStencilState>,
}

/// Information used to request a pipeline
pub struct PipelineRequestInfo<'a> {
    pub color_targets: &'a [wgpu::ColorTargetState],
    pub depth_target: Option<wgpu::DepthStencilState>,
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub vertex_layouts: &'a [wgpu::VertexBufferLayout<'a>],
    pub vertex_module: &'a wgpu::ShaderModule,
    pub fragment_module: Option<&'a wgpu::ShaderModule>,
    pub vertex_entry: &'a str,
    pub fragment_entry: Option<&'a str>,
    pub multisample: &'a wgpu::MultisampleState,
    pub topology: wgpu::PrimitiveState,
}

impl<'a> PipelineRequestInfo<'a> {
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl<'a> Hash for PipelineRequestInfo<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for color_target in self.color_targets {
            color_target.hash(state);
        }

        for vertex_buffer_layout in self.vertex_layouts {
            vertex_buffer_layout.hash(state);
        }

        self.depth_target.hash(state);

        for layout in self.bind_group_layouts {
            layout.hash(state);
        }

        self.vertex_module.hash(state);

        self.fragment_module.hash(state);

        self.vertex_entry.hash(state);
        self.fragment_entry.hash(state);

        self.multisample.hash(state);
        self.topology.hash(state);
    }
}

pub struct PipelineManager {
    pipelines: HashMap<PipelineHandle, PipelineResource>,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn request_pipeline(
        &mut self, 
        device: &wgpu::Device,
        request_info: &PipelineRequestInfo
    ) -> PipelineHandle {
        let handle = PipelineHandle(request_info.get_hash());
        if self.pipelines.contains_key(&handle) {
            // println!("Found existing pipeline.");

            return handle;
        }
        println!("No matching pipeline found. Creating new one");

        let mut pipeline_builder = PipelineBuilder::new(request_info.vertex_module, request_info.fragment_module);
        match request_info.fragment_entry {
            Some(entry) => { pipeline_builder.frag_entry(entry); },
            None => {}
        };
        pipeline_builder
            .vert_entry(request_info.vertex_entry)
            .set_vertex_layouts(request_info.vertex_layouts);

        for target in request_info.color_targets {
            pipeline_builder.add_color_target(target.clone());
        }

        match &request_info.depth_target {
            Some(target) => { pipeline_builder.depth_stencil(target.clone()); },
            None => {}
        };

        pipeline_builder
            .topology(request_info.topology)
            .multisample(request_info.multisample.clone())
            .set_bind_group_layouts(request_info.bind_group_layouts);

        let pipeline = pipeline_builder.build(device);
        let resource = PipelineResource {
            pipeline,
            color_targets: 
                request_info.color_targets
                    .iter()
                    .map(|target| target.clone())
                    .collect::<Vec<_>>(),
            depth_target: request_info.depth_target.clone(),
        };

        self.pipelines.insert(handle, resource);

        handle
    }

    /// Get a pipeline from a handle
    pub fn get_pipeline(&self, handle: PipelineHandle) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(&handle).map(|p| &p.pipeline)
    }
}

pub struct PipelineBuilder<'a> {
    // bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    label: Option<&'a str>,
    vert_module: &'a wgpu::ShaderModule,
    frag_module: Option<&'a wgpu::ShaderModule>,
    topology: wgpu::PrimitiveState,
    vert_entry: &'a str,
    frag_entry: Option<&'a str>,

    vertex_buffers: &'a [wgpu::VertexBufferLayout<'a>],
    color_targets: Vec<Option<wgpu::ColorTargetState>>,
    depth_stencil_state: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    multiview_mask: Option<NonZero<u32>>,
    cache: Option<&'a wgpu::PipelineCache>,

    bind_group_layouts: &'a[&'a wgpu::BindGroupLayout]
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(vert_module: &'a wgpu::ShaderModule, frag_module: Option<&'a wgpu::ShaderModule>) -> Self {
        let topology = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };

        let multisample = wgpu::MultisampleState {
            count: 1,
            mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
            alpha_to_coverage_enabled: false
        };

        Self {
            label: None,
            vert_module,
            frag_module,
            topology,
            frag_entry: match frag_module {
                Some(_) => Some("main"),
                None => None,
            },
            vert_entry: "main",
            vertex_buffers: &[],
            color_targets: Vec::new(),
            depth_stencil_state: None,
            cache: None,
            multiview_mask: None,
            bind_group_layouts: &[],
            multisample,
        }
    }

    pub fn build(&self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_builder_layout"),
            bind_group_layouts: self.bind_group_layouts,
            immediate_size: 0
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: self.vert_module,
                entry_point: Some(self.vert_entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: self.vertex_buffers
            },
            fragment: match self.frag_module {
                None => None,
                Some(module) => {
                    Some(wgpu::FragmentState {
                        module: module,
                        entry_point: self.frag_entry,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: self.color_targets.as_slice()
                    })
                }
            },
            depth_stencil: self.depth_stencil_state.clone(),
            primitive: self.topology,
            multisample: self.multisample,
            multiview_mask: self.multiview_mask,
            cache: self.cache,
        })
    }

    pub fn vert_module(&mut self, module: &'a wgpu::ShaderModule) -> &mut Self {
        self.vert_module = module;

        self
    }

    pub fn frag_module(&mut self, module: &'a wgpu::ShaderModule) -> &mut Self {
        self.frag_module = Some(module);

        self
    }

    pub fn vert_entry(&mut self, entry: &'a str) -> &mut Self {
        self.vert_entry = entry;

        self
    }

    pub fn frag_entry(&mut self, entry: &'a str) -> &mut Self {
        self.frag_entry = Some(entry);

        self
    }

    pub fn label(&mut self, label: &'a str) -> &mut Self {
        self.label = Some(label);

        self
    }

    pub fn add_color_target(&mut self, target: wgpu::ColorTargetState) -> &mut Self {
        self.color_targets.push(Some(target));

        self
    }

    pub fn depth_stencil(&mut self, state: wgpu::DepthStencilState) -> &mut Self {
        self.depth_stencil_state = Some(state);

        self
    }

    pub fn set_bind_group_layouts(&mut self, layouts: &'a [&'a wgpu::BindGroupLayout]) -> &mut Self {
        self.bind_group_layouts = layouts;

        self
    }
    
    pub fn topology(&mut self, topology: wgpu::PrimitiveState) -> &mut Self {
        self.topology = topology;

        self
    }

    pub fn multisample(&mut self, multisample: wgpu::MultisampleState) -> &mut Self {
        self.multisample = multisample;

        self
    }

    pub fn set_vertex_layouts(&mut self, layouts: &'a [wgpu::VertexBufferLayout<'a>]) -> &mut Self {
        self.vertex_buffers = layouts;
        self
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct ShaderFeatureId(u64);

pub trait ShaderFeature: Send + Sync + 'static {
    /// The layout descriptor for this feature
    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static>;
}

pub struct CameraInfoFeature;
impl ShaderFeature for CameraInfoFeature {
    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_info_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        }
    }
}

pub struct TransformFeature;
impl ShaderFeature for TransformFeature {
    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("transform_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        }
    }
}

pub struct DiffuseTextureFeature;
impl ShaderFeature for DiffuseTextureFeature {
    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("diffuse_texture_layout"),
            entries: &[
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                },
            ]
        }
    }
}

pub struct DiffuseColorFeature;
impl ShaderFeature for DiffuseColorFeature {
    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("diffuse_color_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        }
    }
}

pub struct ShaderFeatureEntry {
    pub id: ShaderFeatureId,
    pub layout: wgpu::BindGroupLayout,
}

pub struct ShaderFeatureRegistry {
    id_types: HashMap<TypeId, ShaderFeatureId>,
    entries: Vec<ShaderFeatureEntry>,
}

impl ShaderFeatureRegistry {
    pub fn new() -> Self {
        Self {
            id_types: HashMap::new(),
            entries: Vec::new(),
        }
    }

    pub fn register<F: ShaderFeature>(&mut self, device: &wgpu::Device) -> ShaderFeatureId {
        let type_id = TypeId::of::<F>();
        if let Some(&id) = self.id_types.get(&type_id) {
            return id;
        }

        let id = ShaderFeatureId(self.entries.len() as u64);
        let layout = device.create_bind_group_layout(&F::layout_descriptor());

        self.id_types.insert(type_id, id);
        self.entries.push(ShaderFeatureEntry {
            id,
            layout,
        });

        id
    }

    pub fn get_entry_from_id(&self, id: ShaderFeatureId) -> &ShaderFeatureEntry {
        &self.entries[id.0 as usize]
    }

    pub fn feature_id<F: ShaderFeature>(&self) -> Option<ShaderFeatureId> {
        self.id_types.get(&TypeId::of::<F>()).copied()
    }

    pub fn get_entry<F: ShaderFeature>(&self) -> Option<&ShaderFeatureEntry> {
        match self.feature_id::<F>() {
            Some(id) => Some(&self.entries[id.0 as usize]),
            None => None
        }
    }
}

pub struct ShaderFeatureSet<'a> {
    feature_registry: &'a ShaderFeatureRegistry,
    feature_ids: Vec<ShaderFeatureId>
}

impl<'a> ShaderFeatureSet<'a> {
    pub fn new(feature_registry: &'a ShaderFeatureRegistry) -> Self {
        Self {
            feature_registry,
            feature_ids: Vec::new()
        }
    }

    pub fn include<F: ShaderFeature>(&mut self) -> &mut Self {
        let id = self.feature_registry.feature_id::<F>()
            .expect("Attempting to find id for feature that is not registered");
        self.feature_ids.push(id);
        self
    }

    pub fn finish(&self) -> Vec<ShaderFeatureId> {
        self.feature_ids.clone()
    }
}

impl<'a> Hash for ShaderFeatureSet<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.feature_ids.hash(state);
    }
}

#[derive(Eq, PartialEq, Hash)]
pub enum ShaderKey {
    Material {
        features: Vec<ShaderFeatureId>,
        vertex_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    },
    Global {
        features: Vec<ShaderFeatureId>
    }
}

pub struct ShaderResource {
    pub name: &'static str,
    pub vert_module: wgpu::ShaderModule,
    pub frag_module: wgpu::ShaderModule,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

pub struct ShaderRegistry {
    features: ShaderFeatureRegistry,
    shaders: HashMap<ShaderKey, ShaderResource>
}

impl ShaderRegistry {
    pub fn new(features: ShaderFeatureRegistry) -> Self {
        Self {
            features,
            shaders: HashMap::new()
        }
    }

    pub fn get_feature<F: ShaderFeature>(&self) -> Option<&ShaderFeatureEntry> {
        self.features.get_entry::<F>()
    }

    pub fn get_feature_id<F: ShaderFeature>(&self) -> Option<ShaderFeatureId> {
        self.features.feature_id::<F>()
    }

    pub fn add_global(
        &mut self,
        device: &wgpu::Device,
        name: &'static str,
        source: wgpu::ShaderSource,
        features: Vec<ShaderFeatureId>,
    ) {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source,
        });
        
        let bind_group_layouts = features
            .iter()
            .map(|id| self.features.get_entry_from_id(*id).layout.clone())
            .collect::<Vec<_>>();

        let key = ShaderKey::Global { features };
        self.shaders.insert(key, ShaderResource { 
            name, 
            vert_module: module.clone(), 
            frag_module: module, 
            bind_group_layouts 
        });
    }

    pub fn add_material(
        &mut self,
        device: &wgpu::Device,
        name: &'static str,
        source: wgpu::ShaderSource,
        features: Vec<ShaderFeatureId>,
        vertex_layouts: &[wgpu::VertexBufferLayout<'static>]
    ) {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source,
        });
        
        let bind_group_layouts = features
            .iter()
            .map(|id| self.features.get_entry_from_id(*id).layout.clone())
            .collect::<Vec<_>>();

        let key = ShaderKey::Material { 
            features, 
            vertex_layouts: vertex_layouts.iter().map(|vl| vl.clone()).collect::<Vec<_>>()
        };

        self.shaders.insert(key, ShaderResource { 
            name, 
            vert_module: module.clone(), 
            frag_module: module, 
            bind_group_layouts 
        });
    }

    pub fn get_material(
        &self, 
        features: &[ShaderFeatureId],
        vertex_layouts: &[wgpu::VertexBufferLayout<'static>]
    ) -> Option<&ShaderResource> {
        let key = ShaderKey::Material {
            vertex_layouts: vertex_layouts.iter().map(|v| v.clone()).collect::<Vec<_>>(),
            features: features.iter().map(|f| f.clone()).collect::<Vec<_>>()
        };
        
        self.shaders.get(&key)
    }
}
