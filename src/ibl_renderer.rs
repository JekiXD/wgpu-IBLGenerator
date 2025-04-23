use std::{fs::File, rc::Rc};

use crate::{cube_texture::CubeTexture, texture_2d::Texture2D};



pub struct IBLRenderer
{
  env_map : Rc< CubeTexture >,
  diffuse_texture : Texture2D,
  specular_texture : Texture2D,
  bind_group : wgpu::BindGroup,
  diffuse_pipeline : wgpu::RenderPipeline,
  specular_pipeline : wgpu::RenderPipeline,
  diffuse_buffer : wgpu::Buffer,
  specular_buffer : wgpu::Buffer
}

impl IBLRenderer 
{
  pub fn new
  ( 
    device : &wgpu::Device, 
    env_map : Rc< CubeTexture >, 
    format : wgpu::TextureFormat, 
    diffuse_width : u32, 
    diffuse_height : u32,
    specular_width : u32,
    specular_height : u32
  ) -> Self
  { 
    let specular_format = wgpu::TextureFormat::Rgba8Unorm;

    let diffuse_size = wgpu::Extent3d{ width: diffuse_width, height: diffuse_height, depth_or_array_layers: 1 };
    let specular_size = wgpu::Extent3d{ width: specular_width, height: specular_height, depth_or_array_layers: 1 };

    let diffuse_texture = Texture2D::new( device, format, diffuse_width, diffuse_height );
    let specular_texture = Texture2D::new( device, specular_format, specular_width, specular_height );

    let bind_group_layout = device.create_bind_group_layout
    (
      &wgpu::BindGroupLayoutDescriptor 
      { 
        label: None, 
        entries: &
        [
          wgpu::BindGroupLayoutEntry 
          { 
            binding: 0, 
            visibility: wgpu::ShaderStages::FRAGMENT, 
            ty: wgpu::BindingType::Texture 
            { 
              sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
              view_dimension: wgpu::TextureViewDimension::Cube, 
              multisampled: false 
            },
            count: None 
          },
          wgpu::BindGroupLayoutEntry 
          { 
            binding: 1, 
            visibility: wgpu::ShaderStages::FRAGMENT, 
            ty: wgpu::BindingType::Sampler
            (
              wgpu::SamplerBindingType::Filtering
            ), 
            count: None 
          }
        ] 
      }
    );

    let bind_group = device.create_bind_group
    (
      &wgpu::BindGroupDescriptor
      {
        label : None,
        layout : &bind_group_layout,
        entries : &[
          wgpu::BindGroupEntry
          {
            binding : 0,
            resource : wgpu::BindingResource::TextureView( &env_map.view_cube() )
          },
          wgpu::BindGroupEntry
          {
            binding : 1,
            resource : wgpu::BindingResource::Sampler( &env_map.sampler() )
          },
        ]
      }
    );

    let shader = device.create_shader_module
    ( 
      wgpu::ShaderModuleDescriptor 
      { 
        label: None, 
        source: wgpu::ShaderSource::Wgsl( include_str!( "shaders/ibl.wgsl" ).into() )
      }
    );

    let pipeline_layout = device.create_pipeline_layout
    (
      &wgpu::PipelineLayoutDescriptor
      {
        label : None,
        bind_group_layouts : &
        [
          &bind_group_layout
        ],
        push_constant_ranges : &[]
      }
    );

    let diffuse_pipeline = device.create_render_pipeline
    (
      &wgpu::RenderPipelineDescriptor
      {
        label : None,
        layout : Some( &pipeline_layout ),
        vertex : wgpu::VertexState 
        {
          module: &shader, 
          entry_point: None, 
          compilation_options: wgpu::PipelineCompilationOptions::default(), 
          buffers: &[] 
        },
        primitive : wgpu::PrimitiveState
        {
          topology : wgpu::PrimitiveTopology::TriangleList,
          ..Default::default()
        },
        depth_stencil : None,
        fragment : Some( 
          wgpu::FragmentState 
          { 
            module: &shader, 
            entry_point: Some( "fragment_diffuse_main" ), 
            compilation_options: wgpu::PipelineCompilationOptions::default(), 
            targets: &[
              Some( wgpu::ColorTargetState 
                { 
                  format, 
                  blend: None, 
                  write_mask: wgpu::ColorWrites::all() 
                }
              )
            ] 
          }
        ),
        multisample : wgpu::MultisampleState::default(),
        multiview : None,
        cache : None
      }
    );

    let specular_pipeline = device.create_render_pipeline
    (
      &wgpu::RenderPipelineDescriptor
      {
        label : None,
        layout : Some( &pipeline_layout ),
        vertex : wgpu::VertexState 
        {
          module: &shader, 
          entry_point: None, 
          compilation_options: wgpu::PipelineCompilationOptions::default(), 
          buffers: &[] 
        },
        primitive : wgpu::PrimitiveState
        {
          topology : wgpu::PrimitiveTopology::TriangleList,
          ..Default::default()
        },
        depth_stencil : None,
        fragment : Some( 
          wgpu::FragmentState 
          { 
            module: &shader, 
            entry_point: Some( "fragment_specular_main" ), 
            compilation_options: wgpu::PipelineCompilationOptions::default(), 
            targets: &[
              Some( wgpu::ColorTargetState 
                { 
                  format, 
                  blend: None, 
                  write_mask: wgpu::ColorWrites::all() 
                }
              )
            ] 
          }
        ),
        multisample : wgpu::MultisampleState::default(),
        multiview : None,
        cache : None
      }
    );

    let diffuse_buffer = device.create_buffer
    (
      &wgpu::BufferDescriptor
      {
        label : None,
        size : ( format.block_copy_size( None ).unwrap() * diffuse_width * diffuse_height ) as u64,
        mapped_at_creation : false,
        usage : wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST
      }
    );

    let specular_buffer = device.create_buffer
    (
      &wgpu::BufferDescriptor
      {
        label : None,
        size : ( specular_format.block_copy_size( None ).unwrap() * specular_width * specular_height ) as u64,
        mapped_at_creation : false,
        usage : wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST
      }
    );

    Self
    {
      env_map,
      diffuse_texture,
      specular_texture,
      bind_group,
      diffuse_pipeline,
      specular_pipeline,
      diffuse_buffer,
      specular_buffer
    }
  }

