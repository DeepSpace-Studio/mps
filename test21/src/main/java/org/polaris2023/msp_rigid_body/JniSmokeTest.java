package org.polaris2023.msp_rigid_body;

public final class JniSmokeTest {
    private static final double EPSILON = 1.0e-9;

    private JniSmokeTest() {
    }

    public static void main(String[] args) {
        int javaVersion = Runtime.version().feature();
        if (javaVersion != 21) {
            throw new AssertionError("test21 must run on Java 21, got Java " + javaVersion);
        }

        int abiVersion = RigidBodyNative.abiVersion();
        if (abiVersion < 1) {
            throw new AssertionError("invalid ABI version: " + abiVersion);
        }

        long world = RigidBodyNative.worldCreate(0.0, -9.81, 0.0);
        if (world == 0L) {
            throw new AssertionError("worldCreate returned null");
        }

        try {
            assertClose(-9.81, RigidBodyNative.worldGetGravityY(world), "initial gravity y");
            RigidBodyNative.worldSetGravity(world, 1.0, 2.0, 3.0);
            assertClose(1.0, RigidBodyNative.worldGetGravityX(world), "gravity x");
            assertClose(2.0, RigidBodyNative.worldGetGravityY(world), "gravity y");
            assertClose(3.0, RigidBodyNative.worldGetGravityZ(world), "gravity z");

            long builder = RigidBodyNative.rigidBodyBuilderCreate(0);
            if (builder == 0L) {
                throw new AssertionError("rigidBodyBuilderCreate returned null");
            }

            try {
                RigidBodyNative.rigidBodyBuilderSetTranslation(builder, 4.0, 5.0, 6.0);
                long body = RigidBodyNative.worldInsertRigidBody(world, builder);
                if (body == 0L) {
                    throw new AssertionError("worldInsertRigidBody returned zero handle");
                }
                assertClose(4.0, RigidBodyNative.rigidBodyGetTranslationX(world, body), "body translation x");
                assertClose(5.0, RigidBodyNative.rigidBodyGetTranslationY(world, body), "body translation y");
                assertClose(6.0, RigidBodyNative.rigidBodyGetTranslationZ(world, body), "body translation z");
                RigidBodyNative.worldStep(world, 1.0 / 60.0);
            } finally {
                RigidBodyNative.rigidBodyBuilderDestroy(builder);
            }
        } finally {
            RigidBodyNative.worldDestroy(world);
        }

        long tree = RigidBodyNative.crbTreeCreate();
        if (tree == 0L) {
            throw new AssertionError("crbTreeCreate returned null");
        }
        try {
            if (!RigidBodyNative.crbTreeInsert(tree, 10L, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0)) {
                throw new AssertionError("crbTreeInsert 10 failed");
            }
            if (!RigidBodyNative.crbTreeInsert(tree, 20L, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0)) {
                throw new AssertionError("crbTreeInsert 20 failed");
            }
            int hitCount = RigidBodyNative.crbTreeQueryAabbCount(tree, 0.5, 0.5, 0.5, 2.5, 2.5, 2.5);
            if (hitCount != 2) {
                throw new AssertionError("crbTreeQueryAabbCount expected 2, got " + hitCount);
            }
        } finally {
            RigidBodyNative.crbTreeDestroy(tree);
        }

        System.out.println("JNI smoke test passed on Java " + javaVersion);
    }

    private static void assertClose(double expected, double actual, String label) {
        if (Math.abs(expected - actual) > EPSILON) {
            throw new AssertionError(label + ": expected " + expected + ", got " + actual);
        }
    }
}
