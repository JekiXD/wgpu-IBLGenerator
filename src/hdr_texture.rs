

pub struct HdrTexture
{
  texture : wgpu::Texture,
  size : wgpu::Extent3d,
  view : wgpu::TextureView
}

impl HdrTexture 
{
  pub fn new( device : &wgpu::Device, width : u32, height : u32 ) -> Self
  {
    let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    let texture = device.create_texture
    (
      &wgpu::TextureDescriptor
      {
        label : Option::Some( "HDR_TEXTURE" ),
        size,
        mip_level_count : 1,
        sample_count : 1,
        dimension : wgpu::TextureDimension::D2,
        format : wgpu::TextureFormat::Rgba32Float,
        usage : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats : &[]
      }
    );

    let view = texture.create_view
    ( 
      &wgpu::TextureViewDescriptor
      {
        format : Some( wgpu::TextureFormat::Rgba32Float ),
        ..Default::default()
      }
    );

    Self 
    {
      texture,
      size,
      view
    }
  }

  pub fn size( &self ) -> wgpu::Extent3d
  {
    self.size
  }

  pub fn view( &self ) -> &wgpu::TextureView
  {
    &self.view
  }

  pub fn write_pixels( &self, queue : &wgpu::Queue, pixels : &[ f32 ] )
  {
    queue.write_texture
    (
      wgpu::TexelCopyTextureInfoBase 
      { 
        texture: &self.texture, 
        mip_level: 0, 
        origin: wgpu::Origin3d::ZERO, 
        aspect: wgpu::TextureAspect::All 
      }, 
      bytemuck::cast_slice( pixels ), 
      wgpu::TexelCopyBufferLayout 
      { 
        offset: 0, 
        bytes_per_row: Some( self.size.width * 4 * 4 ), 
        rows_per_image: None 
      },
      self.size
    );
  } 

}