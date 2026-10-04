/** Two numbers along a plane's u and v axes. */
export type Vec2 = [number, number];

/** Three numbers along x, y, and z. */
export type Vec3 = [number, number, number];

/** One of the three axes. */
export type Axis = "x" | "y" | "z";

/** The axes a 3D mirror reflects across. */
export type Axes = "x" | "y" | "z" | "xy" | "xz" | "yz" | "xyz";

/** The direction one end of an axis faces. */
export type Side = "+x" | "-x" | "+y" | "-y" | "+z" | "-z";
