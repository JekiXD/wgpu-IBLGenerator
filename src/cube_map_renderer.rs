use std::rc::Rc;

use crate::{cube_texture::CubeTexture, texture_2d::Texture2D};



pub struct CubeMapRenderer
{
  cube_texture : Rc< CubeTexture >,
  hdr_texture : Rc< Texture2D >,
  bind_group : wgpu::BindGroup,
  pipeline : wgpu::ComputePipeline
}

impl CubeMapRenderer
{
  pub fn new( cube_texture : Rc< CubeTexture >, hdr_texture : Rc< Texture2D >, device : &wgpu::Device ) -> Self
  {
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
            visibility: wgpu::ShaderStages::COMPUTE, 
            ty: wgpu::BindingType::StorageTexture 
            { 
              access: wgpu::StorageTextureAccess::WriteOnly, 
              format: wgpu::TextureFormat::Rgba32Float, 
              view_dimension: wgpu::TextureViewDimension::D2Array 
            }, 
            count: None 
          },
          wgpu::BindGroupLayoutEntry 
          { 
            binding: 1, 
            visibility: wgpu::ShaderStages::all(), 
            ty: wgpu::BindingType::Texture 
            { 
              sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
              view_dimension: wgpu::TextureViewDimension::D2, 
              multisampled: false 
            }, 
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
        entries : &
        [
          wgpu::BindGroupEntry
          {
            binding : 0,
            resource : wgpu::BindingResource::TextureView( cube_texture.view_2d() )
          },
          wgpu::BindGroupEntry
          {
            binding : 1,
            resource : wgpu::BindingResource::TextureView( hdr_texture.view() )
          },
        ]
      }
    );

    let shader = device.create_shader_module
    ( 
      wgpu::ShaderModuleDescriptor 
      { 
        label: None, 
        source: wgpu::ShaderSource::Wgsl( include_str!( "shaders/cube_map.wgsl" ).into() )
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

    let pipeline = device.create_compute_pipeline
    (
      &wgpu::ComputePipelineDescriptor
      {
        label : None,
        layout : Some( &pipeline_layout ),
        module : &shader,
        entry_point : None,
        compilation_options : wgpu::PipelineCompilationOptions::default(),
        cache : None
      }
    );

    

    Self 
    { 
      cube_texture,
      hdr_texture,
      bind_group,
      pipeline
    }
  }  

  pub fn render( &self, encoder : &mut wgpu::CommandEncoder )
  {
    let dst_size = self.cube_texture.size();
    let num_groups = dst_size.width.div_ceil( 16 );
    {
      let mut compute_pass = encoder.begin_compute_pass( &wgpu::ComputePassDescriptor::default() );
      compute_pass.set_pipeline( &self.pipeline );
      compute_pass.set_bind_group( 0, &self.bind_group, &[] );
      compute_pass.dispatch_workgroups( num_groups, num_groups, 6 );
    }
  }  
}