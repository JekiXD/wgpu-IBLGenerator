use std::{rc::Rc, sync::Arc};

use image::{GenericImageView, ImageReader};
use winit::{event::WindowEvent, window::Window};

use crate::{camera::Uniform, cube_map_renderer::CubeMapRenderer, cube_mipmap_renderer::CubeMipmapRenderer, cube_texture::CubeTexture, ibl_renderer::IBLRenderer, texture_2d::Texture2D};

pub struct State {
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub window: Arc<Window>,
  pub surface: wgpu::Surface< 'static >,
  pipeline : wgpu::RenderPipeline,
  hdr_texture : Rc< Texture2D >,
  cube_texture : Rc< CubeTexture >,
  cm_renderer : CubeMapRenderer,
  uniform : Uniform,
  pub bind_group : wgpu::BindGroup,
  pub bind_group_layout : wgpu::BindGroupLayout,
  surface_format : wgpu::TextureFormat,
  cube_mipmap_renderer : CubeMipmapRenderer,
  ibl_renderer : IBLRenderer
}

impl State {
  pub async fn new( window: Arc<Window> ) -> Self
  {
    let instance = wgpu::Instance::new
    (
      &wgpu::InstanceDescriptor 
      {
        backends: wgpu::Backends::all(),
        ..Default::default()
      }
    );

    let window_size = window.inner_size();
    let surface = instance.create_surface( window.clone() ).unwrap();

    let adapter = instance.request_adapter
    (
      &wgpu::RequestAdapterOptions 
      {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some( &surface ),
        force_fallback_adapter: false,
      },
    ).await.unwrap();

    // Surface configuration
    let surface_caps = surface.get_capabilities( &adapter );
    let surface_format = surface_caps.formats.iter()
      .copied()
      .find( | f | f.is_srgb() )
      .unwrap_or( surface_caps.formats[ 0 ] );
    let config = wgpu::SurfaceConfiguration 
    {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: window_size.width,
      height: window_size.height,
      present_mode: surface_caps.present_modes[0],
      desired_maximum_frame_latency: 2,
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
    };

    let ( device, queue ) = adapter.request_device
    ( 
      &wgpu::DeviceDescriptor
      {
        required_features : wgpu::Features::FLOAT32_FILTERABLE,
        ..Default::default()
      } 
    ).await.unwrap();
    surface.configure( &device, &config );

    //let image = ImageReader::open( "assets/rogland_clear_night_4k.hdr" ).unwrap().decode().unwrap();
    //let image = ImageReader::open( "D:/VS_projects/Rust/IBLConverter/assets/rogland_clear_night_4k.hdr" ).unwrap().decode().unwrap();
    //let image = ImageReader::open( "D:/VS_projects/Rust/IBLConverter/assets/autumn_field_puresky_4k.hdr" ).unwrap().decode().unwrap();
    //let image = ImageReader::open( "D:/VS_projects/Rust/IBLConverter/assets/passendorf_snow_4k.hdr" ).unwrap().decode().unwrap();
    //let image = ImageReader::open( "D:/VS_projects/Rust/IBLConverter/assets/metro_noord_4k.hdr" ).unwrap().decode().unwrap();
    let image = ImageReader::open( "D:/VS_projects/Rust/IBLConverter/assets/kloppenheim_06_puresky_4k.hdr" ).unwrap().decode().unwrap();
    let image = image.to_rgba32f();
    let ( img_width, img_height ) = image.dimensions();
    let pixels = image.into_vec();

    let hdr_texture = Rc::new( Texture2D::new( &device, wgpu::TextureFormat::Rgba32Float, img_width, img_height ) );
    hdr_texture.write_pixels( &queue, &pixels );
    let cube_texture = Rc::new( CubeTexture::new( &device, 512, 512 ) );
    let uniform = Uniform::new( &device, window_size.width as f32, window_size.height as f32 );

    let cm_renderer = CubeMapRenderer::new( cube_texture.clone(), hdr_texture.clone(), &device );
    let cube_mipmap_renderer = CubeMipmapRenderer::new( &device, cube_texture.clone() );
    let ibl_renderer = IBLRenderer::new
    ( 
      &device, cube_texture.clone(), 
      wgpu::TextureFormat::Rgba32Float, 
      img_width, 
      img_height, 
      512, 
      512
    );

    let bind_group_layout = device.create_bind_group_layout
    (
      &wgpu::BindGroupLayoutDescriptor 
      { 
        label: None, 
        entries: &
        [
          wgpu::BindGroupLayoutEntry
          {
            binding : 0,
            visibility : wgpu::ShaderStages::all(),
            ty : wgpu::BindingType::Texture 
            { 
              sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
              view_dimension: wgpu::TextureViewDimension::Cube, 
              multisampled: false 
            },
            count : None
          },
          wgpu::BindGroupLayoutEntry
          {
            binding : 1,
            visibility : wgpu::ShaderStages::all(),
            ty : wgpu::BindingType::Sampler
            (
              wgpu::SamplerBindingType::Filtering
            ),
            count : None
          },
        ] 
      }
    );

    let bind_group = device.create_bind_group
    (
      &wgpu::BindGroupDescriptor
      {
        label : None,
        layout : &bind_group_layout,
        entries : &
        [
          wgpu::BindGroupEntry
          {
            binding : 0,
            resource : wgpu::BindingResource::TextureView( cube_texture.view_cube() )
          },
          wgpu::BindGroupEntry
          {
            binding : 1,
            resource : wgpu::BindingResource::Sampler( &cube_texture.sampler() )
          },
        ]
      }
    );

    let main_shader = device.create_shader_module
    (
      wgpu::ShaderModuleDescriptor 
      { 
        label: None, 
        source: wgpu::ShaderSource::Wgsl( include_str!( "shaders/main.wgsl" ).into() )
      }
    );

    let pipeline_layout = device.create_pipeline_layout
    (
      &wgpu::PipelineLayoutDescriptor
      {
        label : None,
        bind_group_layouts : &
        [
          &uniform.bind_group_layout,
          &bind_group_layout
        ],
        push_constant_ranges : &[]
      }
    );

    let pipeline = device.create_render_pipeline
    (
      &wgpu::RenderPipelineDescriptor
      {
        label : None,
        layout : Some( &pipeline_layout ),
        vertex : wgpu::VertexState
        {
          module : &main_shader,
          buffers : &[],
          entry_point : None,
          compilation_options : wgpu::PipelineCompilationOptions::default()
        },
        primitive : wgpu::PrimitiveState::default(),
        depth_stencil : None,
        multisample : wgpu::MultisampleState::default(),
        fragment : Some( wgpu::FragmentState
        {
          module : &main_shader,
          entry_point : None,
          compilation_options : wgpu::PipelineCompilationOptions::default(),
          targets : &
          [
            Some
            (
              wgpu::ColorTargetState 
              { 
                format: surface_format, 
                blend: None, 
                write_mask: wgpu::ColorWrites::all() 
              }
            )
          ]
        }),
        multiview: None,
        cache : None
      }
    );


    Self
    {
      device,
      queue,
      window,
      surface,
      pipeline,
      hdr_texture,
      cm_renderer,
      cube_texture,
      uniform,
      bind_group,
      bind_group_layout,
      surface_format,
      cube_mipmap_renderer,
      ibl_renderer
    }
  }

