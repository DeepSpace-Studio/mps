package org.polaris2023.msp_rigid_body;

import org.polaris2023.msp_rigid_body.util.PhysicsWorld;
import org.polaris2023.msp_rigid_body.util.Query;
import org.polaris2023.msp_rigid_body.util.RigidBody;
import org.polaris2023.msp_rigid_body.util.SpatialIndex;
import org.polaris2023.msp_rigid_body.util.VoxelGrid;

public final class JniSmokeTest {
    private static final double EPSILON = 1.0e-9;
    private static final int VOXEL_MODE_GREEDY_CUBOIDS = 2;

    private JniSmokeTest() {
    }

    public static void main(String[] args) throws Exception {
        int javaVersion = Runtime.version().feature();
        if (javaVersion != 21) {
            throw new AssertionError("test21 must run on Java 21, got Java " + javaVersion);
        }

        int abiVersion = RigidBodyNative.abiVersion();
        if (abiVersion < 1) {
            throw new AssertionError("invalid ABI version: " + abiVersion);
        }

        try (PhysicsWorld world = new PhysicsWorld(0.0, -9.81, 0.0)) {
            if (world.isEmpty()) {
                throw new AssertionError("worldCreate returned null");
            }

            assertClose(-9.81, world.gravityY(), "initial gravity y");
            world.set(1.0, 2.0, 3.0);
            assertClose(1.0, world.gravityX(), "gravity x");
            assertClose(2.0, world.gravityY(), "gravity y");
            assertClose(3.0, world.gravityZ(), "gravity z");

            RigidBody.Builder body = world.body(0);
            if (body.isEmpty()) {
                throw new AssertionError("rigidBodyBuilderCreate returned null");
            }

            try {
                world.translation(4.0, 5.0, 6.0);
                world.insert();
                assertClose(4.0, world.translationX(), "body translation x");
                assertClose(5.0, world.translationY(), "body translation y");
                assertClose(6.0, world.translationZ(), "body translation z");
                world.step();
            } finally {
                body.close();
            }
        }

        assertVoxelColliderCanBeCreatedAndInserted();
        assertSafeWrappersCoverCommonJniFeatures();
        assertInvalidInputsAreRejected();

        System.out.println("JNI smoke test passed on Java " + javaVersion);
    }

    private static void assertVoxelColliderCanBeCreatedAndInserted() throws Exception {
        int sizeX = 16;
        int sizeY = 8;
        int sizeZ = 16;
        try (VoxelGrid voxels = new VoxelGrid(sizeX, sizeY, sizeZ).fillBox(4, 0, 4, 12, 4, 12);
             PhysicsWorld world = new PhysicsWorld(0.0, -9.81, 0.0);
             org.polaris2023.msp_rigid_body.util.Collider.Builder builder = world.voxelCollider(
                    voxels.address(),
                    voxels.sizeX(), voxels.sizeY(), voxels.sizeZ(),
                    1.0,
                    0.0, 0.0, 0.0,
                    VOXEL_MODE_GREEDY_CUBOIDS,
                    false,
                    128,
                    20_000)) {
            if (world.isEmpty()) {
                throw new AssertionError("worldCreate returned null for voxel test");
            }
            if (builder.isEmpty()) {
                throw new AssertionError("colliderBuilderCreateVoxels returned null");
            }

            org.polaris2023.msp_rigid_body.util.Collider collider = builder
                    .friction(0.8)
                    .restitution(0.1)
                    .insert();
            if (collider.isEmpty()) {
                throw new AssertionError("worldInsertCollider returned null for voxel collider");
            }
            if (world.colliderCount() != 1) {
                throw new AssertionError("voxel collider was not inserted into world");
            }

            world.step();
        }
    }

