package team.dove.rigidbody.ffm;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.nio.file.Path;

public final class FfmSmokeTest {
    private static final double EPSILON = 1.0e-9;
    private static final MemoryLayout RC_VEC3 = MemoryLayout.structLayout(
            ValueLayout.JAVA_DOUBLE.withName("x"),
            ValueLayout.JAVA_DOUBLE.withName("y"),
            ValueLayout.JAVA_DOUBLE.withName("z"));
    private static final long RC_VEC3_X = RC_VEC3.byteOffset(MemoryLayout.PathElement.groupElement("x"));
    private static final long RC_VEC3_Y = RC_VEC3.byteOffset(MemoryLayout.PathElement.groupElement("y"));
    private static final long RC_VEC3_Z = RC_VEC3.byteOffset(MemoryLayout.PathElement.groupElement("z"));

    private FfmSmokeTest() {
    }

    public static void main(String[] args) throws Throwable {
        int javaVersion = Runtime.version().feature();
        if (javaVersion != 25) {
            throw new AssertionError("test25 must run on Java 25, got Java " + javaVersion);
        }

        String nativePath = System.getProperty("rigidbody.native.path");
        if (nativePath == null || nativePath.isBlank()) {
            throw new AssertionError("missing rigidbody.native.path");
        }

        try (Arena arena = Arena.ofShared()) {
            NativeApi api = new NativeApi(Path.of(nativePath), arena);

            int abiVersion = (int) api.rcAbiVersion.invokeExact();
            if (abiVersion < 1) {
                throw new AssertionError("invalid ABI version: " + abiVersion);
            }

            MemorySegment world = (MemorySegment) api.rcWorldCreate.invokeExact(vec3(arena, 0.0, -9.81, 0.0));
            if (world.equals(MemorySegment.NULL)) {
                throw new AssertionError("rc_world_create returned null");
            }

            try {
                MemorySegment outGravity = arena.allocate(RC_VEC3);
                api.rcWorldGetGravityOut.invokeExact(world, outGravity);
                assertClose(-9.81, outGravity.get(ValueLayout.JAVA_DOUBLE, RC_VEC3_Y), "initial gravity y");

                api.rcWorldSetGravity.invokeExact(world, vec3(arena, 1.0, 2.0, 3.0));
                api.rcWorldGetGravityOut.invokeExact(world, outGravity);
                assertClose(1.0, outGravity.get(ValueLayout.JAVA_DOUBLE, RC_VEC3_X), "gravity x");
                assertClose(2.0, outGravity.get(ValueLayout.JAVA_DOUBLE, RC_VEC3_Y), "gravity y");
                assertClose(3.0, outGravity.get(ValueLayout.JAVA_DOUBLE, RC_VEC3_Z), "gravity z");

                MemorySegment builder = (MemorySegment) api.rcRigidBodyBuilderCreate.invokeExact(0);
                if (builder.equals(MemorySegment.NULL)) {
                    throw new AssertionError("rc_rigid_body_builder_create returned null");
                }

                try {
                    api.rcRigidBodyBuilderSetTranslation.invokeExact(builder, vec3(arena, 4.0, 5.0, 6.0));
                    long body = (long) api.rcWorldInsertRigidBody.invokeExact(world, builder);
                    if (body == 0L) {
                        throw new AssertionError("rc_world_insert_rigid_body returned zero handle");
                    }

                    MemorySegment outTranslation = arena.allocate(RC_VEC3);
                    api.rcRigidBodyGetTranslationOut.invokeExact(world, body, outTranslation);
                    assertClose(4.0, outTranslation.get(ValueLayout.JAVA_DOUBLE, RC_VEC3_X), "body translation x");
                    assertClose(5.0, outTranslation.get(ValueLayout.JAVA_DOUBLE, RC_VEC3_Y), "body translation y");
                    assertClose(6.0, outTranslation.get(ValueLayout.JAVA_DOUBLE, RC_VEC3_Z), "body translation z");
                    api.rcWorldStep.invokeExact(world, 1.0 / 60.0);
                } finally {
                    api.rcRigidBodyBuilderDestroy.invokeExact(builder);
                }
            } finally {
                api.rcWorldDestroy.invokeExact(world);
            }
        }

        System.out.println("FFM smoke test passed on Java " + javaVersion);
    }

    private static MemorySegment vec3(Arena arena, double x, double y, double z) {
        MemorySegment value = arena.allocate(RC_VEC3);
        value.set(ValueLayout.JAVA_DOUBLE, RC_VEC3_X, x);
        value.set(ValueLayout.JAVA_DOUBLE, RC_VEC3_Y, y);
        value.set(ValueLayout.JAVA_DOUBLE, RC_VEC3_Z, z);
        return value;
    }

    private static void assertClose(double expected, double actual, String label) {
        if (Math.abs(expected - actual) > EPSILON) {
            throw new AssertionError(label + ": expected " + expected + ", got " + actual);
        }
    }

    private static final class NativeApi {
        private static final Linker LINKER = Linker.nativeLinker();

        private final SymbolLookup lookup;
        private final MethodHandle rcAbiVersion;
        private final MethodHandle rcWorldCreate;
        private final MethodHandle rcWorldDestroy;
        private final MethodHandle rcWorldStep;
        private final MethodHandle rcWorldSetGravity;
        private final MethodHandle rcWorldGetGravityOut;
        private final MethodHandle rcRigidBodyBuilderCreate;
        private final MethodHandle rcRigidBodyBuilderDestroy;
        private final MethodHandle rcRigidBodyBuilderSetTranslation;
        private final MethodHandle rcWorldInsertRigidBody;
        private final MethodHandle rcRigidBodyGetTranslationOut;

        private NativeApi(Path library, Arena arena) {
            lookup = SymbolLookup.libraryLookup(library, arena);
            rcAbiVersion = downcall("rc_abi_version", FunctionDescriptor.of(ValueLayout.JAVA_INT));
            rcWorldCreate = downcall("rc_world_create", FunctionDescriptor.of(ValueLayout.ADDRESS, RC_VEC3));
            rcWorldDestroy = downcall("rc_world_destroy", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
            rcWorldStep = downcall("rc_world_step", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.JAVA_DOUBLE));
            rcWorldSetGravity = downcall("rc_world_set_gravity", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, RC_VEC3));
            rcWorldGetGravityOut = downcall("rc_world_get_gravity_out", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
            rcRigidBodyBuilderCreate = downcall("rc_rigid_body_builder_create", FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.JAVA_INT));
            rcRigidBodyBuilderDestroy = downcall("rc_rigid_body_builder_destroy", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
            rcRigidBodyBuilderSetTranslation = downcall("rc_rigid_body_builder_set_translation", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, RC_VEC3));
            rcWorldInsertRigidBody = downcall("rc_world_insert_rigid_body", FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.ADDRESS));
            rcRigidBodyGetTranslationOut = downcall("rc_rigid_body_get_translation_out", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        }

        private MethodHandle downcall(String symbol, FunctionDescriptor descriptor) {
            MemorySegment address = lookup.find(symbol)
                    .orElseThrow(() -> new UnsatisfiedLinkError("missing native symbol: " + symbol));
            return LINKER.downcallHandle(address, descriptor);
        }
    }
}
