
@group( 0 ) @binding( 0 ) var dst : texture_storage_2d_array< rgba32float, write >;
@group( 0 ) @binding( 1 ) var src : texture_2d< f32 >;

struct Face 
{
  forward: vec3<f32>,
  up: vec3<f32>,
  right: vec3<f32>,
};

const tangent_normalizer : vec2f = vec2f( 0.15915, 0.3183 );


@compute @workgroup_size(16, 16, 1)
fn main( @builtin( global_invocation_id ) gid: vec3< u32 > )
{
  if gid.x >= u32( textureDimensions( dst ).x ) 
  {
    return;
  }

  // https://www.w3.org/TR/webgpu/#coordinate-systems
  // Wwebgpu uses left-handed coordinate system to represent the face of a cube
  // When transforming into the right-handed coordinate system, the Y is flipped
  var faces : array< Face, 6 > = array
  (
    // +X
    Face
    (
      vec3f( 1.0, 0.0, 0.0 ),
      vec3f( 0.0, 0.0, -1.0 ),
      vec3f( 0.0, 1.0, 0.0 )
    ),
    // -X
    Face
    (
      vec3f( -1.0, 0.0, 0.0 ),
      vec3f( 0.0, 0.0, 1.0 ),
      vec3f( 0.0, 1.0, 0.0 )
    ),
    // +Y
    Face
    (
      vec3f( 0.0, -1.0, 0.0 ),
      vec3f( 1.0, 0.0, 0.0 ),
      vec3f( 0.0, 0.0, 1.0 )
    ),
    // -Y
    Face
    (
      vec3f( 0.0, 1.0, 0.0 ),
      vec3f( 1.0, 0.0, 0.0 ),
      vec3f( 0.0, 0.0, -1.0 )
    ),
    // +Z
    Face
    (
      vec3f( 0.0, 0.0, 1.0 ),
      vec3f( 1.0, 0.0, 0.0 ),
      vec3f( 0.0, 1.0, 0.0 )
    ),
    // -Z
    Face
    (
      vec3f( 0.0, 0.0, -1.0 ),
      vec3f( -1.0, 0.0, 0.0 ),
      vec3f( 0.0, 1.0, 0.0 )
    )
  );

  let size = textureDimensions( dst );
  // Get the YZ coordinates from the invocation ID
  var face_uv = vec2f( gid.xy );
  // Normallize it in range 0.0..1.0
  face_uv /= vec2f( size );
  // Transform to range -1.0..1.0
  let uv2 = face_uv * 2.0 - vec2f( 1.0 );

  // The face of the cube to draw to
  let face = faces[ gid.z ];
  let rot_mat = mat3x3< f32 >( face.forward, face.up, face.right );

  // Get the direction vector
  var dir = vec3f( 1.0, uv2 );
  // Rotate it towards the current face of the cube
  dir = rot_mat * dir;
  dir = normalize( dir );

  // Get the spherical coordinates from the direction
  let longitude = asin( dir.y );
  let latitude = atan2( dir.z, dir.x );

  var hdr_uv = vec2f( latitude, longitude ) * tangent_normalizer + vec2f( 0.5 );
  //let hdr_uv2 = vec2u( hdr_uv * vec2f( textureDimensions( src ) ) );
  
  let hdr_sample = sampleHDR( src, hdr_uv );
  //textureStore( dst, gid.xy, gid.z, vec4f( vec3f( hdr_uv.x ), 1.0,) );
  textureStore(  dst, gid.xy, gid.z, hdr_sample );
}

fn sampleHDR( src : texture_2d< f32 >, uv : vec2f ) -> vec4f
{
  var dimensions = textureDimensions( src );
  let size = vec2f( dimensions );
  let big_uv = uv * size;
  let cell_id = vec2u( floor( big_uv ) );
  let offset = fract( big_uv );

  dimensions -= vec2u( 1 );

  let sample1 = textureLoad( src, cell_id + vec2u( 0, 0 ), 0 );
  let sample2 = textureLoad( src, min( cell_id + vec2u( 1, 0 ), dimensions ), 0 );
  let sample3 = textureLoad( src, min( cell_id + vec2u( 0, 1 ), dimensions ), 0 );
  let sample4 = textureLoad( src, min( cell_id + vec2u( 1, 1 ), dimensions ), 0 );

  let mix12 = mix( sample1, sample2, offset.x );
  let mix34 = mix( sample3, sample4, offset.x );

  return mix( mix12, mix34, offset.y );
}