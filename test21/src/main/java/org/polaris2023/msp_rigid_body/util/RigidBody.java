package org.polaris2023.msp_rigid_body.util;

import org.polaris2023.msp_rigid_body.RigidBodyNative;

public final class RigidBody {
    private long body;

    public double[] translation(PhysicsWorld world) {
        if (body == 0L) {
            throw new IllegalStateException("rigid body is empty");
        }
        return RigidBodyNative.rigidBodyGetTranslation(world.handle(), body);
    }

    public double translationX(PhysicsWorld world) {
        return translation(world)[0];
    }

    public double translationY(PhysicsWorld world) {
        return translation(world)[1];
    }

    public double translationZ(PhysicsWorld world) {
        return translation(world)[2];
    }

    public static final class Builder implements AutoCloseable, IParent<PhysicsWorld> {
        private int status;
        private long handle;
        private final PhysicsWorld parent;

        private Builder(PhysicsWorld parent) {
            this.parent = parent;
        }

        public static Builder builder(PhysicsWorld parent) {
            return new Builder(parent);
        }

        public Builder status(int status) {
            this.status = status;
            return this;
        }

        public Builder build() {
            handle = RigidBodyNative.rigidBodyBuilderCreate(status);
            return this;
        }

        public Builder translation(double x, double y, double z) {
            requireOpen();
            RigidBodyNative.rigidBodyBuilderSetTranslation(handle, x, y, z);
            return this;
        }

        public RigidBody body(PhysicsWorld world) {
            requireOpen();
            RigidBody value = new RigidBody();
            long rigidBody = RigidBodyNative.rigidBodyBuilderBuild(handle);
            handle = 0L;
            if (rigidBody == 0L) {
                return value;
            }
            value.body = RigidBodyNative.worldInsertRigidBody(world.handle(), rigidBody);
            return value;
        }

        public boolean isEmpty() {
            return handle == 0L;
        }

        @Override
        public void close() {
            if (handle != 0L) {
                RigidBodyNative.rigidBodyBuilderDestroy(handle);
                handle = 0L;
            }
        }

        @Override
        public PhysicsWorld parent() {
            return parent;
        }

        private void requireOpen() {
            if (handle == 0L) {
                throw new IllegalStateException("rigid body builder is closed");
            }
        }
    }
}
