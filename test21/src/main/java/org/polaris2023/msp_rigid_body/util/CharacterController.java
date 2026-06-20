package org.polaris2023.msp_rigid_body.util;

import org.polaris2023.msp_rigid_body.RigidBodyNative;

public final class CharacterController implements AutoCloseable {
    private long handle;

    public CharacterController() {
        handle = RigidBodyNative.characterControllerCreate();
        if (handle == 0L) {
            throw new IllegalStateException("character controller create failed");
        }
    }

    public CharacterController up(double x, double y, double z) {
        requireOpen();
        RigidBodyNative.characterControllerSetUp(handle, x, y, z);
        return this;
    }

    public CharacterController offsetAbsolute(double offset) {
        requireOpen();
        RigidBodyNative.characterControllerSetOffsetAbsolute(handle, offset);
        return this;
    }

    public CharacterController slide(boolean slide) {
        requireOpen();
        RigidBodyNative.characterControllerSetSlide(handle, slide ? 1 : 0);
        return this;
    }

    public Movement moveCuboid(
            PhysicsWorld world,
            double dt,
            double hx, double hy, double hz,
            double tx, double ty, double tz,
            double dx, double dy, double dz) {
        requireOpen();
        try (NativeMemory out = new NativeMemory(32)) {
            RigidBodyNative.characterControllerMoveShape(
                    world.handle(),
                    handle,
                    dt,
                    Query.SHAPE_CUBOID, hx, hy, hz, 0.0,
                    tx, ty, tz,
                    0.0, 0.0, 0.0, 1.0,
                    dx, dy, dz,
                    out.address());
            return new Movement(out.getVec3(0), out.getBool(24), out.getBool(25));
        }
    }

    public int collisionCount() {
        requireOpen();
        return RigidBodyNative.characterControllerCollisionCount(handle);
    }

    public Collision collision(int index) {
        requireOpen();
        try (NativeMemory out = new NativeMemory(184)) {
            long collider = RigidBodyNative.characterControllerGetCollision(handle, index, out.address());
            return new Collision(
                    collider,
                    out.getVec3(8),
                    out.getVec3(32),
                    out.getVec3(56),
                    out.getVec3(80),
                    out.getVec3(104),
                    out.getVec3(128),
                    out.getVec3(152),
                    out.getDouble(176));
        }
    }

    @Override
    public void close() {
        if (handle != 0L) {
            RigidBodyNative.characterControllerDestroy(handle);
            handle = 0L;
        }
    }

    private void requireOpen() {
        if (handle == 0L) {
            throw new IllegalStateException("character controller is closed");
        }
    }

    public record Movement(double[] translation, boolean grounded, boolean slidingDownSlope) {
    }

    public record Collision(
            long collider,
            double[] characterTranslation,
            double[] translationApplied,
            double[] translationRemaining,
            double[] worldWitness1,
            double[] worldWitness2,
            double[] normal1,
            double[] normal2,
            double timeOfImpact) {
    }
}
