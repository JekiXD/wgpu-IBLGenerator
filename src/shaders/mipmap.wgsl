

struct VertexOutput
{
  @builtin( position ) pos : vec4f,
  @location( 0 ) uv : vec2f,
}

@vertex
fn vertex_main( @builtin( vertex_index ) id : u32 ) -> VertexOutput
{
  let x = f32( id / 2 );
  let y = f32( id % 2 );

  var result : VertexOutput;
  result.pos = vec4f( vec2f( x * 4.0 - 1.0, 1.0 - y * 4.0 ), 1.0, 1.0 );
  result.uv = vec2f( x, y ) * 2.0;

  return result;
}

@group( 0 ) @binding( 0 ) var t : texture_2d< f32 >;
@group( 0 ) @binding( 1 ) var s : sampler;


@fragment
fn fragment_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  return textureSample( t, s, in.uv );
}