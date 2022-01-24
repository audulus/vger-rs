
fn proj(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return normalize(a) * dot(a,b) / length(a);
}

fn orth(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return b - proj(a, b);
}

fn rot90(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(-p.y, p.x);
}

// From https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm
// See also https://www.shadertoy.com/view/4dfXDn

fn sdCircle(p: vec2<f32>, r: f32) -> f32
{
    return length(p) - r;
}

fn sdBox(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32
{
    let d = abs(p)-b+r;
    return length(max(d,vec2<f32>(0.0, 0.0))) + min(max(d.x,d.y),0.0)-r;
}

fn sdSegment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32
{
    let pa = p-a;
    let ba = b-a;
    let h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
    return length( pa - ba*h );
}

fn sdSegment2(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, width: f32) -> f32
{
    let u = normalize(b-a);
    let v = rot90(u);

    var pp = p;
    pp = pp - (a+b)/2.0;
    pp = pp * mat2x2<f32>(u, v);
    return sdBox(pp, vec2<f32>(length(b-a)/2.0, width/2.0), 0.0);
}

// sca is {sin,cos} of orientation
// scb is {sin,cos} of aperture angle
fn sdArc(p: vec2<f32>, sca: vec2<f32>, scb: vec2<f32>, ra: f32, rb: f32 ) -> f32
{
    var pp = p * mat2x2<f32>(vec2<f32>(sca.x,sca.y),vec2<f32>(-sca.y,sca.x));
    pp.x = abs(pp.x);
    var k = 0.0;
    if (scb.y*pp.x>scb.x*pp.y) {
        k = dot(pp,scb);
    } else {
        k = length(pp);
    }
    return sqrt( dot(pp,pp) + ra*ra - 2.0*ra*k ) - rb;
}

fn dot2(v: vec2<f32>) -> f32 {
    return dot(v,v);
}

fn sdBezier(pos: vec2<f32>, A: vec2<f32>, B: vec2<f32>, C: vec2<f32> ) -> f32
{
    let a = B - A;
    let b = A - 2.0*B + C;
    let c = a * 2.0;
    let d = A - pos;
    let kk = 1.0/dot(b,b);
    let kx = kk * dot(a,b);
    let ky = kk * (2.0*dot(a,a)+dot(d,b)) / 3.0;
    let kz = kk * dot(d,a);
    var res = 0.0;
    let p = ky - kx*kx;
    let p3 = p*p*p;
    let q = kx*(2.0*kx*kx + -3.0*ky) + kz;
    var h = q*q + 4.0*p3;
    if( h >= 0.0)
    {
        h = sqrt(h);
        let x = (vec2<f32>(h,-h)-q)/2.0;
        let uv = sign(x)*pow(abs(x), vec2<f32>(1.0/3.0));
        let t = clamp( uv.x+uv.y-kx, 0.0, 1.0 );
        res = dot2(d + (c + b*t)*t);
    }
    else
    {
        let z = sqrt(-p);
        let v = acos( q/(p*z*2.0) ) / 3.0;
        let m = cos(v);
        let n = sin(v)*1.732050808;
        let vv = vec3<f32>(m+m,-n-m,n-m)*z-kx;
        let t = vec3<f32>(clamp(vv.x, 0.0, 1.0), clamp(vv.y, 0.0, 1.0), clamp(vv.z, 0.0, 1.0));
        res = min( dot2(d+(c+b*t.x)*t.x),
                   dot2(d+(c+b*t.y)*t.y) );
        // the third root cannot be the closest
        // res = min(res,dot2(d+(c+b*t.z)*t.z));
    }
    return sqrt( res );
}

fn vs_main() { }