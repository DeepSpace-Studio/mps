package org.polaris2023.msp_rigid_body.util;

import org.polaris2023.msp_rigid_body.RigidBodyNative;

public final class RigidBody {
    public Long body;
    public Builder builder;

    public double[] translation(PhysicsWorld world) {
        return RigidBodyNative.rigidBodyGetTranslation(world.handle, body);
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
        int status;
        Long build;
        final PhysicsWorld parent;

        private Builder(PhysicsWorld parent) {
            status = 0;
            this.parent = parent;
        }

        public static Builder builder(PhysicsWorld parent) {
            return new Builder(parent);
        }

        public Builder status(int status) {
            this.status = status;
            return this;
        }

        public Builder translation(double x, double y, double z) {
            RigidBodyNative.rigidBodyBuilderSetTranslation(build, x, y, z);
            return this;
        }

        public Builder build() {
            build = RigidBodyNative.rigidBodyBuilderCreate(status);
            return this;
        }

        public RigidBody body(PhysicsWorld world) {
            RigidBody b = new RigidBody();
            b.body = RigidBodyNative.worldInsertRigidBody(world.handle, build);
            b.builder = this;
            return b;
        }

        public boolean isEmpty() {
            return build == 0L;
        }

        @Override
        public void close() throws Exception {
            RigidBodyNative.rigidBodyBuilderDestroy(build);
        }

        @Override
        public PhysicsWorld parent() {
            return parent;
        }
    }
}
