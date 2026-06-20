package org.polaris2023.msp_rigid_body.util;

public final class VoxelGrid implements AutoCloseable {
    private final int sizeX;
    private final int sizeY;
    private final int sizeZ;
    private final NativeMemory voxels;

    public VoxelGrid(int sizeX, int sizeY, int sizeZ) {
        if (sizeX <= 0 || sizeY <= 0 || sizeZ <= 0) {
            throw new IllegalArgumentException("voxel dimensions must be positive");
        }
        long count = (long) sizeX * sizeY * sizeZ;
        if (count > Integer.MAX_VALUE) {
            throw new IllegalArgumentException("voxel grid is too large for Java helper");
        }
        this.sizeX = sizeX;
        this.sizeY = sizeY;
        this.sizeZ = sizeZ;
        this.voxels = new NativeMemory(count);
    }

    public int sizeX() {
        return sizeX;
    }

    public int sizeY() {
        return sizeY;
    }

    public int sizeZ() {
        return sizeZ;
    }

    public long address() {
        return voxels.address();
    }

    public int count() {
        return Math.multiplyExact(Math.multiplyExact(sizeX, sizeY), sizeZ);
    }

    public byte[] toByteArray() {
        byte[] values = new byte[count()];
        long base = voxels.address();
        for (int i = 0; i < values.length; i++) {
            values[i] = NativeMemory.UNSAFE.getByte(base + i);
        }
        return values;
    }

    public boolean get(int x, int y, int z) {
        if (!contains(x, y, z)) {
            throw new IndexOutOfBoundsException("voxel coordinate is outside grid");
        }
        return NativeMemory.UNSAFE.getByte(voxels.address() + index(x, y, z)) != 0;
    }

    public int solidCount() {
        int solids = 0;
        long base = voxels.address();
        int count = count();
        for (int i = 0; i < count; i++) {
            if (NativeMemory.UNSAFE.getByte(base + i) != 0) {
                solids++;
            }
        }
        return solids;
    }

    public VoxelGrid clear() {
        NativeMemory.UNSAFE.setMemory(voxels.address(), count(), (byte) 0);
        return this;
    }

    public VoxelGrid set(int x, int y, int z, boolean solid) {
        if (!contains(x, y, z)) {
            throw new IndexOutOfBoundsException("voxel coordinate is outside grid");
        }
        voxels.putByte(index(x, y, z), solid ? 1 : 0);
        return this;
    }

    public VoxelGrid fillBox(int minX, int minY, int minZ, int maxX, int maxY, int maxZ) {
        int fromX = Math.max(0, minX);
        int fromY = Math.max(0, minY);
        int fromZ = Math.max(0, minZ);
        int toX = Math.min(sizeX, maxX);
        int toY = Math.min(sizeY, maxY);
        int toZ = Math.min(sizeZ, maxZ);
        for (int z = fromZ; z < toZ; z++) {
            for (int y = fromY; y < toY; y++) {
                for (int x = fromX; x < toX; x++) {
                    voxels.putByte(index(x, y, z), 1);
                }
            }
        }
        return this;
    }

    public VoxelGrid fillAabb(
            double minX, double minY, double minZ,
            double maxX, double maxY, double maxZ,
            double voxelSize,
            double originX, double originY, double originZ) {
        if (!Double.isFinite(voxelSize) || voxelSize <= 0.0) {
            throw new IllegalArgumentException("voxelSize must be positive and finite");
        }
        if (!finite(minX, minY, minZ, maxX, maxY, maxZ, originX, originY, originZ)) {
            throw new IllegalArgumentException("AABB and origin values must be finite");
        }
        if (minX > maxX || minY > maxY || minZ > maxZ) {
            throw new IllegalArgumentException("AABB min values must be <= max values");
        }

        int fromX = clampFloor((minX - originX) / voxelSize, sizeX);
        int fromY = clampFloor((minY - originY) / voxelSize, sizeY);
        int fromZ = clampFloor((minZ - originZ) / voxelSize, sizeZ);
        int toX = clampCeil((maxX - originX) / voxelSize, sizeX);
        int toY = clampCeil((maxY - originY) / voxelSize, sizeY);
        int toZ = clampCeil((maxZ - originZ) / voxelSize, sizeZ);
        return fillBox(fromX, fromY, fromZ, toX, toY, toZ);
    }

    public VoxelGrid fillAabb(
            double minX, double minY, double minZ,
            double maxX, double maxY, double maxZ,
            double voxelSize) {
        return fillAabb(minX, minY, minZ, maxX, maxY, maxZ, voxelSize, 0.0, 0.0, 0.0);
    }

