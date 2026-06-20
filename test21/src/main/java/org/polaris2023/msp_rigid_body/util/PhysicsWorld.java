package org.polaris2023.msp_rigid_body.util;

import org.polaris2023.msp_rigid_body.RigidBodyNative;

public final class PhysicsWorld implements AutoCloseable {
    private long handle;
    private double deltaSeconds = 1.0 / 60.0;
    private RigidBody.Builder builder;
    private RigidBody rigidBody;

    public PhysicsWorld(double gravityX, double gravityY, double gravityZ) {
        handle = RigidBodyNative.worldCreate(gravityX, gravityY, gravityZ);
    }

    public boolean isEmpty() {
        return handle == 0L;
    }

    public long handle() {
        requireOpen();
        return handle;
    }

    public int colliderCount() {
        requireOpen();
        return RigidBodyNative.worldGetColliderSetSize(handle);
    }

    public int rigidBodyCount() {
        requireOpen();
        return RigidBodyNative.worldGetRigidBodySetSize(handle);
    }

    public PhysicsWorld set(double gravityX, double gravityY, double gravityZ) {
        requireOpen();
        RigidBodyNative.worldSetGravity(handle, gravityX, gravityY, gravityZ);
        return this;
    }

    public double[] gravity() {
        requireOpen();
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

    public PhysicsWorld deltaSeconds(double deltaSeconds) {
        requireOpen();
        this.deltaSeconds = deltaSeconds;
        return this;
    }

    public PhysicsWorld integrationParameters(double dt, int solverIterations, int ccdSubsteps) {
        requireOpen();
        if (!RigidBodyNative.worldSetIntegrationParameters(handle, dt, solverIterations, ccdSubsteps)) {
            throw new IllegalArgumentException(RigidBodyNative.abiLastErrorMessage());
        }
        this.deltaSeconds = dt;
        return this;
    }

    public double[] integrationParameters() {
        requireOpen();
        try (NativeMemory out = new NativeMemory(3L * Double.BYTES)) {
            int written = RigidBodyNative.worldGetIntegrationParameters(handle, out.address(), 3);
            if (written != 3) {
                throw new IllegalStateException(RigidBodyNative.abiLastErrorMessage());
            }
            return new double[] {out.getDouble(0), out.getDouble(8), out.getDouble(16)};
        }
    }

    public PhysicsWorld step() {
        requireOpen();
        RigidBodyNative.worldStep(handle, deltaSeconds);
        return this;
    }

    public RigidBody.Builder body() {
        return body(0);
    }

    public RigidBody.Builder body(int status) {
        requireOpen();
        builder = RigidBody.Builder.builder(this).status(status).build();
        return builder;
    }

    public RigidBody insert(RigidBody.Builder builder) {
        requireOpen();
        if (builder == null || builder.isEmpty()) {
            throw new IllegalArgumentException("rigid body builder is empty");
        }
        return builder.body(this);
    }

    public Collider.Builder cuboidCollider(double hx, double hy, double hz) {
        requireOpen();
        return Collider.Builder.cuboid(this, hx, hy, hz);
    }

    public Collider.Builder sphereCollider(double x, double y, double z, double radius) {
        requireOpen();
        return Collider.Builder.sphere(this, x, y, z, radius);
    }

    public Collider.Builder capsuleCollider(double ax, double ay, double az, double bx, double by, double bz, double radius) {
        requireOpen();
        return Collider.Builder.capsule(this, ax, ay, az, bx, by, bz, radius);
    }

    public Collider.Builder cylinderCollider(double x, double y, double z, double radius, double halfHeight) {
        requireOpen();
        return Collider.Builder.cylinder(this, x, y, z, radius, halfHeight);
    }

    public PhysicsWorld translation(double x, double y, double z) {
        if (builder == null) {
            throw new IllegalStateException("body() must be called before translation()");
        }
        builder.translation(x, y, z);
        return this;
    }

    public Collider.Builder voxelCollider(
            long voxels, int sizeX, int sizeY, int sizeZ, double voxelSize,
            double originX, double originY, double originZ,
            int mode, boolean dynamicBody, int smallVoxelLimit, int meshVoxelLimit) {
        requireOpen();
        return Collider.Builder.voxels(this,
                voxels, sizeX, sizeY, sizeZ, voxelSize,
                originX, originY, originZ,
                mode, dynamicBody, smallVoxelLimit, meshVoxelLimit);
    }

    public Collider.Builder voxelCollider(long voxels, int sizeX, int sizeY, int sizeZ, double voxelSize) {
        requireOpen();
        return Collider.Builder.voxels(this, voxels, sizeX, sizeY, sizeZ, voxelSize);
    }

    public Collider.Builder voxelCollider(
            long voxels, int sizeX, int sizeY, int sizeZ, double voxelSize,
            double originX, double originY, double originZ, boolean dynamicBody) {
        requireOpen();
        return Collider.Builder.voxels(this,
                voxels, sizeX, sizeY, sizeZ, voxelSize,
                originX, originY, originZ, dynamicBody);
    }

    public Collider.Builder voxelCollider(byte[] voxels, int sizeX, int sizeY, int sizeZ, double voxelSize) {
        requireOpen();
        return Collider.Builder.voxelBytes(this, voxels, sizeX, sizeY, sizeZ, voxelSize);
    }

    public Collider.Builder voxelCollider(
            byte[] voxels, int sizeX, int sizeY, int sizeZ, double voxelSize,
            double originX, double originY, double originZ, boolean dynamicBody) {
        requireOpen();
        return Collider.Builder.voxelBytes(this,
                voxels, sizeX, sizeY, sizeZ, voxelSize,
                originX, originY, originZ, dynamicBody);
    }

    public Collider.Builder voxelCollider(
            byte[] voxels, int sizeX, int sizeY, int sizeZ, double voxelSize,
            double originX, double originY, double originZ,
            int mode, boolean dynamicBody, int smallVoxelLimit, int meshVoxelLimit) {
        requireOpen();
        return Collider.Builder.voxelBytes(this,
                voxels, sizeX, sizeY, sizeZ, voxelSize,
                originX, originY, originZ,
                mode, dynamicBody, smallVoxelLimit, meshVoxelLimit);
    }

    public Collider.Builder voxelAabbCollider(
            double minX, double minY, double minZ,
            double maxX, double maxY, double maxZ,
            double voxelSize) {
        requireOpen();
        return Collider.Builder.voxelAabb(this, minX, minY, minZ, maxX, maxY, maxZ, voxelSize);
    }

    public Collider.Builder voxelAabbCollider(
            double minX, double minY, double minZ,
            double maxX, double maxY, double maxZ,
            double voxelSize, boolean dynamicBody) {
        requireOpen();
        return Collider.Builder.voxelAabb(this, minX, minY, minZ, maxX, maxY, maxZ, voxelSize, dynamicBody);
    }

    public Collider.Builder voxelAabbCollider(
            double minX, double minY, double minZ,
            double maxX, double maxY, double maxZ,
            double voxelSize, int mode, boolean dynamicBody, int smallVoxelLimit, int meshVoxelLimit) {
        requireOpen();
        return Collider.Builder.voxelAabb(
                this, minX, minY, minZ, maxX, maxY, maxZ,
                voxelSize, mode, dynamicBody, smallVoxelLimit, meshVoxelLimit);
    }

    public Collider.Builder voxelObbCollider(
            double cx, double cy, double cz,
            double hx, double hy, double hz,
            double qi, double qj, double qk, double qw,
            double voxelSize) {
        requireOpen();
        return Collider.Builder.voxelObb(this, cx, cy, cz, hx, hy, hz, qi, qj, qk, qw, voxelSize);
    }

    public Collider.Builder voxelObbCollider(
            double cx, double cy, double cz,
            double hx, double hy, double hz,
            double qi, double qj, double qk, double qw,
            double voxelSize, boolean dynamicBody) {
        requireOpen();
        return Collider.Builder.voxelObb(this, cx, cy, cz, hx, hy, hz, qi, qj, qk, qw, voxelSize, dynamicBody);
    }

    public Collider.Builder voxelObbCollider(
            double cx, double cy, double cz,
            double hx, double hy, double hz,
            double qi, double qj, double qk, double qw,
            double voxelSize, int mode, boolean dynamicBody, int smallVoxelLimit, int meshVoxelLimit) {
        requireOpen();
        return Collider.Builder.voxelObb(
                this, cx, cy, cz, hx, hy, hz, qi, qj, qk, qw,
                voxelSize, mode, dynamicBody, smallVoxelLimit, meshVoxelLimit);
    }

    public Collider insert(Collider.Raw raw) {
        requireOpen();
        if (raw == null || raw.isEmpty()) {
            throw new IllegalArgumentException("raw collider is empty");
        }
        long collider = RigidBodyNative.worldInsertCollider(handle, raw.release());
        return new Collider(this, collider);
    }

    public Collider insert(Collider.Raw raw, RigidBody parent) {
        requireOpen();
        if (raw == null || raw.isEmpty()) {
            throw new IllegalArgumentException("raw collider is empty");
        }
        long collider = RigidBodyNative.worldInsertColliderWithParent(handle, raw.release(), parent.handle());
        return new Collider(this, collider);
    }

    public RigidBody insertStaticVoxelAabb(
            double minX, double minY, double minZ,
            double maxX, double maxY, double maxZ,
            double voxelSize, double friction, double restitution) {
        requireOpen();
        long body = RigidBodyNative.worldInsertStaticVoxelAabb(
                handle,
                minX, minY, minZ,
                maxX, maxY, maxZ,
                voxelSize,
                Collider.Builder.VOXEL_MODE_AUTO,
                Collider.Builder.DEFAULT_SMALL_VOXEL_LIMIT,
                Collider.Builder.DEFAULT_MESH_VOXEL_LIMIT,
                friction,
                restitution);
        return new RigidBody(body);
    }

    public RigidBody insertDynamicVoxelObb(
            double cx, double cy, double cz,
            double hx, double hy, double hz,
            double qi, double qj, double qk, double qw,
            double voxelSize, double density, double friction, double restitution) {
        requireOpen();
        long body = RigidBodyNative.worldInsertDynamicVoxelObb(
                handle,
                cx, cy, cz,
                hx, hy, hz,
                qi, qj, qk, qw,
                voxelSize,
                Collider.Builder.VOXEL_MODE_AUTO,
                Collider.Builder.DEFAULT_SMALL_VOXEL_LIMIT,
                Collider.Builder.DEFAULT_MESH_VOXEL_LIMIT,
                density,
                friction,
                restitution);
        return new RigidBody(body);
    }

    public Joint.Builder fixedJoint() {
        requireOpen();
        return Joint.Builder.fixed(this);
    }

    public Joint.Builder revoluteJoint(double ax, double ay, double az) {
        requireOpen();
        return Joint.Builder.revolute(this, ax, ay, az);
    }

    public Query query() {
        requireOpen();
        return new Query(this);
    }

    public void clearEvents() {
        requireOpen();
        RigidBodyNative.worldClearEvents(handle);
    }

    public int collisionEventCount() {
        requireOpen();
        return RigidBodyNative.worldCollisionEventCount(handle);
    }

    public CollisionEvent collisionEvent(int index) {
        requireOpen();
        try (NativeMemory out = new NativeMemory(32)) {
            RigidBodyNative.worldGetCollisionEvent(handle, index, out.address());
            return new CollisionEvent(
                    out.getBool(0),
                    out.getLong(8),
                    out.getLong(16),
                    out.getBool(24),
                    out.getBool(25));
        }
    }

    public CollisionEvent[] collisionEvents() {
        requireOpen();
        int count = collisionEventCount();
        if (count <= 0) {
            return new CollisionEvent[0];
        }
        try (NativeMemory out = new NativeMemory((long) count * 32L)) {
            int written = RigidBodyNative.worldGetCollisionEvents(handle, out.address(), count);
            CollisionEvent[] events = new CollisionEvent[written];
            for (int i = 0; i < written; i++) {
                long offset = (long) i * 32L;
                events[i] = new CollisionEvent(
                        out.getBool(offset),
                        out.getLong(offset + 8),
                        out.getLong(offset + 16),
                        out.getBool(offset + 24),
                        out.getBool(offset + 25));
            }
            return events;
        }
    }

    public int contactForceEventCount() {
        requireOpen();
        return RigidBodyNative.worldContactForceEventCount(handle);
    }

    public ContactForceEvent contactForceEvent(int index) {
        requireOpen();
        try (NativeMemory out = new NativeMemory(80)) {
            RigidBodyNative.worldGetContactForceEvent(handle, index, out.address());
            return new ContactForceEvent(
                    out.getLong(0),
                    out.getLong(8),
                    out.getVec3(16),
                    out.getDouble(40),
                    out.getVec3(48),
                    out.getDouble(72));
        }
    }

    public ContactForceEvent[] contactForceEvents() {
        requireOpen();
        int count = contactForceEventCount();
        if (count <= 0) {
            return new ContactForceEvent[0];
        }
        try (NativeMemory out = new NativeMemory((long) count * 80L)) {
            int written = RigidBodyNative.worldGetContactForceEvents(handle, out.address(), count);
            ContactForceEvent[] events = new ContactForceEvent[written];
            for (int i = 0; i < written; i++) {
                long offset = (long) i * 80L;
                events[i] = new ContactForceEvent(
                        out.getLong(offset),
                        out.getLong(offset + 8),
                        out.getVec3(offset + 16),
                        out.getDouble(offset + 40),
                        out.getVec3(offset + 48),
                        out.getDouble(offset + 72));
            }
            return events;
        }
    }

    public BodySnapshot[] bodySnapshot() {
        requireOpen();
        int count = RigidBodyNative.worldBodySnapshotCount(handle);
        if (count <= 0) {
            return new BodySnapshot[0];
        }
        try (NativeMemory handles = NativeMemory.longs(count);
             NativeMemory values = new NativeMemory((long) count * 13L * Double.BYTES)) {
            int written = RigidBodyNative.worldBodySnapshot(handle, handles.address(), values.address(), count);
            BodySnapshot[] snapshots = new BodySnapshot[written];
            for (int i = 0; i < written; i++) {
                long valueOffset = (long) i * 13L * Double.BYTES;
                snapshots[i] = new BodySnapshot(
                        handles.getLong((long) i * Long.BYTES),
                        values.getVec3(valueOffset),
                        new double[] {
                                values.getDouble(valueOffset + 24),
                                values.getDouble(valueOffset + 32),
                                values.getDouble(valueOffset + 40),
                                values.getDouble(valueOffset + 48)
                        },
                        values.getVec3(valueOffset + 56),
                        values.getVec3(valueOffset + 80));
            }
            return snapshots;
        }
    }

    public int updateBodyPoses(BodyPoseUpdate[] updates, boolean wakeUp) {
        requireOpen();
        if (updates == null || updates.length == 0) {
            return 0;
        }
        try (NativeMemory handles = NativeMemory.longs(updates.length);
             NativeMemory values = new NativeMemory((long) updates.length * 7L * Double.BYTES)) {
            for (int i = 0; i < updates.length; i++) {
                BodyPoseUpdate update = updates[i];
                handles.putLong((long) i * Long.BYTES, update.handle());
                long offset = (long) i * 7L * Double.BYTES;
                putVec3(values, offset, update.translation());
                putQuat(values, offset + 24, update.rotation());
            }
            return RigidBodyNative.worldUpdateBodyPoses(handle, handles.address(), values.address(), updates.length, wakeUp ? 1 : 0);
        }
    }

    public int updateBodyVelocities(BodyVelocityUpdate[] updates, boolean wakeUp) {
        requireOpen();
        if (updates == null || updates.length == 0) {
            return 0;
        }
        try (NativeMemory handles = NativeMemory.longs(updates.length);
             NativeMemory values = new NativeMemory((long) updates.length * 6L * Double.BYTES)) {
            for (int i = 0; i < updates.length; i++) {
                BodyVelocityUpdate update = updates[i];
                handles.putLong((long) i * Long.BYTES, update.handle());
                long offset = (long) i * 6L * Double.BYTES;
                putVec3(values, offset, update.linvel());
                putVec3(values, offset + 24, update.angvel());
            }
            return RigidBodyNative.worldUpdateBodyVelocities(handle, handles.address(), values.address(), updates.length, wakeUp ? 1 : 0);
        }
    }

    public PhysicsWorld insert() {
        if (builder == null) {
            throw new IllegalStateException("body() must be called before insert()");
        }
        rigidBody = builder.body(this);
        return this;
    }

    public double[] translation() {
        if (rigidBody == null) {
            throw new IllegalStateException("insert() must be called before translation()");
        }
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

    @Override
    public void close() {
        if (builder != null) {
            builder.close();
            builder = null;
        }
        if (handle != 0L) {
            RigidBodyNative.worldDestroy(handle);
            handle = 0L;
        }
    }

    private void requireOpen() {
        if (handle == 0L) {
            throw new IllegalStateException("world is closed");
        }
    }

    private static void putVec3(NativeMemory memory, long offset, double[] value) {
        if (value == null || value.length < 3) {
            throw new IllegalArgumentException("vec3 requires at least 3 values");
        }
        memory.putDouble(offset, value[0]);
        memory.putDouble(offset + 8, value[1]);
        memory.putDouble(offset + 16, value[2]);
    }

    private static void putQuat(NativeMemory memory, long offset, double[] value) {
        if (value == null || value.length < 4) {
            throw new IllegalArgumentException("quat requires at least 4 values");
        }
        memory.putDouble(offset, value[0]);
        memory.putDouble(offset + 8, value[1]);
        memory.putDouble(offset + 16, value[2]);
        memory.putDouble(offset + 24, value[3]);
    }

    public record CollisionEvent(boolean started, long collider1, long collider2, boolean sensor, boolean removed) {
    }

    public record ContactForceEvent(
            long collider1,
            long collider2,
            double[] totalForce,
            double totalForceMagnitude,
            double[] maxForceDirection,
            double maxForceMagnitude) {
    }

    public record BodySnapshot(
            long handle,
            double[] translation,
            double[] rotation,
            double[] linvel,
            double[] angvel) {
    }

    public record BodyPoseUpdate(long handle, double[] translation, double[] rotation) {
    }

    public record BodyVelocityUpdate(long handle, double[] linvel, double[] angvel) {
    }
}
