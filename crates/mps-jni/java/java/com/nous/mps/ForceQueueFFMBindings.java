package com.nous.mps;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodType;

/**
 * FFM (Foreign Function & Memory) bindings for the force queue consumer.
 * Uses Java 21+ FFM API (JEP 454) to call the native
 * {@code rigid_body_consume_force_queue} function directly without JNI overhead.
 *
 * Usage:
 * <pre>{@code
 * try (Arena arena = Arena.ofConfined()) {
 *     ForceQueueFFM queue = ForceQueueFFM.allocate(arena, 1024, 7);
 *     // ... enqueue forces ...
 *
 *     // Call native consumer via FFM
 *     ForceQueueFFMBindings.consumeForceQueue(worldPtr, queue.address());
 * }
 * }</pre>
 */
public final class ForceQueueFFMBindings {

    // Native library name (same as JNI: mps_rigid_body)
    private static final String LIB_NAME = "mps_rigid_body";

    // Linker and symbol lookup
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LIB_LOOKUP = SymbolLookup.libraryLookup(LIB_NAME, Arena.global());

    // Function descriptor for rigid_body_consume_force_queue:
    // uint32_t rigid_body_consume_force_queue(void* world, ForceQueueHeader* queue)
    private static final FunctionDescriptor CONSUME_FQ_DESC =
        FunctionDescriptor.of(ValueLayout.JAVA_INT,      // return: u32
            ValueLayout.ADDRESS,                         // world: WorldHandle*
            ValueLayout.ADDRESS);                        // queue: ForceQueueHeader*

    // Downcall method handle for rigid_body_consume_force_queue
    private static final MethodHandle CONSUME_FQ_HANDLE;

    static {
        try {
            MemorySegment symbol = LIB_LOOKUP.find("rigid_body_consume_force_queue")
                .orElseThrow(() -> new UnsatisfiedLinkError(
                    "Cannot find rigid_body_consume_force_queue in " + LIB_NAME));
            CONSUME_FQ_HANDLE = LINKER.downcallHandle(symbol, CONSUME_FQ_DESC);
        } catch (Throwable e) {
            throw new ExceptionInInitializerError(e);
        }
    }

    private ForceQueueFFMBindings() {}

    /**
     * Calls the native {@code rigid_body_consume_force_queue} function via FFM.
     *
     * @param worldPtr  Pointer to WorldHandle (from {@code worldCreate})
     * @param queueAddr Base address of the ForceQueueHeader (from {@code ForceQueue.segment().address()} or {@code ForceQueueFFM.segment().address()})
     * @return Error code (0 = ERR_OK)
     * @throws Throwable if the downcall fails
     */
    public static int consumeForceQueue(long worldPtr, long queueAddr) throws Throwable {
        return (int) CONSUME_FQ_HANDLE.invokeExact(worldPtr, queueAddr);
    }

    /**
     * Calls the native {@code rigid_body_consume_force_queue} function via FFM
     * using MemorySegment addresses.
     *
     * @param worldSegment  MemorySegment for WorldHandle
     * @param queueSegment  MemorySegment for ForceQueueHeader
     * @return Error code (0 = ERR_OK)
     * @throws Throwable if the downcall fails
     */
    public static int consumeForceQueue(MemorySegment worldSegment, MemorySegment queueSegment) throws Throwable {
        return (int) CONSUME_FQ_HANDLE.invokeExact(worldSegment, queueSegment);
    }

    /**
     * Loads the native library explicitly (optional, usually done automatically).
     */
    public static void loadLibrary() {
        System.loadLibrary(LIB_NAME);
    }

    /**
     * Checks if the native symbol is available.
     */
    public static boolean isAvailable() {
        return LIB_LOOKUP.find("rigid_body_consume_force_queue").isPresent();
    }
}