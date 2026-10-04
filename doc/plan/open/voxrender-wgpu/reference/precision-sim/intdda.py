import math, random
from fractions import Fraction as Fr
from tie import light_dir

F=13; DB=16          # origin fraction bits, direction magnitude bits
ONE=1<<F
def wrap32(x):        # two's-complement i32 wrap, as WGSL/SPIR-V integer ops do
    x&=0xffffffff
    return x-(1<<32) if x>=1<<31 else x

def quant_dir(d):
    m=max(abs(c) for c in d)
    return tuple(int(round(c/m*((1<<DB)-1))) for c in d)   # CPU-side, shared by reference and GPU

def int_walk(O, D, steps, bounds=None):
    # O: fixed-point origin (ints, units 2^-F), D: integer direction. Mirror to D>=0 per axis.
    sgn=[1 if c>0 else (-1 if c<0 else 0) for c in D]
    A=[abs(c) for c in D]
    # cell containing O; an origin exactly on a boundary belongs to the cell the ray moves into
    cell=[(O[a]>>F) if (sgn[a]>=0 or (O[a]&(ONE-1))) else (O[a]>>F)-1 for a in range(3)]
    # distance to next boundary in units 2^-F, along the direction of travel (in (0, ONE])
    rem=[0]*3
    for a in range(3):
        if sgn[a]>0: rem[a]=(cell[a]+1)*ONE-O[a]
        elif sgn[a]<0: rem[a]=O[a]-cell[a]*ONE
    act=[a for a in range(3) if A[a]>0]
    # pairwise error terms E[a][b] = rem_a*A_b - rem_b*A_a  (sign of t_a - t_b), bounded by ONE*max(A)
    E={(a,b): wrap32(rem[a]*A[b]-rem[b]*A[a]) for a in act for b in act if a<b}
    peak=max([abs(v) for v in E.values()]+[0])
    seq=[]
    for _ in range(steps):
        # axis with least t; ties go to the lower axis (x<y<z), as the reference's min_by does
        best=act[0]
        for a in act[1:]:
            e=E[(best,a)] if best<a else -E[(a,best)]
            if e>0 or (e==0 and a<best): best=a
        cell[best]+=sgn[best]; seq.append(best)
        for (a,b) in E:
            if a==best: E[(a,b)]=wrap32(E[(a,b)]+ONE*A[b])
            elif b==best: E[(a,b)]=wrap32(E[(a,b)]-ONE*A[a])
        peak=max([peak]+[abs(v) for v in E.values()])
    return seq, peak

def exact_walk(O, D, steps):
    o=[Fr(x,ONE) for x in O]; d=[Fr(x) for x in D]
    cell=[]
    for a in range(3):
        c=math.floor(o[a])
        if d[a]<0 and o[a]==c: c-=1
        cell.append(c)
    tmax=[None]*3
    for a in range(3):
        if d[a]>0: tmax[a]=(cell[a]+1-o[a])/d[a]
        elif d[a]<0: tmax[a]=(cell[a]-o[a])/d[a]
    seq=[]; ties=0
    for _ in range(steps):
        c=[a for a in range(3) if tmax[a] is not None]; m=min(tmax[a] for a in c)
        b=[a for a in c if tmax[a]==m]; ties+=len(b)>1
        a=b[0]; seq.append(a); tmax[a]+=abs(1/d[a])
    return seq, ties

mismatch=0; ties=0; n=0; peak=0
inset=1<<(F-10)   # 2^-10 voxel, exact in fixed point
for az,el in [(45,30),(45,45),(-30,30),(0,90),(90,0),(45,35.26438968)]:
    D=quant_dir(light_dir(az,el))
    for x in range(16):
        for z in range(16):
            for cu in (0,1):
                for cv in (0,1):
                    O=(x*ONE+(inset if cu==0 else ONE-inset), 1*ONE, z*ONE+(inset if cv==0 else ONE-inset))
                    s,p=int_walk(O,D,64); e,t=exact_walk(O,D,64)
                    n+=1; mismatch+=s!=e; ties+=t>0; peak=max(peak,p)
print(f"rays={n} integer-vs-exact mismatches={mismatch} rays containing exact ties={ties} peak|E|=2^{math.log2(peak):.2f} (bound 2^{F+DB})")
