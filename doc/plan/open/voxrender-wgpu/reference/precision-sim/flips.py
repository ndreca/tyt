import math, random
from tie import F32, light_dir, corner_rays, dda_exact
from fractions import Fraction as Fr

def walk_cells(o, d, dt, recip=False, maxsteps=200, bounds=(24,10,24)):
    one = dt(1.0)
    cell = [int(math.floor(float(o[a]))) for a in range(3)]
    tmax=[dt(math.inf)]*3; tdel=[dt(math.inf)]*3; step=[0]*3
    for a in range(3):
        if d[a] > 0:
            step[a]=1; num=dt(cell[a]+1)-o[a]
        elif d[a] < 0:
            step[a]=-1; num=dt(cell[a])-o[a]
        else: continue
        tmax[a] = num*(one/d[a]) if recip else num/d[a]
        tdel[a] = one/d[a] if d[a]>0 else -(one/d[a])
    for _ in range(maxsteps):
        a=min(range(3), key=lambda k:tmax[k])
        cell[a]+=step[a]; tmax[a]=tmax[a]+tdel[a]
        if not all(0<=cell[k]<bounds[k] for k in range(3)): return
        yield tuple(cell)

def walk_exact(o, d, maxsteps=200, bounds=(24,10,24)):
    o=[Fr(float(x)) for x in o]; d=[Fr(float(x)) for x in d]
    cell=[math.floor(x) for x in o]; step=[0]*3; tmax=[None]*3
    for a in range(3):
        if d[a]>0: step[a]=1; tmax[a]=(cell[a]+1-o[a])/d[a]
        elif d[a]<0: step[a]=-1; tmax[a]=(cell[a]-o[a])/d[a]
    for _ in range(maxsteps):
        cands=[a for a in range(3) if tmax[a] is not None]
        m=min(tmax[a] for a in cands); a=[k for k in cands if tmax[k]==m][0]
        cell[a]+=step[a]; tmax[a]+=abs(1/d[a])
        if not all(0<=cell[k]<bounds[k] for k in range(3)): return
        yield tuple(cell)

def blocked(it, occ):
    return any(c in occ for c in it)

def scene_random(seed, density):
    rnd=random.Random(seed); occ=set()
    for x in range(24):
        for y in range(1,10):
            for z in range(24):
                if rnd.random()<density: occ.add((x,y,z))
    return occ

def scene_stairs():
    # diagonal staircases along x=z, common in voxel art: columns at (k, 1..h, k) and (k+1,.., k)
    occ=set()
    for k in range(24):
        for y in range(1,4):
            occ.add((k,y,(k+7)%24)); occ.add(((k+13)%24,y,k))
    return occ

def run(name, occ, az, el):
    d64=light_dir(az,el); d32=tuple(F32(c) for c in d64)
    n=f64x=f32x=f32r=0
    for o in corner_rays(24):
        if o[0] < 0: continue
        o64=tuple(o); o32=tuple(F32(c) for c in o)
        ex=blocked(walk_exact(o64,d64),occ)
        b64=blocked(walk_cells(o64,d64,float),occ)
        b32=blocked(walk_cells(o32,d32,F32),occ)
        b32r=blocked(walk_cells(o32,d32,F32,recip=True),occ)
        n+=1; f64x+=b64!=ex; f32x+=b32!=b64; f32r+=b32r!=b64
    print(f"{name:10s} az={az:>4} el={el:>5} corner rays={n} f64!=exact={f64x} f32div!=f64={f32x} f32rcp!=f64={f32r}")

for az,el in [(45,30),(45,45),(-30,30)]:
    run('random15', scene_random(1,0.15), az, el)
    run('random40', scene_random(2,0.40), az, el)
    run('stairs', scene_stairs(), az, el)
