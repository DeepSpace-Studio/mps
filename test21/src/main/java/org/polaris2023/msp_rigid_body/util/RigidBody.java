package org.polaris2023.msp_rigid_body.util;

import org.polaris2023.msp_rigid_body.RigidBodyNative;

public final class RigidBody {
    private long body;

    RigidBody() {
    }

    RigidBody(long body) {
        this.body = body;
    }

    public boolean isEmpty() {
        return body == 0L;
    }

    public long handle() {
        requirePresent();
        return body;
    }

    public double[] translation(PhysicsWorld world) {
        requirePresent();
        return RigidBodyNative.rigidBodyGetTranslation(world.handle(), body);
    }

    public double[] rotation(PhysicsWorld world) {
        requirePresent();
        return RigidBodyNative.rigidBodyGetRotation(world.handle(), body);
    }

    public double[] linvel(PhysicsWorld world) {
        requirePresent();
        return RigidBodyNative.rigidBodyGetLinvel(world.handle(), body);
    }

    public double[] angvel(PhysicsWorld world) {
        requirePresent();
        return RigidBodyNative.rigidBodyGetAngvel(world.handle(), body);
    }

    public RigidBody translation(PhysicsWorld world, double x, double y, double z, boolean wakeUp) {
        requirePresent();
        RigidBodyNative.rigidBodySetTranslation(world.handle(), body, x, y, z, wakeUp ? 1 : 0);
        return this;
    }

    public RigidBody pose(
            PhysicsWorld world,
            double x, double y, double z,
            double qi, double qj, double qk, double qw,
            boolean wakeUp) {
        requirePresent();
        RigidBodyNative.rigidBodySetPose(world.handle(), body, x, y, z, qi, qj, qk, qw, wakeUp ? 1 : 0);
        return this;
    }

    public RigidBody linvel(PhysicsWorld world, double x, double y, double z, boolean wakeUp) {
        requirePresent();
        RigidBodyNative.rigidBodySetLinvel(world.handle(), body, x, y, z, wakeUp ? 1 : 0);
        return this;
    }

    public RigidBody angvel(PhysicsWorld world, double x, double y, double z, boolean wakeUp) {
        requirePresent();
        RigidBodyNative.rigidBodySetAngvel(world.handle(), body, x, y, z, wakeUp ? 1 : 0);
        return this;
    }

    public RigidBody addForce(PhysicsWorld world, double x, double y, double z, boolean wakeUp) {
        requirePresent();
        RigidBodyNative.rigidBodyAddForce(world.handle(), body, x, y, z, wakeUp ? 1 : 0);
        return this;
    }

    public RigidBody applyImpulse(PhysicsWorld world, double x, double y, double z, boolean wakeUp) {
        requirePresent();
        RigidBodyNative.rigidBodyApplyImpulse(world.handle(), body, x, y, z, wakeUp ? 1 : 0);
        return this;
    }

    public boolean remove(PhysicsWorld world, boolean removeAttachedColliders) {
        requirePresent();
        boolean removed = RigidBodyNative.worldRemoveRigidBody(world.handle(), body, removeAttachedColliders ? 1 : 0);
        if (removed) {
            body = 0L;
        }
        return removed;
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

        public Builder linvel(double x, double y, double z) {
            requireOpen();
            RigidBodyNative.rigidBodyBuilderSetLinvel(handle, x, y, z);
            return this;
        }

        public Builder damping(double linear, double angular) {
            requireOpen();
            RigidBodyNative.rigidBodyBuilderSetLinearDamping(handle, linear);
            RigidBodyNative.rigidBodyBuilderSetAngularDamping(handle, angular);
            return this;
        }

        public Builder additionalMass(double mass) {
            requireOpen();
            RigidBodyNative.rigidBodyBuilderSetAdditionalMass(handle, mass);
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

    private void requirePresent() {
        if (body == 0L) {
            throw new IllegalStateException("rigid body is empty");
        }
    }
}
