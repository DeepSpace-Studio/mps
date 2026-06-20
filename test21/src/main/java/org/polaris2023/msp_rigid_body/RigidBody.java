package org.polaris2023.msp_rigid_body;

public final class RigidBody {
    private RigidBody() {
    }

    public static int abiVersion() {
        return RigidBodyNative.abiVersion();
    }

    public static boolean abiSupportsFfm() {
        return RigidBodyNative.abiSupportsFfm();
    }

    public static boolean abiSupportsJni() {
        return RigidBodyNative.abiSupportsJni();
    }

    public static long worldCreate(double gravityX, double gravityY, double gravityZ) {
        return RigidBodyNative.worldCreate(gravityX, gravityY, gravityZ);
    }

    public static void worldDestroy(long world) {
        RigidBodyNative.worldDestroy(world);
    }

    public static void worldStep(long world, double deltaSeconds) {
        RigidBodyNative.worldStep(world, deltaSeconds);
    }

    public static void worldSetGravity(long world, double x, double y, double z) {
        RigidBodyNative.worldSetGravity(world, x, y, z);
    }

    public static double[] worldGetGravity(long world) {
        return RigidBodyNative.worldGetGravity(world);
    }

    public static double worldGetGravityX(long world) {
        return worldGetGravity(world)[0];
    }

    public static double worldGetGravityY(long world) {
        return worldGetGravity(world)[1];
    }

    public static double worldGetGravityZ(long world) {
        return worldGetGravity(world)[2];
    }

    public static long rigidBodyBuilderCreate(int status) {
        return RigidBodyNative.rigidBodyBuilderCreate(status);
    }

    public static void rigidBodyBuilderDestroy(long builder) {
        RigidBodyNative.rigidBodyBuilderDestroy(builder);
    }

    public static void rigidBodyBuilderSetTranslation(long builder, double x, double y, double z) {
        RigidBodyNative.rigidBodyBuilderSetTranslation(builder, x, y, z);
    }

    public static long worldInsertRigidBody(long world, long builder) {
        return RigidBodyNative.worldInsertRigidBody(world, builder);
    }

    public static double[] rigidBodyGetTranslation(long world, long body) {
        return RigidBodyNative.rigidBodyGetTranslation(world, body);
    }

    public static double rigidBodyGetTranslationX(long world, long body) {
        return rigidBodyGetTranslation(world, body)[0];
    }

    public static double rigidBodyGetTranslationY(long world, long body) {
        return rigidBodyGetTranslation(world, body)[1];
    }

    public static double rigidBodyGetTranslationZ(long world, long body) {
        return rigidBodyGetTranslation(world, body)[2];
    }

    public static long crbTreeCreate() {
        return RigidBodyNative.crbTreeCreate();
    }

    public static void crbTreeDestroy(long tree) {
        RigidBodyNative.crbTreeDestroy(tree);
    }

    public static void crbTreeClear(long tree) {
        RigidBodyNative.crbTreeClear(tree);
    }

    public static int crbTreeLen(long tree) {
        return RigidBodyNative.crbTreeLen(tree);
    }

    public static boolean crbTreeInsert(
            long tree, long id, double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
        return RigidBodyNative.crbTreeInsert(tree, id, minX, minY, minZ, maxX, maxY, maxZ);
    }

    public static boolean crbTreeUpdate(
            long tree, long id, double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
        return RigidBodyNative.crbTreeUpdate(tree, id, minX, minY, minZ, maxX, maxY, maxZ);
    }

    public static boolean crbTreeRemove(long tree, long id) {
        return RigidBodyNative.crbTreeRemove(tree, id);
    }

    public static int crbTreeQueryAabbCount(
            long tree, double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
        return RigidBodyNative.crbTreeQueryAabbCount(tree, minX, minY, minZ, maxX, maxY, maxZ);
    }

    public static int crbTreeQueryAabb(
            long tree, double minX, double minY, double minZ, double maxX, double maxY, double maxZ,
            long outIds, int capacity) {
        return RigidBodyNative.crbTreeQueryAabb(tree, minX, minY, minZ, maxX, maxY, maxZ, outIds, capacity);
    }

    public static long rtreeCreate() {
        return RigidBodyNative.rtreeCreate();
    }

    public static void rtreeDestroy(long tree) {
        RigidBodyNative.rtreeDestroy(tree);
    }

    public static boolean rtreeInsert(
            long tree, long id, double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
        return RigidBodyNative.rtreeInsert(tree, id, minX, minY, minZ, maxX, maxY, maxZ);
    }

    public static int rtreeQueryAabbCount(
            long tree, double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
        return RigidBodyNative.rtreeQueryAabbCount(tree, minX, minY, minZ, maxX, maxY, maxZ);
    }
}
