import math, struct
from fractions import Fraction as Fr

def _r(x):
    return struct.unpack('f', struct.pack('f', x))[0] if math.isfinite(x) else x
class F32(float):
    # every op computed in f64 then rounded once to f32: correctly rounded RTE for + - * /
    def __new__(cls, v): return float.__new__(cls, _r(float(v)))
    def __add__(a,b): return F32(float(a)+float(b))
    def __radd__(a,b): return F32(float(b)+float(a))
    def __sub__(a,b): return F32(float(a)-float(b))
    def __rsub__(a,b): return F32(float(b)-float(a))
    def __mul__(a,b): return F32(float(a)*float(b))
    def __rmul__(a,b): return F32(float(b)*float(a))
    def __truediv__(a,b): return F32(float(a)/float(b))
    def __rtruediv__(a,b): return F32(float(b)/float(a))
    def __neg__(a): return F32(-float(a))
STEPS = 48

def dda_float(o, d, dt, recip=False, steps=STEPS):
    # mirrors render_ray_walk.rs: incremental t_max += t_delta, first-min axis on ties
    one = dt(1.0)
    cell = [int(math.floor(float(o[a]))) for a in range(3)]
    tmax = [dt(math.inf)]*3; tdel = [dt(math.inf)]*3; step=[0]*3
    for a in range(3):
        if d[a] > 0:
            step[a]=1
            num = dt(cell[a]+1) - o[a]
            tmax[a] = num * (one/d[a]) if recip else num / d[a]
            tdel[a] = one/d[a]
        elif d[a] < 0:
            step[a]=-1
            num = dt(cell[a]) - o[a]
            tmax[a] = num * (one/d[a]) if recip else num / d[a]
            tdel[a] = -(one/d[a])
    seq=[]
    for _ in range(steps):
        a = min(range(3), key=lambda k: tmax[k])
        cell[a]+=step[a]; tmax[a] = tmax[a] + tdel[a]
        seq.append(a)
    return seq

def dda_exact(o, d, steps=STEPS):
    # exact rational traversal of the same inputs; ties broken x<y<z like min_by
    o=[Fr(float(x)) for x in o]; d=[Fr(float(x)) for x in d]
    cell=[math.floor(x) for x in o]; step=[0]*3; tmax=[None]*3
    for a in range(3):
        if d[a]>0: step[a]=1; tmax[a]=(cell[a]+1-o[a])/d[a]
        elif d[a]<0: step[a]=-1; tmax[a]=(cell[a]-o[a])/d[a]
    seq=[]; ties=0
    for _ in range(steps):
        cands=[a for a in range(3) if tmax[a] is not None]
        m=min(tmax[a] for a in cands)
        best=[a for a in cands if tmax[a]==m]
        if len(best)>1: ties+=1
        a=best[0]; cell[a]+=step[a]; tmax[a]+=abs(1/d[a]); seq.append(a)
    return seq, ties

def light_dir(az, el):
    az=math.radians(az); el=math.radians(el)
    return (math.sin(az)*math.cos(el), math.sin(el), math.cos(az)*math.cos(el))

def corner_rays(n, inset=1e-3, bias=1e-4):
    # top faces (+Y) of cells (x, 0, z): corners inset along x and z, pushed up by bias
    for x in range(n):
        for z in range(n):
            for cu in (0,1):
                for cv in (0,1):
                    ox = x + (inset if cu==0 else 1-inset)
                    oz = z + (inset if cv==0 else 1-inset)
                    yield (ox, 1.0+bias, oz)

def run(az, el, n=24, steps=STEPS):
    d64 = light_dir(az, el)
    d32 = tuple(F32(c) for c in d64)
    tot=div64=div32=div32r=tiey=0; first=[]
    for o in corner_rays(n):
        o64=tuple(float(c) for c in o); o32=tuple(F32(c) for c in o)
        ex,ties = dda_exact(o64, d64, steps)
        s64 = dda_float(o64, d64, float, steps=steps)
        s32 = dda_float(o32, d32, F32, steps=steps)
        s32r= dda_float(o32, d32, F32, recip=True, steps=steps)
        tot+=1; tiey+= ties>0
        div64 += s64!=ex; div32 += s32!=s64; div32r += s32r!=s64
    return tot, tiey, div64, div32, div32r

print("az el | rays | rays_with_exact_tie | f64!=exact | f32div!=f64 | f32rcp!=f64   (first %d steps)"%STEPS)
for az,el in [(45,30),(45,45),(-30,30),(30,30),(120,10),(45,35.26438968)]:
    print(az, el, '|', *run(az,el))
