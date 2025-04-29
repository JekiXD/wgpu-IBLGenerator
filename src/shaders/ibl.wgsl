

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

struct Uniform
{
  mip_level : u32,
  total_mips : u32
}

@group( 0 ) @binding( 0 ) var env_map : texture_cube< f32 >;
@group( 0 ) @binding( 1 ) var env_sampler : sampler;
@group( 0 ) @binding( 2 ) var< uniform > uniforms : Uniform ;


const PI : f32 = 3.1415926535;

@fragment
fn fragment_diffuse_main( in : VertexOutput ) -> @location( 0 ) vec4f
{ 
  var uv = vec2f( in.uv.x, 1.0 - in.uv.y ); 
  // -1.0..1.0
  uv = uv * 2.0 - vec2f( 1.0 );
  // vec2f( -PI..PI, -PI/2..PI/2 )
  uv *= vec2f( PI, PI / 2.0 );
  var normal = vec3f( cos( uv.x ) * cos( uv.y ), sin( uv.y ), sin( uv.x ) * cos( uv.y ) );
  normal = normalize( normal );

  var up = vec3f( 0.0, 1.0, 0.0 );
  // if( 1.0 - abs( normal.y ) < 1e-5 )
  // {
  //   up = vec3f( 1.0, 0.0, 0.0 );
  // }

  var right = normalize( cross( up, normal ) );
  up = normalize( cross( normal, right ) );

  let TBN = mat3x3< f32 >( up, normal, right );

  let NUM_SAMPLES_X : f32 = 50.0;
  let NUM_SAMPLES_Y : f32 = 50.0;
  let DELTA_X : f32 = 1.0 / NUM_SAMPLES_X;
  let DELTA_Y : f32 = 1.0 / NUM_SAMPLES_Y;

  var result = vec4f( 0.0 );

  for( var x = 0.0; x < 1.0; x += DELTA_X )
  {
    for( var y = 0.0; y < 1.0; y += DELTA_Y )
    {
      var uv = vec2f( x, y ) * vec2f( 2.0 * PI, PI / 2.0 );
      var sample_dir = normalize( vec3f( sin( uv.x ) * sin( uv.y ), cos( uv.y ), cos( uv.x ) * sin( uv.y ) ) );
      sample_dir = TBN * sample_dir;

      result += textureSample( env_map, env_sampler, sample_dir ) * cos( uv.y ) * sin( uv.y );
    }
  }

  result = PI * result / ( NUM_SAMPLES_X * NUM_SAMPLES_Y );

  return result;
}

@fragment
fn fragment_specular_1_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  var uv = vec2f( in.uv.x, 1.0 - in.uv.y ); 
  // -1.0..1.0
  uv = uv * 2.0 - vec2f( 1.0 );
  // vec2f( -PI..PI, -PI/2..PI/2 )
  uv *= vec2f( PI, PI / 2.0 );
  var N = vec3f( cos( uv.x ) * cos( uv.y ), sin( uv.y ), sin( uv.x ) * cos( uv.y ) );
  N = normalize( N );
  let V = N;

  let roughness = f32( uniforms.mip_level ) / f32( uniforms.total_mips );
  let alpha = roughness * roughness;
  let NUM_SAMPLES = u32( 512 );

  var result = vec3f( 0.0 );
  var total_weight = 0.0;

  let env_dim = f32( textureDimensions( env_map ).x );

  for( var i = 0u; i < NUM_SAMPLES; i += 1u )
  {
    let Xi = Hammersley( i, NUM_SAMPLES );
    let H = importance_sample( Xi, N, alpha );

    let dotVH = saturate( dot( V, H ) );
    let L = normalize( 2.0 * dotVH * H - V );

    let dotNL = saturate( dot( N, L ) );
    let dotNH = saturate( dot( N, H ) );

    if( dotNL > 0.0 )
    {
      let D = D_GGX( alpha, dotNH );
      let pdf = D * dotNH / ( 4.0 * dotVH );
      let saTexel = 4.0 * PI / ( 6.0 * env_dim * env_dim );
      let saSample = 1.0 / ( f32( NUM_SAMPLES ) * pdf );
      var mipLevel = 0.0;
      if( roughness != 0.0 )
      {
        mipLevel = 0.5 * log2( saSample / saTexel );
      }

      result += textureSampleLevel( env_map, env_sampler, L, mipLevel ).rgb * dotNL;
      total_weight += dotNL;
    }
  }

  result /= f32( NUM_SAMPLES );
  result /= total_weight;
  return vec4f( result, 1.0 );
}