  pub fn input( &mut self, _event: &WindowEvent ) -> bool 
  {
    false
  }

  pub fn update( &mut self ) 
  {
    self.uniform.update( &self.queue );
  }

  pub fn render_hdr_to_cube( &mut self )
  {
    let mut encoder = self.device.create_command_encoder( &wgpu::CommandEncoderDescriptor::default() );

    self.cm_renderer.render( &mut encoder );
    self.cube_mipmap_renderer.generate_mipmaps( &self.device, &mut encoder );
    self.ibl_renderer.render( &mut encoder );

    self.queue.submit( std::iter::once( encoder.finish() ) );
  }

  pub async fn save_ibl( &self )
  {
    self.ibl_renderer.save( &self.device ).await;
  }

  pub fn render( &mut self ) -> Result< (), wgpu::SurfaceError > 
  {
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view( &wgpu::TextureViewDescriptor::default() );

    let mut encoder = self.device.create_command_encoder( &wgpu::CommandEncoderDescriptor::default() );

    {
      let mut render_pass = encoder.begin_render_pass
      (
        &wgpu::RenderPassDescriptor
        {
          label : None,
          color_attachments : &
          [
            Some( wgpu::RenderPassColorAttachment
            {
              view : &view,
              resolve_target : None,
              ops : wgpu::Operations 
              { 
                load: wgpu::LoadOp::Clear( wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } ), 
                store: wgpu::StoreOp::Store 
              }
            })
          ],
          depth_stencil_attachment : None,
          timestamp_writes : None,
          occlusion_query_set : None
        }
      );

      render_pass.set_pipeline( &self.pipeline );
      
      render_pass.set_bind_group( 0, &self.uniform.bind_group, &[] );
      render_pass.set_bind_group( 1, &self.bind_group, &[] );

      render_pass.draw( 0..3, 0..1 );
    }

    self.queue.submit( std::iter::once( encoder.finish() ) );
    output.present();

    Ok( () )
  }

}