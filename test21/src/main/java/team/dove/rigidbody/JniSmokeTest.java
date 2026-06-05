package team.dove.rigidbody;

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

        System.out.println("JNI smoke test passed on Java " + javaVersion);
    }

    private static void assertClose(double expected, double actual, String label) {
        if (Math.abs(expected - actual) > EPSILON) {
            throw new AssertionError(label + ": expected " + expected + ", got " + actual);
        }
    }
}
