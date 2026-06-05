package org.polaris2023.msp_rigid_body;

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
            System.loadLibrary("msp_rigid_body");
            return;
        } catch (UnsatisfiedLinkError loadLibraryError) {
            String mappedName = System.mapLibraryName("msp_rigid_body");
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

    public static native long colliderBuilderCreatePointCloudBounds(long pointsXyz, int pointCount);

    public static native long colliderBuilderCreateDoubleBv(
            double aMinX,
            double aMinY,
            double aMinZ,
            double aMaxX,
            double aMaxY,
            double aMaxZ,
            double bMinX,
            double bMinY,
            double bMinZ,
            double bMaxX,
            double bMaxY,
            double bMaxZ);

    public static native long colliderBuilderCreateSkewedObb(
            double centerX,
            double centerY,
            double centerZ,
            double axisXX,
            double axisXY,
            double axisXZ,
            double axisYX,
            double axisYY,
            double axisYZ,
            double axisZX,
            double axisZY,
            double axisZZ);

    public static native long colliderBuilderCreateDiscreteObb(long pointsXyz, int pointCount, int axis);

    public static native long colliderBuilderCreateFusedCollapsingBounds(long pointsXyz, int pointCount, double padding);

    public static native long colliderBuilderCreateEdgeBvh(
            long verticesXyz,
            int vertexCount,
            long edges,
            int edgeCount,
            double radius);

    public static native long colliderBuilderCreateMedialSpheres(long spheresXyzw, int sphereCount);

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

    public static native long crbTreeCreate();

    public static native void crbTreeDestroy(long tree);

    public static native void crbTreeClear(long tree);

    public static native int crbTreeLen(long tree);

    public static native boolean crbTreeInsert(
            long tree,
            long id,
            double minX,
            double minY,
            double minZ,
            double maxX,
            double maxY,
            double maxZ);

    public static native boolean crbTreeUpdate(
            long tree,
            long id,
            double minX,
            double minY,
            double minZ,
            double maxX,
            double maxY,
            double maxZ);

    public static native boolean crbTreeRemove(long tree, long id);

    public static native int crbTreeQueryAabbCount(
            long tree,
            double minX,
            double minY,
            double minZ,
            double maxX,
            double maxY,
            double maxZ);

    public static native int crbTreeQueryAabb(
            long tree,
            double minX,
            double minY,
            double minZ,
            double maxX,
            double maxY,
            double maxZ,
            long outIds,
            int capacity);
}
