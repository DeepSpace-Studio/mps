package org.polaris2023.msp_rigid_body.util;

import org.polaris2023.msp_rigid_body.RigidBodyNative;

public final class Collider {
    private final PhysicsWorld world;
    private final long handle;

    Collider(PhysicsWorld world, long handle) {
        this.world = world;
        this.handle = handle;
    }

    public boolean isEmpty() {
        return handle == 0L;
    }

    public long handle() {
        return handle;
    }

    public PhysicsWorld world() {
        return world;
    }

    public double density() {
        return RigidBodyNative.colliderGetDensity(world.handle(), handle);
    }

    public static final class Builder implements AutoCloseable, IParent<PhysicsWorld> {
        private final PhysicsWorld parent;
        private long handle;

        private Builder(PhysicsWorld parent, long handle) {
            this.parent = parent;
            this.handle = handle;
        }

        public static Builder voxels(
                PhysicsWorld parent,
                long voxels, int sizeX, int sizeY, int sizeZ, double voxelSize,
                double originX, double originY, double originZ,
                int mode, boolean dynamicBody, int smallVoxelLimit, int meshVoxelLimit) {
            long handle = RigidBodyNative.colliderBuilderCreateVoxels(
                    voxels, sizeX, sizeY, sizeZ, voxelSize,
                    originX, originY, originZ,
                    mode, dynamicBody ? 1 : 0, smallVoxelLimit, meshVoxelLimit);
            return new Builder(parent, handle);
        }

        public boolean isEmpty() {
            return handle == 0L;
        }

        public Builder friction(double friction) {
            requireOpen();
            RigidBodyNative.colliderBuilderSetFriction(handle, friction);
            return this;
        }

        public Builder restitution(double restitution) {
            requireOpen();
            RigidBodyNative.colliderBuilderSetRestitution(handle, restitution);
            return this;
        }

        public Raw buildRaw() {
            requireOpen();
            long raw = RigidBodyNative.colliderBuilderBuild(handle);
            handle = 0L;
            return new Raw(raw);
        }

        public Collider insert() {
            try (Raw raw = buildRaw()) {
                return parent.insert(raw);
            }
        }

        @Override
        public void close() {
            if (handle != 0L) {
                RigidBodyNative.colliderBuilderDestroy(handle);
                handle = 0L;
            }
        }

        @Override
        public PhysicsWorld parent() {
            return parent;
        }

        private void requireOpen() {
            if (handle == 0L) {
                throw new IllegalStateException("collider builder is closed");
            }
        }
    }

    public static final class Raw implements AutoCloseable {
        private long handle;

        private Raw(long handle) {
            this.handle = handle;
        }

        public boolean isEmpty() {
            return handle == 0L;
        }

        long release() {
            long value = handle;
            handle = 0L;
            return value;
        }

        @Override
        public void close() {
            if (handle != 0L) {
                RigidBodyNative.colliderDestroyRaw(handle);
                handle = 0L;
            }
        }
    }
}