    public VoxelGrid fillSphere(
            double centerX, double centerY, double centerZ,
            double radius,
            double voxelSize,
            double originX, double originY, double originZ) {
        if (!Double.isFinite(radius) || radius < 0.0 || !Double.isFinite(voxelSize) || voxelSize <= 0.0) {
            throw new IllegalArgumentException("radius must be non-negative and voxelSize must be positive");
        }
        if (!finite(centerX, centerY, centerZ, originX, originY, originZ)) {
            throw new IllegalArgumentException("sphere and origin values must be finite");
        }

        double radiusSquared = radius * radius;
        int fromX = clampFloor((centerX - radius - originX) / voxelSize, sizeX);
        int fromY = clampFloor((centerY - radius - originY) / voxelSize, sizeY);
        int fromZ = clampFloor((centerZ - radius - originZ) / voxelSize, sizeZ);
        int toX = clampCeil((centerX + radius - originX) / voxelSize, sizeX);
        int toY = clampCeil((centerY + radius - originY) / voxelSize, sizeY);
        int toZ = clampCeil((centerZ + radius - originZ) / voxelSize, sizeZ);
        for (int z = fromZ; z < toZ; z++) {
            double dz = originZ + (z + 0.5) * voxelSize - centerZ;
            for (int y = fromY; y < toY; y++) {
                double dy = originY + (y + 0.5) * voxelSize - centerY;
                for (int x = fromX; x < toX; x++) {
                    double dx = originX + (x + 0.5) * voxelSize - centerX;
                    if (dx * dx + dy * dy + dz * dz <= radiusSquared) {
                        voxels.putByte(index(x, y, z), 1);
                    }
                }
            }
        }
        return this;
    }

    public VoxelGrid fillSphere(double centerX, double centerY, double centerZ, double radius, double voxelSize) {
        return fillSphere(centerX, centerY, centerZ, radius, voxelSize, 0.0, 0.0, 0.0);
    }

    public VoxelGrid copyFrom(VoxelGrid other) {
        requireSameSize(other);
        NativeMemory.UNSAFE.copyMemory(other.voxels.address(), voxels.address(), count());
        return this;
    }

    public VoxelGrid union(VoxelGrid other) {
        requireSameSize(other);
        long dst = voxels.address();
        long src = other.voxels.address();
        int count = count();
        for (int i = 0; i < count; i++) {
            if (NativeMemory.UNSAFE.getByte(src + i) != 0) {
                NativeMemory.UNSAFE.putByte(dst + i, (byte) 1);
            }
        }
        return this;
    }

    public VoxelGrid subtract(VoxelGrid other) {
        requireSameSize(other);
        long dst = voxels.address();
        long src = other.voxels.address();
        int count = count();
        for (int i = 0; i < count; i++) {
            if (NativeMemory.UNSAFE.getByte(src + i) != 0) {
                NativeMemory.UNSAFE.putByte(dst + i, (byte) 0);
            }
        }
        return this;
    }

    public VoxelGrid intersect(VoxelGrid other) {
        requireSameSize(other);
        long dst = voxels.address();
        long src = other.voxels.address();
        int count = count();
        for (int i = 0; i < count; i++) {
            if (NativeMemory.UNSAFE.getByte(src + i) == 0) {
                NativeMemory.UNSAFE.putByte(dst + i, (byte) 0);
            }
        }
        return this;
    }

    @Override
    public void close() {
        voxels.close();
    }

    private boolean contains(int x, int y, int z) {
        return x >= 0 && y >= 0 && z >= 0 && x < sizeX && y < sizeY && z < sizeZ;
    }

    private long index(int x, int y, int z) {
        return (long) z * sizeX * sizeY + (long) y * sizeX + x;
    }

    private void requireSameSize(VoxelGrid other) {
        if (other == null || other.sizeX != sizeX || other.sizeY != sizeY || other.sizeZ != sizeZ) {
            throw new IllegalArgumentException("voxel grids must have the same dimensions");
        }
    }

    private static boolean finite(double... values) {
        for (double value : values) {
            if (!Double.isFinite(value)) {
                return false;
            }
        }
        return true;
    }

    private static int clampFloor(double value, int size) {
        return Math.max(0, Math.min(size, (int) Math.floor(value)));
    }

    private static int clampCeil(double value, int size) {
        return Math.max(0, Math.min(size, (int) Math.ceil(value)));
    }
}
