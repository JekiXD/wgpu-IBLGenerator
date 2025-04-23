#[ repr( C ) ]
#[ derive( Clone, Copy, Default, bytemuck::NoUninit ) ]
pub struct UniformRaw
{
  view_matrix : [ f32; 16 ],
  inverse_view_matrix : [ f32; 16 ],
  projection_matrix : [ f32; 16 ],
  inverse_projection_matrix : [ f32; 16 ],
  time : f32,
  padding : [ f32; 3 ]
}

pub struct Uniform
{
  camera : Camera,
  timer : Timer,
  buffer : wgpu::Buffer,
  pub bind_group : wgpu::BindGroup,
  pub bind_group_layout : wgpu::BindGroupLayout
}

impl Uniform 
{
  pub fn new( device : &wgpu::Device, width : f32, height : f32 ) -> Self
  {
    let timer = Timer::new();
    let camera = Camera::new( 70.0f32.to_radians(), width / height, 0.1, 1000.0 );

    let buffer = device.create_buffer
    (
      &wgpu::BufferDescriptor
      {
        label : None,
        size : std::mem::size_of::< UniformRaw >() as u64,
        usage : wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation : false
      }
    );

    let bind_group_layout = device.create_bind_group_layout
    (
      &wgpu::BindGroupLayoutDescriptor
      {
        label : None,
        entries : &
        [
          wgpu::BindGroupLayoutEntry
          {
            binding : 0,
            visibility : wgpu::ShaderStages::all(),
            ty : wgpu::BindingType::Buffer 
            { 
              ty : wgpu::BufferBindingType::Uniform, 
              has_dynamic_offset : false, 
              min_binding_size : None 
            },
            count : None
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
            resource : wgpu::BindingResource::Buffer
            ( 
              wgpu::BufferBinding
              {
                buffer : &buffer,
                offset : 0,
                size : None
              }
            )
          }
        ]
      }
    );

    Self
    {
      camera,
      bind_group,
      bind_group_layout,
      buffer,
      timer
    }
  }  

  pub fn update( &mut self, queue : &wgpu::Queue )
  {
    self.timer.update();
    self.camera.update();

    queue.write_buffer
    (
      &self.buffer, 
      0, 
      bytemuck::cast_slice( &[ self.as_raw() ] )
    );
  }  

  pub fn as_raw( &self ) -> UniformRaw
  {
    let view = self.camera.get_view();
    let projection = self.camera.get_projection();
    let elapsed_time = self.timer.elapsed_time();

    UniformRaw
    {
      view_matrix : view.to_cols_array(),
      inverse_view_matrix : view.inverse().to_cols_array(),
      projection_matrix : projection.to_cols_array(),
      inverse_projection_matrix : projection.inverse().to_cols_array(),
      time : elapsed_time,
      ..Default::default()
    }
  }
}

pub struct Camera
{
  eye : glam::Vec3,
  up : glam::Vec3,
  view_dir : glam::Vec3,
  projection_matrix : glam::Mat4
}

impl Camera 
{
  pub fn new( fov : f32, aspect : f32, z_near : f32, z_far : f32 ) -> Self
  {
    let eye = glam::Vec3::ZERO;
    let up = glam::Vec3::Y;
    let view_dir = glam::Vec3::X;

    let projection_matrix = glam::Mat4::perspective_rh( fov, aspect, z_near, z_far );

    Self
    {
      eye,
      up,
      view_dir,
      projection_matrix
    }
  }

  pub fn get_view( &self ) -> glam::Mat4
  {
    glam::Mat4::look_to_rh( self.eye, self.view_dir, self.up )
  }

  pub fn get_projection( &self ) -> glam::Mat4
  {
    self.projection_matrix
  }

  pub fn update( &mut self )
  {
    self.view_dir = glam::Mat3::from_axis_angle( glam::Vec3::Y, 0.001 ) * self.view_dir;
    self.view_dir = self.view_dir.normalize();
    // self.view_dir = glam::Mat3::from_axis_angle( glam::Vec3::Z, 0.001 ) * self.view_dir;
  }
}

pub struct Timer
{
  now : std::time::Instant,
  elapsed_time : f32
}

impl Timer 
{
  pub fn new() -> Self
  {
    let now = std::time::Instant::now();
    let elapsed_time = 0.0;

    Self
    {
      now,
      elapsed_time
    }
  }

  pub fn update( &mut self )
  {
    self.elapsed_time = self.now.elapsed().as_secs_f32();
  }

  pub fn elapsed_time( &self ) -> f32
  {
    self.elapsed_time
  }
}