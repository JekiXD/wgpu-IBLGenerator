use crate::{cube_texture::CubeTexture, texture_2d::Texture2D};



pub struct IBLRenderer
{
  env_map : Rc< CubeTexture >,
  ibl_diffuse : Texture2D,
  ibl_specular : Texture2D
}

impl IBLRenderer 
{
  pub fn new
  ( 
    device : &wgpu::Device, 
    env_map : Rc< CubeTexture >, 
    format : wgpu::TextureFormat, 
    width : u32, 
    height : u32 
  )
  { 
    let size = wgpu::Extent3d{ width, height, depth_or_array_layers: 1 };
    let ibl_diffuse = device.create_texture
    (
      &wgpu::TextureDescriptor
      {
        size,
        dimension : wgpu::TextureDimension::D2,
        mip_level_count : 1,
        label : None,
        sample_count : 1,
        usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
        format : wgpu::
      }
    )
  }
}