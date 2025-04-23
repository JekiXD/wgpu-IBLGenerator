use std::rc::Rc;

use crate::cube_texture::CubeTexture;



pub struct CubeMipmapRenderer
{
  cube_texture : Rc< CubeTexture >,
  pipeline : wgpu::RenderPipeline,
  sampler : wgpu::Sampler,
  bind_group_layout : wgpu::BindGroupLayout
}

impl CubeMipmapRenderer 
{
  pub fn new( device : &wgpu::Device, cube_texture : Rc< CubeTexture > ) -> Self
  {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor 
    {
      label : None,
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

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
              view_dimension: wgpu::TextureViewDimension::D2, 
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

    let shader = device.create_shader_module
    ( 
      wgpu::ShaderModuleDescriptor 
      { 
        label: None, 
        source: wgpu::ShaderSource::Wgsl( include_str!( "shaders/mipmap.wgsl" ).into() )
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

    let pipeline = device.create_render_pipeline
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
        fragment : Some( wgpu::FragmentState 
          { 
            module: &shader, 
            entry_point: None, 
            compilation_options: wgpu::PipelineCompilationOptions::default(), 
            targets: &[
              Some( wgpu::ColorTargetState 
                { 
                  format: cube_texture.format(), 
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

    Self 
    { 
      cube_texture, 
      pipeline,
      sampler,
      bind_group_layout
    }
  }

  pub fn generate_mipmaps( &self, device : &wgpu::Device, encoder : &mut wgpu::CommandEncoder )
  {
    let mip_levels = self.cube_texture.size().max_mips( wgpu::TextureDimension::D2 );

    let views = ( 0..mip_levels ).into_iter().flat_map( | mip_level | 
    {
      ( 0..6 ).into_iter().map( | array_level | 
      {
        self.cube_texture.create_mip_view( array_level, mip_level )
      }).collect::< Vec< wgpu::TextureView > >()
    })
    .collect::< Vec< wgpu::TextureView > >();


    for mip_level in 1..mip_levels
    {
      for array_level in 0..6
      {
        let src_view = &views[ ( ( mip_level - 1 ) * 6 + array_level ) as usize ];
        let dst_view = &views[ ( mip_level * 6 + array_level ) as usize ];

        let bind_group = device.create_bind_group
        (
          &wgpu::BindGroupDescriptor
          {
            label : None,
            layout : &self.bind_group_layout,
            entries : &[
              wgpu::BindGroupEntry
              {
                binding : 0,
                resource : wgpu::BindingResource::TextureView( src_view )
              },
              wgpu::BindGroupEntry
              {
                binding : 1,
                resource : wgpu::BindingResource::Sampler( &self.sampler )
              }
            ]
          }
        );
  
        let mut render_pass = encoder.begin_render_pass
        (
          &wgpu::RenderPassDescriptor
          {
            label : None,
            color_attachments : &
            [
              Some( wgpu::RenderPassColorAttachment
              {
                view : dst_view,
                resolve_target : None,
                ops : wgpu::Operations 
                { 
                  load: wgpu::LoadOp::Clear( wgpu::Color::WHITE ), 
                  store: wgpu::StoreOp::Store
                }
              })
            ],
            ..Default::default()
          }
        );
        render_pass.set_pipeline( &self.pipeline );
        render_pass.set_bind_group( 0, &bind_group, &[] );
        render_pass.draw( 0..3, 0..1 );
      }
    }
  }
}