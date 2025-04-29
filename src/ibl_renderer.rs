use std::{fs::File, rc::Rc};

use crate::{cube_texture::CubeTexture, texture_2d::Texture2D};

struct UniformRaw
{
  mip_level : u32,
  total_mips : u32
}

struct BufferWrapper
{
  pub buffer : wgpu::Buffer,
  pub unpadded_bytes_per_row : u32,
  pub padded_bytes_per_row : u32,
  pub num_rows : u32
}

pub struct IBLRenderer
{
  env_map : Rc< CubeTexture >,
  diffuse_texture : Texture2D,
  specular_1_texture : Texture2D,
  specular_2_texture : Texture2D,
  bind_group : wgpu::BindGroup,
  diffuse_pipeline : wgpu::RenderPipeline,
  specular_1_pipeline : wgpu::RenderPipeline,
  specular_2_pipeline : wgpu::RenderPipeline,
  diffuse_buffer : wgpu::Buffer,
  specular_1_buffers : Vec< BufferWrapper >,
  specular_2_buffer : wgpu::Buffer,
  uniform_buffer : wgpu::Buffer,
  total_mips : u32
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
    specular_1_width : u32,
    specular_1_height : u32,
    specular_2_width : u32,
    specular_2_height : u32
  ) -> Self
  { 
    let specular_1_format = wgpu::TextureFormat::Rgba32Float;
    let specular_2_format = wgpu::TextureFormat::Rgba32Float;

    let diffuse_size = wgpu::Extent3d{ width: diffuse_width, height: diffuse_height, depth_or_array_layers: 1 };
    let specular_1_size = wgpu::Extent3d{ width: specular_1_width, height: specular_1_height, depth_or_array_layers: 1 };
    let specular_2_size = wgpu::Extent3d{ width: specular_2_width, height: specular_2_height, depth_or_array_layers: 1 };

    let diffuse_texture = Texture2D::new( device, format, diffuse_width, diffuse_height, false );
    let specular_1_texture = Texture2D::new( device, specular_1_format, specular_1_width, specular_1_height, true );
    let specular_2_texture = Texture2D::new( device, specular_2_format, specular_2_width, specular_2_height, false );

    let total_mips = specular_1_size.max_mips( wgpu::TextureDimension::D2 ).min( 5 );

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
          },
          wgpu::BindGroupLayoutEntry 
          { 
            binding: 2, 
            visibility: wgpu::ShaderStages::FRAGMENT, 
            ty: wgpu::BindingType::Buffer 
            { 
              ty: wgpu::BufferBindingType::Uniform, 
              has_dynamic_offset: false, 
              min_binding_size: None 
            }, 
            count: None 
          },
        ] 
      }
    );

    let uniform_buffer = device.create_buffer
    (
      &wgpu::BufferDescriptor
      {
        label : None,
        size : std::mem::size_of::< UniformRaw >() as u64,
        mapped_at_creation : false,
        usage : wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM
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
          wgpu::BindGroupEntry
          {
            binding : 2,
            resource : wgpu::BindingResource::Buffer( uniform_buffer.as_entire_buffer_binding() )
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

    let specular_1_pipeline = device.create_render_pipeline
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
            entry_point: Some( "fragment_specular_1_main" ), 
            compilation_options: wgpu::PipelineCompilationOptions::default(), 
            targets: &[
              Some( wgpu::ColorTargetState 
                { 
                  format: specular_2_format, 
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

    let specular_2_pipeline = device.create_render_pipeline
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
            entry_point: Some( "fragment_specular_2_main" ), 
            compilation_options: wgpu::PipelineCompilationOptions::default(), 
            targets: &[
              Some( wgpu::ColorTargetState 
                { 
                  format: specular_2_format, 
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
        size : diffuse_texture.mip_memory_size( 0 ) as u64,
        mapped_at_creation : false,
        usage : wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST
      }
    );

    let bytes_per_texel = specular_1_format.block_copy_size( None ).unwrap();
    let mut specular_1_buffers = Vec::new();
    for i in 0..total_mips
    {
      let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
      let size = specular_1_texture.mip_level_size( i );
      let padded_bytes_per_row = ( bytes_per_texel * size.width ).div_ceil( alignment ) * alignment;
      let buffer = device.create_buffer
      (
        &wgpu::BufferDescriptor
        {
          label : None,
          size : ( padded_bytes_per_row * size.height  ) as u64,
          mapped_at_creation : false,
          usage : wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST
        }
      );

      let wrapper = BufferWrapper
      {
        buffer,
        unpadded_bytes_per_row : size.width * bytes_per_texel,
        padded_bytes_per_row,
        num_rows : size.height
      };
      specular_1_buffers.push( wrapper );
    }

    let specular_2_buffer = device.create_buffer
    (
      &wgpu::BufferDescriptor
      {
        label : None,
        size : specular_2_texture.mip_memory_size( 0 ) as u64,
        mapped_at_creation : false,
        usage : wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST
      }
    );

    Self
    {
      env_map,
      diffuse_texture,
      specular_1_texture,
      specular_2_texture,
      bind_group,
      diffuse_pipeline,
      specular_1_pipeline,
      specular_2_pipeline,
      diffuse_buffer,
      specular_1_buffers,
      specular_2_buffer,
      uniform_buffer,
      total_mips
    }
  }

  pub fn render_diffuse( &self, encoder : &mut wgpu::CommandEncoder )
  {
    let diffuse_view = self.diffuse_texture.view();

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

  pub fn render_specular_1( &self, encoder : &mut wgpu::CommandEncoder, queue : &wgpu::Queue )
  {
    for mip_level in 0..self.total_mips
    {
      queue.write_buffer
      (
        &self.uniform_buffer, 
        0, 
        bytemuck::cast_slice( &[ mip_level, self.total_mips ] )
      );

      let view = self.specular_1_texture.create_mip_view( mip_level );

      {
        let mut render_pass = encoder.begin_render_pass
        (
          &wgpu::RenderPassDescriptor
          {
            label : None,
            color_attachments : &[
              Some( wgpu::RenderPassColorAttachment
              {
                view : &view,
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
  
        render_pass.set_pipeline( &self.specular_1_pipeline );
        render_pass.set_bind_group( 0, &self.bind_group, &[] );
        render_pass.draw( 0..3, 0..1 );
      }
  
      // Copy diffuse texture to the buffer
      let wrapper = &self.specular_1_buffers[ mip_level as usize ];
      let texture = self.specular_1_texture.texture();
      let size = self.specular_1_texture.mip_level_size( mip_level );
      encoder.copy_texture_to_buffer
      (
        wgpu::TexelCopyTextureInfoBase 
        { 
          texture: texture, 
          mip_level, 
          origin: wgpu::Origin3d::ZERO, 
          aspect: wgpu::TextureAspect::All 
        }, 
        wgpu::TexelCopyBufferInfo
        {
          buffer : &wrapper.buffer,
          layout : wgpu::TexelCopyBufferLayout
          {
            offset : 0,
            bytes_per_row : Some( wrapper.padded_bytes_per_row ),
            rows_per_image : None
          }
        },
        size
      );
    }
  }

  pub fn render_specular_2( &self, encoder : &mut wgpu::CommandEncoder )
  {
    let specular_view = self.specular_2_texture.view();

    {
      let mut render_pass = encoder.begin_render_pass
      (
        &wgpu::RenderPassDescriptor
        {
          label : None,
          color_attachments : &[
            Some( wgpu::RenderPassColorAttachment
            {
              view : specular_view,
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

      render_pass.set_pipeline( &self.specular_2_pipeline );
      render_pass.set_bind_group( 0, &self.bind_group, &[] );
      render_pass.draw( 0..3, 0..1 );
    }

    // Copy diffuse texture to the buffer
    let specular_2_texture = self.specular_2_texture.texture();
    encoder.copy_texture_to_buffer
    (
      wgpu::TexelCopyTextureInfoBase 
      { 
        texture: specular_2_texture, 
        mip_level: 0, 
        origin: wgpu::Origin3d::ZERO, 
        aspect: wgpu::TextureAspect::All 
      }, 
      wgpu::TexelCopyBufferInfo
      {
        buffer : &self.specular_2_buffer,
        layout : wgpu::TexelCopyBufferLayout
        {
          offset : 0,
          bytes_per_row : Some( specular_2_texture.width() * specular_2_texture.format().block_copy_size( None ).unwrap() ),
          rows_per_image : None
        }
      },
      wgpu::Extent3d 
      { 
        width: specular_2_texture.width(), 
        height: specular_2_texture.height(), 
        depth_or_array_layers: 1 
      }
    );
  }

  pub async fn save_diffuse( &self, device : &wgpu::Device )
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

  pub async fn save_specular_2( &self, device : &wgpu::Device )
  {
    let ( sender, reciever ) = flume::bounded( 1 );

    self.specular_2_buffer.map_async
    (
      wgpu::MapMode::Read, 
      .., 
      move | r | sender.send( r ).unwrap()
    );

    device.poll( wgpu::PollType::wait() ).unwrap();
    reciever.recv_async().await.unwrap().unwrap();

    use image::ImageEncoder;

    {
      let size = self.specular_2_texture.size();
      let view = self.specular_2_buffer.get_mapped_range( .. );

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

      let file = File::create( "result/specular_2.hdr" ).unwrap();
      
      let encoder = image::codecs::hdr::HdrEncoder::new( file );
      encoder.write_image( &data, size.width, size.height, image::ExtendedColorType::Rgb32F ).unwrap();

      // let file = File::create( "result/specular_2.png" ).unwrap();

      // let encoder = image::codecs::png::PngEncoder::new( file );
      // encoder.write_image( &view, size.width, size.height, image::ExtendedColorType::Rgba8 ).unwrap();
    }
  }

  pub async fn save_specular_1( &self, device : &wgpu::Device )
  {
    use image::ImageEncoder;
    let ( sender, reciever ) = flume::bounded( 1 );

    for mip_level in 0..self.total_mips
    {
      let sender = sender.clone();
      let wrapper = &self.specular_1_buffers[ mip_level as usize ];
      let buffer = &wrapper.buffer;
      buffer.map_async
      (
        wgpu::MapMode::Read, 
        .., 
        move | r | sender.send( r ).unwrap()
      );

      device.poll( wgpu::PollType::wait() ).unwrap();
      reciever.recv_async().await.unwrap().unwrap();

      {
        let size = self.specular_1_texture.mip_level_size( mip_level );
        let view = buffer.get_mapped_range( .. );

        let mut data = Vec::with_capacity( ( wrapper.unpadded_bytes_per_row * wrapper.num_rows ) as usize );
        for row in 0..wrapper.num_rows
        {
          let start = ( wrapper.padded_bytes_per_row * row ) as usize;
          let end = start + wrapper.unpadded_bytes_per_row as usize;
          data.extend_from_slice( &view[ start..end ] );
        }

        // HDR type only support RGBF32, so we need to remove the alpha channel
        let data = data.iter().enumerate().filter( | ( i, _v ) |
        {
          let rem = i % 16;
          rem != 15 && rem != 14 && rem != 13 && rem != 12 
        })
        .map( | ( _i, v ) |
        {
          *v
        })
        .collect::< Vec< u8 > >();

        let file = File::create( format!( "result/specular_1_{}.hdr", mip_level ) ).unwrap();
        
        let encoder = image::codecs::hdr::HdrEncoder::new( file );
        encoder.write_image( &data, size.width, size.height, image::ExtendedColorType::Rgb32F ).unwrap();

        // let file = File::create( format!( "result/specular_1_{}.png", mip_level ) ).unwrap();

        // let encoder = image::codecs::png::PngEncoder::new( file );
        // encoder.write_image( &view, size.width, size.height, image::ExtendedColorType::Rgba8 ).unwrap();
      }
    }
  }
}