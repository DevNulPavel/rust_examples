#if defined BUILD_IOS
#define PLATFORM_PRECISION highp
#elif defined BUILD_ANDROID
#define PLATFORM_PRECISION highp
#else
#define PLATFORM_PRECISION
#endif

attribute PLATFORM_PRECISION vec4 a_position;
attribute vec3 a_normal;
attribute PLATFORM_PRECISION vec2 a_texcoord;

varying PLATFORM_PRECISION vec2 v_texcoord;
varying PLATFORM_PRECISION vec2 v_texcoord1;
varying vec2 v_texcoord2;
varying vec3 v_normal;

uniform PLATFORM_PRECISION mat4 u_transform;

uniform PLATFORM_PRECISION float u_time;

void main(void)
{
    PLATFORM_PRECISION float xmax = 14173.4;
    PLATFORM_PRECISION float ymax = 9046.35351;
    
    v_texcoord2 = vec2((a_position.x) / xmax, 1.0 - (a_position.z) / ymax);
    
    PLATFORM_PRECISION float v_opc = a_position.z / ymax;
    
    PLATFORM_PRECISION float p = pow((1.0 - v_opc), 2.0);
    
    v_texcoord = 0.6 * (1.0 + p) * a_texcoord;
    
    v_texcoord1 = 0.45 * v_texcoord + vec2(12.0, 15.0) * u_time;
    v_texcoord += 6.0 * u_time;
    
    v_normal = 6.0 * a_normal;
    gl_Position = u_transform * a_position;
}
