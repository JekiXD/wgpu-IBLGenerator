struct Uniforms
{
  view_matrix : mat4x4< f32 >,
  inverse_view_matrix : mat4x4< f32 >,
  projection_matrix : mat4x4< f32 >,
  inverse_projection_matrix : mat4x4< f32 >,
  time : f32
};

@group( 0 ) @binding( 0 ) var< uniform > uniforms : Uniforms;
@group( 1 ) @binding( 0 ) var env_map : texture_cube< f32 >;
@group( 1 ) @binding( 1 ) var env_sampler : sampler;


struct VertexOutput
{
  @builtin( position ) pos : vec4f,
  @location( 0 ) uv : vec2f,
  @location( 1 ) dir : vec3f
}

@vertex
fn vertex_main( @builtin( vertex_index ) id : u32 ) -> VertexOutput
{
  let x = f32( id % 2 );
  let y = f32( id / 2 );

  var result : VertexOutput;
  result.pos = vec4f( vec2f( x * 4.0 - 1.0, 1.0 - y * 4.0 ), 1.0, 1.0 );
  result.uv = vec2f( x, y ) * 2.0;

  let unprojected_pos = uniforms.inverse_projection_matrix * result.pos;
  let inv_view_mat = transpose( mat3x3< f32 >( 
    uniforms.view_matrix[ 0 ].xyz, 
    uniforms.view_matrix[ 1 ].xyz, 
    uniforms.view_matrix[ 2 ].xyz 
  ));
  result.dir = inv_view_mat * unprojected_pos.xyz;
 // result.dir = normalize( result.dir );
  return result;
}

struct FragmentOutput
{
  @location( 0 ) frag_color : vec4f
}

@fragment
fn fragment_main( in : VertexOutput ) -> FragmentOutput
{
  let sample = textureSample( env_map, env_sampler, normalize( in.dir ) );
  let color = aces_tone_map( sample.rgb );

  var result : FragmentOutput;
  result.frag_color = vec4f( color, 1.0 );
  return result;
}

fn aces_tone_map( hdr: vec3< f32 > ) -> vec3< f32 > 
{
  let m1 = mat3x3
  (
    0.59719, 0.07600, 0.02840,
    0.35458, 0.90834, 0.13383,
    0.04823, 0.01566, 0.83777,
  );
  let m2 = mat3x3
  (
    1.60475, -0.10208, -0.00327,
    -0.53108,  1.10813, -0.07276,
    -0.07367, -0.00605,  1.07602,
  );
  let v = m1 * hdr;
  let a = v * ( v + 0.0245786 ) - 0.000090537;
  let b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;
  return clamp( m2 * ( a / b ), vec3( 0.0 ), vec3( 1.0 ) );
}