    private static void assertSafeWrappersCoverCommonJniFeatures() {
        try (PhysicsWorld world = new PhysicsWorld(0.0, -9.81, 0.0);
             RigidBody.Builder bodyBuilder = RigidBody.Builder.builder(world).status(0).build().translation(0.0, 4.0, 0.0);
             RigidBody.Builder otherBuilder = RigidBody.Builder.builder(world).status(0).build().translation(0.0, 6.0, 0.0)) {
            world.integrationParameters(1.0 / 120.0, 8, 2);
            double[] integration = world.integrationParameters();
            assertClose(1.0 / 120.0, integration[0], "integration dt");
            assertClose(8.0, integration[1], "integration solver iterations");
            assertClose(2.0, integration[2], "integration ccd substeps");

            RigidBody body = world.insert(bodyBuilder)
                    .linvel(world, 0.0, -1.0, 0.0, true);
            RigidBody other = world.insert(otherBuilder);
            PhysicsWorld.BodySnapshot[] before = world.bodySnapshot();
            if (before.length != 2) {
                throw new AssertionError("body snapshot expected 2 bodies, got " + before.length);
            }
            if (world.updateBodyPoses(new PhysicsWorld.BodyPoseUpdate[] {
                    new PhysicsWorld.BodyPoseUpdate(body.handle(), new double[] {1.0, 4.0, 0.0}, new double[] {0.0, 0.0, 0.0, 1.0})
            }, true) != 1) {
                throw new AssertionError("batch body pose update failed");
            }
            if (world.updateBodyVelocities(new PhysicsWorld.BodyVelocityUpdate[] {
                    new PhysicsWorld.BodyVelocityUpdate(body.handle(), new double[] {0.0, -2.0, 0.0}, new double[] {0.0, 0.0, 0.0})
            }, true) != 1) {
                throw new AssertionError("batch body velocity update failed");
            }

            org.polaris2023.msp_rigid_body.util.Collider collider = world.cuboidCollider(0.5, 0.5, 0.5)
                    .density(1.0)
                    .friction(0.4)
                    .restitution(0.2)
                    .insert();
            org.polaris2023.msp_rigid_body.util.Collider sphere = world.sphereCollider(0.0, 4.0, 0.0, 0.75)
                    .sensor(true)
                    .insert();
            if (collider.isEmpty() || sphere.isEmpty() || world.colliderCount() != 2) {
                throw new AssertionError("safe collider wrappers failed");
            }

            world.step();
            long[] sphereHits = world.query().intersectSphere(0.0, 4.0, 0.0, 2.0, 8);
            if (sphereHits.length < 1) {
                throw new AssertionError("query intersect sphere returned no hits");
            }
            Query.RayHit rayHit = world.query().castRay(0.0, 8.0, 0.0, 0.0, -1.0, 0.0, 20.0);
            if (rayHit.isEmpty()) {
                throw new AssertionError("query raycast returned no hit");
            }
            Query.RayHit[] rayHits = world.query().castRays(new double[] {
                    0.0, 8.0, 0.0, 0.0, -1.0, 0.0,
                    2.0, 8.0, 0.0, 0.0, -1.0, 0.0
            }, 20.0);
            if (rayHits.length != 2 || rayHits[0].isEmpty()) {
                throw new AssertionError("batch raycast wrapper failed");
            }
            if (world.query().countPoint(0.0, 4.0, 0.0) < 1) {
                throw new AssertionError("point intersection count returned no hits");
            }
            Query.PointProjection projection = world.query().projectPoint(0.0, 4.0, 0.0, 10.0, true);
            if (projection.isEmpty()) {
                throw new AssertionError("point projection returned no collider");
            }

            if (world.collisionEvents().length != world.collisionEventCount()) {
                throw new AssertionError("bulk collision event read count mismatch");
            }
            if (world.contactForceEvents().length != world.contactForceEventCount()) {
                throw new AssertionError("bulk contact force event read count mismatch");
            }
            RigidBodyNative.abiClearLastError();
            if (RigidBodyNative.worldGetCollisionEvents(world.handle(), 0L, 1) != 0 || RigidBodyNative.abiLastErrorCode() == 0) {
                throw new AssertionError("bulk event invalid output did not set last error");
            }

            try (org.polaris2023.msp_rigid_body.util.Joint.Builder jointBuilder = world.fixedJoint()) {
                org.polaris2023.msp_rigid_body.util.Joint joint = jointBuilder.insert(body, other, true);
                if (joint.isEmpty()) {
                    throw new AssertionError("joint insert failed");
                }
                if (!joint.remove(true)) {
                    throw new AssertionError("joint remove failed");
                }
            }

            try (org.polaris2023.msp_rigid_body.util.CharacterController controller =
                         new org.polaris2023.msp_rigid_body.util.CharacterController()) {
                org.polaris2023.msp_rigid_body.util.CharacterController.Movement movement =
                        controller.offsetAbsolute(0.01)
                                .slide(true)
                                .moveCuboid(world, 1.0 / 60.0, 0.25, 0.5, 0.25, 0.0, 8.0, 0.0, 0.0, -0.5, 0.0);
                if (movement.translation().length != 3 || controller.collisionCount() < 0) {
                    throw new AssertionError("character controller wrapper returned invalid data");
                }
            }
        }

        try (SpatialIndex tree = SpatialIndex.compactTree(); SpatialIndex rtree = SpatialIndex.rtree()) {
            assertSpatialIndex(tree, "compact tree");
            assertSpatialIndex(rtree, "rtree");
        }
    }

