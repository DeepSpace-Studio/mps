package team.dove.rigidbody;

import java.nio.file.Files;
import java.nio.file.Path;

public final class RigidBodyNative {
    static {
        loadNativeLibrary();
    }

    private RigidBodyNative() {
    }

    private static void loadNativeLibrary() {
        String explicitPath = System.getProperty("rigidbody.native.path");
        if (explicitPath != null && !explicitPath.isBlank()) {
            System.load(Path.of(explicitPath).toAbsolutePath().normalize().toString());
            return;
        }

        try {
            System.loadLibrary("rigid_body");
            return;
        } catch (UnsatisfiedLinkError loadLibraryError) {
            String mappedName = System.mapLibraryName("rigid_body");
            Path[] candidates = {
                    Path.of("target", "release", mappedName),
                    Path.of("..", "target", "release", mappedName)
            };

            for (Path candidate : candidates) {
                Path absolute = candidate.toAbsolutePath().normalize();
                if (Files.isRegularFile(absolute)) {
                    System.load(absolute.toString());
                    return;
                }
            }

            UnsatisfiedLinkError error = new UnsatisfiedLinkError(
                    loadLibraryError.getMessage()
                            + "; also tried -Drigidbody.native.path and "
                            + "target/release/" + mappedName);
            error.initCause(loadLibraryError);
            throw error;
        }
    }

    public static native int abiVersion();

    public static native long worldCreate(double gravityX, double gravityY, double gravityZ);

    public static native void worldDestroy(long world);

    public static native void worldStep(long world, double deltaSeconds);

    public static native void worldSetGravity(long world, double gravityX, double gravityY, double gravityZ);

    public static native double worldGetGravityX(long world);

    public static native double worldGetGravityY(long world);

    public static native double worldGetGravityZ(long world);

    public static native int worldDynamicBodySnapshotCount(long world);

    public static native long rigidBodyBuilderCreate(int status);

    public static native void rigidBodyBuilderDestroy(long builder);

    public static native void rigidBodyBuilderSetTranslation(long builder, double x, double y, double z);

    public static native long worldInsertRigidBody(long world, long builder);

    public static native double rigidBodyGetTranslationX(long world, long body);

    public static native double rigidBodyGetTranslationY(long world, long body);

    public static native double rigidBodyGetTranslationZ(long world, long body);
}
