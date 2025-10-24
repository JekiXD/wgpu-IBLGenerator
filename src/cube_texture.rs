

pub struct CubeTexture
{
  texture : wgpu::Texture,
  sampler : wgpu::Sampler,
  view_cube : wgpu::TextureView,
  view_2d : wgpu::TextureView,
  format : wgpu::TextureFormat,
  size : wgpu::Extent3d
}

impl CubeTexture 
{
  pub fn new( device : &wgpu::Device, format : wgpu::TextureFormat, width : u32, height : u32, with_mips : bool ) -> Self
  {
    let size = wgpu::Extent3d { width, height, depth_or_array_layers: 6 };
    let mip_level_count = if with_mips { size.max_mips( wgpu::TextureDimension::D2 ) } else { 1 };

    let texture = device.create_texture
    (
      &wgpu::TextureDescriptor
      {
        label : Option::Some( "CUBE_TEXTURE" ), 
        size,
        mip_level_count,
        sample_count : 1,
        dimension : wgpu::TextureDimension::D2,
        format,
        usage : wgpu::TextureUsages::TEXTURE_BINDING 
        | wgpu::TextureUsages::STORAGE_BINDING 
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC,
        view_formats : &[]
      }
    );

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor 
    {
      label : None,
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });

    let view_cube = texture.create_view
    (
      &wgpu::TextureViewDescriptor
      {
        dimension : Some( wgpu::TextureViewDimension::Cube ),
        ..Default::default()
      }
    );

    let view_2d = texture.create_view
    (
      &wgpu::TextureViewDescriptor
      {
        dimension : Some( wgpu::TextureViewDimension::D2Array ),
        base_mip_level : 0,
        mip_level_count : Some( 1 ),
        ..Default::default()
      }
    );

    Self 
    {
      texture,
      sampler,
      view_cube,
      view_2d,
      format,
      size
    }
  }   

  pub fn view_cube( &self ) -> &wgpu::TextureView { &self.view_cube }

  pub fn view_2d( &self ) -> &wgpu::TextureView { &self.view_2d }

  pub fn create_mip_view( &self, array_level : u32, mip_level : u32 ) -> wgpu::TextureView
  {
    self.texture.create_view
    (
      &wgpu::TextureViewDescriptor
      {
        base_mip_level : mip_level,
        mip_level_count : Some( 1 ),
        base_array_layer : array_level,
        array_layer_count : Some( 1 ),
        dimension : Some( wgpu::TextureViewDimension::D2 ),
        ..Default::default()
      }
    )
  }

  pub fn mip_level_size( &self, mip_level : u32 ) -> wgpu::Extent3d
  {
    self.size.mip_level_size( mip_level, wgpu::TextureDimension::D2 )
  }

  pub fn mip_memory_size( &self, mip_level : u32 ) -> u32
  {
    let size = self.mip_level_size( mip_level );
    self.format.block_copy_size( None ).unwrap() * size.width * size.height * size.depth_or_array_layers
  }

  pub fn mip_memory_size_row( &self, mip_level : u32 ) -> u32
  {
    let size = self.mip_level_size( mip_level );
    self.format.block_copy_size( None ).unwrap() * size.width
  }

  pub fn format( &self ) -> wgpu::TextureFormat { self.format }

  pub fn sampler( &self ) -> &wgpu::Sampler { &self.sampler }

  pub fn texture( &self ) -> &wgpu::Texture { &self.texture }

  pub fn size( &self ) -> wgpu::Extent3d { self.size }
}