  pub fn render( &self, encoder : &mut wgpu::CommandEncoder )
  {
    let diffuse_view = self.diffuse_texture.view();
    let specular_view = self.specular_texture.view();

    {
      let mut render_pass = encoder.begin_render_pass
      (
        &wgpu::RenderPassDescriptor
        {
          label : None,
          color_attachments : &[
            Some( wgpu::RenderPassColorAttachment
            {
              view : diffuse_view,
              resolve_target : None,
              ops : wgpu::Operations
              {
                load : wgpu::LoadOp::Clear( wgpu::Color::BLACK ),
                store : wgpu::StoreOp::Store
              }
            })
          ],
          depth_stencil_attachment : None,
          timestamp_writes : None,
          occlusion_query_set : None
        }
      );

      render_pass.set_pipeline( &self.diffuse_pipeline );
      render_pass.set_bind_group( 0, &self.bind_group, &[] );
      render_pass.draw( 0..3, 0..1 );
    }

    // Copy diffuse texture to the buffer
    let diffuse_texture = self.diffuse_texture.texture();
    encoder.copy_texture_to_buffer
    (
      wgpu::TexelCopyTextureInfoBase 
      { 
        texture: diffuse_texture, 
        mip_level: 0, 
        origin: wgpu::Origin3d::ZERO, 
        aspect: wgpu::TextureAspect::All 
      }, 
      wgpu::TexelCopyBufferInfo
      {
        buffer : &self.diffuse_buffer,
        layout : wgpu::TexelCopyBufferLayout
        {
          offset : 0,
          bytes_per_row : Some( diffuse_texture.width() * diffuse_texture.format().block_copy_size( None ).unwrap() ),
          rows_per_image : None
        }
      },
      wgpu::Extent3d 
      { 
        width: diffuse_texture.width(), 
        height: diffuse_texture.height(), 
        depth_or_array_layers: 1 
      }
    );
  }

  pub async fn save( &self, device : &wgpu::Device )
  {
    let ( sender, reciever ) = flume::bounded( 1 );

    self.diffuse_buffer.map_async
    (
      wgpu::MapMode::Read, 
      .., 
      move | r | sender.send( r ).unwrap()
    );

    device.poll( wgpu::PollType::wait() ).unwrap();
    reciever.recv_async().await.unwrap().unwrap();

    use image::ImageEncoder;

    {
      let size = self.diffuse_texture.size();
      let view = self.diffuse_buffer.get_mapped_range( .. );

      // HDR type only support RGBF32, so we need to remove the alpha channel
      let data = view.iter().enumerate().filter( | ( i, _v ) |
      {
        let rem = i % 16;
        rem != 15 && rem != 14 && rem != 13 && rem != 12 
      })
      .map( | ( _i, v ) |
      {
        *v
      })
      .collect::< Vec< u8 > >();

      let file = File::create( "result/diffuse.hdr" ).unwrap();
      
      let encoder = image::codecs::hdr::HdrEncoder::new( file );
      encoder.write_image( &data, size.width, size.height, image::ExtendedColorType::Rgb32F ).unwrap();
    }

  }
}