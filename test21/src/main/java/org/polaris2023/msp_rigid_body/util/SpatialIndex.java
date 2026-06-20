package org.polaris2023.msp_rigid_body.util;

import org.polaris2023.msp_rigid_body.RigidBodyNative;

import java.util.Arrays;

public final class SpatialIndex implements AutoCloseable {
    private final boolean compact;
    private long handle;

    private SpatialIndex(boolean compact) {
        this.compact = compact;
        this.handle = compact ? RigidBodyNative.crbTreeCreate() : RigidBodyNative.rtreeCreate();
        if (handle == 0L) {
            throw new IllegalStateException("spatial index create failed");
        }
    }

    public static SpatialIndex rtree() {
        return new SpatialIndex(false);
    }

    public static SpatialIndex compactTree() {
        return new SpatialIndex(true);
    }

    public int size() {
        requireOpen();
        return compact ? RigidBodyNative.crbTreeLen(handle) : RigidBodyNative.rtreeLen(handle);
    }

    public SpatialIndex clear() {
        requireOpen();
        if (compact) {
            RigidBodyNative.crbTreeClear(handle);
        } else {
            RigidBodyNative.rtreeClear(handle);
        }
        return this;
    }

    public boolean insert(long id, double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
        requireOpen();
        return compact
                ? RigidBodyNative.crbTreeInsert(handle, id, minX, minY, minZ, maxX, maxY, maxZ)
                : RigidBodyNative.rtreeInsert(handle, id, minX, minY, minZ, maxX, maxY, maxZ);
    }

    public boolean update(long id, double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
        requireOpen();
        return compact
                ? RigidBodyNative.crbTreeUpdate(handle, id, minX, minY, minZ, maxX, maxY, maxZ)
                : RigidBodyNative.rtreeUpdate(handle, id, minX, minY, minZ, maxX, maxY, maxZ);
    }

    public boolean remove(long id) {
        requireOpen();
        return compact ? RigidBodyNative.crbTreeRemove(handle, id) : RigidBodyNative.rtreeRemove(handle, id);
    }

    public int countAabb(double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
        requireOpen();
        return compact
                ? RigidBodyNative.crbTreeQueryAabbCount(handle, minX, minY, minZ, maxX, maxY, maxZ)
                : RigidBodyNative.rtreeQueryAabbCount(handle, minX, minY, minZ, maxX, maxY, maxZ);
    }

    public long[] queryAabb(double minX, double minY, double minZ, double maxX, double maxY, double maxZ, int capacity) {
        requireOpen();
        if (capacity <= 0) {
            return new long[0];
        }
        try (NativeMemory out = NativeMemory.longs(capacity)) {
            int written = compact
                    ? RigidBodyNative.crbTreeQueryAabb(handle, minX, minY, minZ, maxX, maxY, maxZ, out.address(), capacity)
                    : RigidBodyNative.rtreeQueryAabb(handle, minX, minY, minZ, maxX, maxY, maxZ, out.address(), capacity);
            return Arrays.copyOf(out.getLongs(capacity), Math.max(0, Math.min(written, capacity)));
        }
    }

    @Override
    public void close() {
        if (handle != 0L) {
            if (compact) {
                RigidBodyNative.crbTreeDestroy(handle);
            } else {
                RigidBodyNative.rtreeDestroy(handle);
            }
            handle = 0L;
        }
    }

    private void requireOpen() {
        if (handle == 0L) {
            throw new IllegalStateException("spatial index is closed");
        }
    }
}