@fragment
fn fragment_specular_2_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  let roughness = in.uv.y;
  let dotNV = in.uv.x;

  let alpha = roughness * roughness;
  let NUM_SAMPLES = u32( 1024 );

  let N = vec3f( 0.0, 1.0, 0.0 );
  let V = vec3f( 0.0, dotNV, sqrt( 1.0 - dotNV * dotNV ) );

  var result = vec2f( 0.0 );

  for( var i = 0u; i < NUM_SAMPLES; i += 1u )
  {
    let Xi = Hammersley( i, NUM_SAMPLES );
    let H = importance_sample( Xi, N, alpha );

    let dotVH = saturate( dot( V, H ) );
    let L = normalize( 2.0 * dotVH * H - V );

    let dotNL = saturate( L.y );
    let dotNH = saturate( H.y );

    if( dotNL > 0.0 )
    {
      let G = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
      //let G = GeometrySmith( dotNV, dotNL, alpha );
      let BRDF = G * dotVH / ( dotNH * dotNV );

      let Fp5 = pow( 1.0 - dotVH, 5.0 );
      result.x += BRDF * ( 1.0 - Fp5 );
      result.y += BRDF * Fp5;
    }
  }

  result = result / f32( NUM_SAMPLES );
  return vec4f( result, 0.0, 1.0 );
}


fn GeometrySchlickGGX( dotNV : f32, a : f32 ) -> f32
{
  let k = (a * a) / 2.0;

  let nom   = dotNV;
  let denom = dotNV * (1.0 - k) + k;

  return nom / denom;
}
// ----------------------------------------------------------------------------
fn GeometrySmith( dotNV : f32, dotNL : f32, alpha : f32) -> f32
{
  let ggx2 = GeometrySchlickGGX(dotNV, alpha);
  let ggx1 = GeometrySchlickGGX(dotNL, alpha);

  return ggx1 * ggx2;
} 

// https://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
fn RadicalInverse_VdC( bits_in : u32 ) -> f32
{
  var bits = bits_in;
  bits = (bits << 16u) | (bits >> 16u);
  bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
  bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
  bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
  bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
  return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

fn Hammersley( i : u32, N : u32 ) -> vec2f
{
  return vec2f( f32( i ) / f32( N ), RadicalInverse_VdC( i ) );
} 

fn pow2( x : f32 ) -> f32
{
  return x * x;
}

// Geometry function
fn V_GGX_SmithCorrelated( alpha : f32, dotNL : f32, dotNV : f32 ) -> f32
{
  let a2 = pow2( alpha );
  let gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
  let gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
  return 0.5 / max( gv + gl, 1e-6 );
}

// Normal distribution function
fn D_GGX( alpha : f32, dotNH : f32 ) -> f32
{
  let a2 = pow2( alpha );
  let denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
  return 0.3183098861837907 * a2 / pow2( denom );
}

fn importance_sample( Xi : vec2f, n : vec3f, alpha : f32 ) -> vec3f
{
  let phi = 2.0 * PI * Xi.x;
  let cosTheta = sqrt( ( 1.0 - Xi.y ) / ( 1.0 + ( alpha * alpha - 1.0 ) * Xi.y ) );
  let sinTheta = sqrt( 1.0 - cosTheta * cosTheta );

  var sampleDir : vec3f;
  sampleDir.z = cos( phi ) * sinTheta;
  sampleDir.x = sin( phi ) * sinTheta;
  sampleDir.y = cosTheta;

  var up = vec3f( 0.0, 1.0, 0.0 );
  if( abs( n.y ) > 0.999 )
  {
    up = vec3f( 1.0, 0.0, 0.0 );
  }
  let forward = normalize( cross( up, n ) );
  let right = normalize( cross( n, forward ) );
  let TBN = mat3x3< f32 >( right, n, forward );

  //return normalize( sampleDir );
  return normalize( TBN * sampleDir );
}