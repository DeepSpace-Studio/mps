package org.polaris2023.msp_rigid_body.util;


import org.polaris2023.msp_rigid_body.RigidBodyNative;

public final class PhysicsWorld implements AutoCloseable {
    Long handle;
    double deltaSeconds = 1.0 / 60.0;
    RigidBody.Builder builder;
    RigidBody rigidBody;
    public PhysicsWorld(double gravityX, double gravityY, double gravityZ) {
        handle = RigidBodyNative.worldCreate(gravityX, gravityY, gravityZ);
    }

    public boolean isEmpty() {
        return handle == 0L;
    }

    public PhysicsWorld translation(double x, double y, double z) {
        builder.translation(x, y, z);
        return this;
    }

    public double[] translation() {
        return rigidBody.translation(this);
    }
    public double translationX() {
        return translation()[0];
    }
    public double translationY() {
        return translation()[1];
    }
    public double translationZ() {
        return translation()[2];
    }

    public RigidBody.Builder body() {
        builder = RigidBody.Builder.builder(this).build();
        return builder;
    }

    public RigidBody.Builder body(int status) {
        builder = RigidBody.Builder.builder(this).status(status).build();
        return builder;
    }

    public PhysicsWorld insert() {
        rigidBody = builder.body(this);
        return this;
    }

    public long handle() {
        return handle;
    }

    public double[] gravity() {
        return RigidBodyNative.worldGetGravity(handle);
    }

    public double gravityX() {
        return gravity()[0];
    }

    public double gravityY() {
        return gravity()[1];
    }

    public double gravityZ() {
        return gravity()[2];
    }



    public PhysicsWorld set(double gravityX, double gravityY, double gravityZ) {
        RigidBodyNative.worldSetGravity(handle, gravityX, gravityY, gravityZ);
        return this;
    }

    public PhysicsWorld step() {
        RigidBodyNative.worldStep(handle, deltaSeconds);
        return this;
    }

    public PhysicsWorld deltaSeconds(double deltaSeconds) {
        this.deltaSeconds = deltaSeconds;
        return this;
    }

    @Override
    public void close() throws Exception {
        RigidBodyNative.worldDestroy(handle);
        builder.close();
        handle = null;
    }
}
