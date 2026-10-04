import math, random
from tie import F32

B=(24,10,24)
def ref_walk(o, d, dt, occ):
    # mirrors PlacementWalk::new + advance from render_ray_walk.rs, returns (cell, axis) of first live cell
    tE=-math.inf; tX=math.inf; ea=0
    for a in range(3):
        if d[a]==0:
            if o[a]<0 or o[a]>=B[a]: return None
            continue
        t0=(dt(0.0)-o[a])/d[a]; t1=(dt(B[a])-o[a])/d[a]
        if min(t0,t1)>tE: tE=min(t0,t1); ea=a
        tX=min(tX,max(t0,t1))
    if tX<tE or tX<0: return None
    p=[o[a]+d[a]*max(tE,dt(0.0)) for a in range(3)]
    cell=[min(max(int(math.floor(float(p[a]))),0),B[a]-1) for a in range(3)]
    step=[0]*3; tmax=[dt(math.inf)]*3; tdel=[dt(math.inf)]*3
    one=dt(1.0)
    for a in range(3):
        if d[a]>0: step[a]=1; tmax[a]=(dt(cell[a]+1)-o[a])/d[a]; tdel[a]=one/d[a]
        elif d[a]<0: step[a]=-1; tmax[a]=(dt(cell[a])-o[a])/d[a]; tdel[a]=-one/d[a]
    if tuple(cell) in occ: return tuple(cell), ea
    while True:
        a=min(range(3), key=lambda k:tmax[k])
        cell[a]+=step[a]
        if not (0<=cell[a]<B[a]): return None
        tmax[a]=tmax[a]+tdel[a]
        if tuple(cell) in occ: return tuple(cell), a

def scene(seed, density):
    rnd=random.Random(seed); occ=set()
    for x in range(B[0]):
        for y in range(B[1]):
            for z in range(B[2]):
                if rnd.random()<density: occ.add((x,y,z))
    return occ

def cross(a,b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def run(az, el, center, res=128, dist=40.0, fov=30.0, ortho=None):
    occ=scene(3,0.08)
    a=math.radians(az); e=math.radians(el)
    back=(math.sin(a)*math.cos(e), math.sin(e), math.cos(a)*math.cos(e))
    eye=tuple(center[k]+dist*back[k] for k in range(3)); fwd=tuple(-b for b in back)
    right=cross(fwd,(0.0,1.0,0.0)); rn=math.sqrt(sum(r*r for r in right)); right=tuple(r/rn for r in right)
    up=cross(right,fwd)
    h=math.tan(math.radians(fov)/2)
    f=lambda v: tuple(F32(x) for x in v)
    fw32,r32,u32,eye32=f(fwd),f(right),f(up),f(eye)
    n=dc=df=0
    for j in range(res):
        for i in range(res):
            sx=2*(i+0.5)/res-1; sy=1-2*(j+0.5)/res
            sx32=F32(2.0)*(F32(i)+F32(0.5))/F32(res)-F32(1.0); sy32=F32(1.0)-F32(2.0)*(F32(j)+F32(0.5))/F32(res)
            if ortho:
                o=tuple(eye[k]+sx*ortho*right[k]+sy*ortho*up[k] for k in range(3)); d=fwd
                o32=tuple(eye32[k]+sx32*F32(ortho)*r32[k]+sy32*F32(ortho)*u32[k] for k in range(3)); d32=fw32
            else:
                d=[fwd[k]+sx*h*right[k]+sy*h*up[k] for k in range(3)]; dn=math.sqrt(sum(x*x for x in d)); d=tuple(x/dn for x in d)
                d32=[fw32[k]+sx32*F32(h)*r32[k]+sy32*F32(h)*u32[k] for k in range(3)]
                dn32=F32(math.sqrt(float(sum(x*x for x in d32)))); d32=tuple(x/dn32 for x in d32)
                o=eye; o32=eye32
            h64=ref_walk(o,d,float,occ); h32=ref_walk(o32,d32,F32,occ)
            n+=1
            if (h64 is None)!=(h32 is None) or (h64 and h32 and h64[0]!=h32[0]): dc+=1
            elif h64 and h32 and h64[1]!=h32[1]: df+=1
    print(f"{'ortho' if ortho else 'persp'} az={az:>4} el={el:>3} center={center} pixels={n} cell-differs={dc} face-differs={df}")

for az,el in [(45,30),(-30,30),(0,0),(0,90)]:
    run(az,el,(12.0,5.0,12.0))
    run(az,el,(12.0,5.0,12.0),ortho=16.0)
run(0,0,(12.3,5.1,12.7))
run(0,0,(12.3,5.1,12.7),ortho=16.0)
