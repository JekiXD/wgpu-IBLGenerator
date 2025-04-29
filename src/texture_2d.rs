

pub struct Texture2D
{
  texture : wgpu::Texture,
  size : wgpu::Extent3d,
  view : wgpu::TextureView,
  format : wgpu::TextureFormat
}

impl Texture2D 
{
  pub fn new
  ( 
    device : &wgpu::Device,
    format : wgpu::TextureFormat, 
    width : u32, 
    height : u32,
    with_mips : bool
) -> Self
  {
    let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    let mip_level_count = if with_mips { size.max_mips( wgpu::TextureDimension::D2 ) } else { 1 };
    let texture = device.create_texture
    (
      &wgpu::TextureDescriptor
      {
        label : Option::Some( "2D_TEXTURE" ),
        size,
        mip_level_count,
        sample_count : 1,
        dimension : wgpu::TextureDimension::D2,
        format,
        usage : wgpu::TextureUsages::TEXTURE_BINDING 
        | wgpu::TextureUsages::COPY_DST 
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC,
        view_formats : &[]
      }
    );

    let view = texture.create_view( &wgpu::TextureViewDescriptor::default() );

    Self 
    {
      texture,
      size,
      view,
      format
    }
  }

  pub fn size( &self ) -> wgpu::Extent3d
  {
    self.size
  }

  pub fn mip_count( &self ) -> u32
  {
    self.texture.mip_level_count()
  }

  pub fn view( &self ) -> &wgpu::TextureView
  {
    &self.view
  }

  pub fn texture( &self ) -> &wgpu::Texture
  {
    &self.texture
  }

  pub fn mip_level_size( &self, mip_level : u32 ) -> wgpu::Extent3d
  {
    self.size.mip_level_size( mip_level, wgpu::TextureDimension::D2 )
  }

  pub fn mip_memory_size( &self, mip_level : u32 ) -> u32
  {
    let size = self.mip_level_size( mip_level );
    self.format.block_copy_size( None ).unwrap() * size.width * size.height
  }

  pub fn mip_memory_size_row( &self, mip_level : u32 ) -> u32
  {
    let size = self.mip_level_size( mip_level );
    self.format.block_copy_size( None ).unwrap() * size.width
  }

  pub fn create_mip_view( &self, mip_level : u32 ) -> wgpu::TextureView
  {
    self.texture.create_view
    (
      &wgpu::TextureViewDescriptor
      {
        base_mip_level : mip_level,
        mip_level_count : Some( 1 ),
        base_array_layer : 0,
        array_layer_count : Some( 1 ),
        dimension : Some( wgpu::TextureViewDimension::D2 ),
        ..Default::default()
      }
    )
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
        bytes_per_row: Some( self.size.width * self.format.block_copy_size( None ).unwrap() ), 
        rows_per_image: None 
      },
      self.size
    );
  } 

}