    private static void assertSpatialIndex(SpatialIndex tree, String label) {
        if (!tree.insert(10L, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0)) {
            throw new AssertionError(label + " insert 10 failed");
        }
        if (!tree.insert(20L, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0)) {
            throw new AssertionError(label + " insert 20 failed");
        }
        if (tree.countAabb(0.5, 0.5, 0.5, 2.5, 2.5, 2.5) != 2) {
            throw new AssertionError(label + " count failed");
        }
        long[] ids = tree.queryAabb(0.5, 0.5, 0.5, 2.5, 2.5, 2.5, 4);
        if (ids.length != 2) {
            throw new AssertionError(label + " query expected 2 hits, got " + ids.length);
        }
    }

    private static void assertInvalidInputsAreRejected() {
        long world = RigidBodyNative.worldCreate(0.0, -9.81, 0.0);
        if (world == 0L) {
            throw new AssertionError("worldCreate returned null for invalid input test");
        }
        try {
            double[] gravity = RigidBodyNative.worldGetGravity(world);
            RigidBodyNative.worldStep(world, Double.NaN);
            RigidBodyNative.worldSetGravity(world, Double.NaN, 1.0, 2.0);
            assertClose(gravity[1], RigidBodyNative.worldGetGravity(world)[1], "invalid gravity should be ignored");

            if (RigidBodyNative.colliderBuilderCreate(1, Double.NaN, 1.0, 1.0) != 0L) {
                throw new AssertionError("invalid cuboid builder should be rejected");
            }
            if (RigidBodyNative.colliderBuilderCreateVoxels(0L, 1, 1, 1, 1.0, 0.0, 0.0, 0.0, 0, 0, 128, 20_000) != 0L) {
                throw new AssertionError("null voxel pointer should be rejected");
            }
            if (RigidBodyNative.colliderBuilderBuild(0L) != 0L) {
                throw new AssertionError("null collider builder build should return null");
            }
            if (RigidBodyNative.rigidBodyBuilderBuild(0L) != 0L) {
                throw new AssertionError("null rigid body builder build should return null");
            }
            if (RigidBodyNative.queryIntersectSphere(
                    world,
                    0.0, 0.0, 0.0, 1.0,
                    0, 0xffff, 0xffff, 0,
                    0L, 0, 0L, 0,
                    0L, -1) != 0) {
                throw new AssertionError("negative query capacity should be rejected");
            }
            if (RigidBodyNative.crbTreeQueryAabb(0L, 0, 0, 0, 1, 1, 1, 0L, -1) != 0) {
                throw new AssertionError("negative tree query capacity should be rejected");
            }
        } finally {
            RigidBodyNative.worldDestroy(world);
        }
    }

    private static void assertClose(double expected, double actual, String label) {
        if (Math.abs(expected - actual) > EPSILON) {
            throw new AssertionError(label + ": expected " + expected + ", got " + actual);
        }
    }
}
