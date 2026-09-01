package com.nous.mps;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.VarHandle;

/**
 * FFM (Foreign Function & Memory) variant of ForceQueue using MemorySegment directly.
 * For Java 21+ with FFM API (JEP 454).
 *
 * Same memory layout as ForceQueue, but uses MemorySegment and Arena for
 * deterministic lifetime management instead of DirectByteBuffer.
 */
public final class ForceQueueFFM {
    // Header offsets (bytes) - same as ForceQueue
    private static final long OFF_CAPACITY = 0;
    private static final long OFF_HEAD = 8;
    private static final long OFF_TAIL = 16;
    private static final long OFF_GENERATION = 24;
    private static final long OFF_STRIDE = 32;
    private static final long OFF_FLAGS = 36;
    private static final int HEADER_SIZE = 64;

    // VarHandles for atomic access
    private static final VarHandle VH_CAPACITY;
    private static final VarHandle VH_HEAD;
    private static final VarHandle VH_TAIL;
    private static final VarHandle VH_GENERATION;
    private static final VarHandle VH_STRIDE;
    private static final VarHandle VH_FLAGS;
    private static final VarHandle VH_BITMAP_WORD;
    private static final VarHandle VH_PAYLOAD_DOUBLE;

    static {
        try {
            VH_CAPACITY = MethodHandles.byteBufferViewVarHandle(long.class, java.nio.ByteOrder.LITTLE_ENDIAN);
            VH_HEAD = MethodHandles.byteBufferViewVarHandle(long.class, java.nio.ByteOrder.LITTLE_ENDIAN);
            VH_TAIL = MethodHandles.byteBufferViewVarHandle(long.class, java.nio.ByteOrder.LITTLE_ENDIAN);
            VH_GENERATION = MethodHandles.byteBufferViewVarHandle(long.class, java.nio.ByteOrder.LITTLE_ENDIAN);
            VH_STRIDE = MethodHandles.byteBufferViewVarHandle(int.class, java.nio.ByteOrder.LITTLE_ENDIAN);
            VH_FLAGS = MethodHandles.byteBufferViewVarHandle(int.class, java.nio.ByteOrder.LITTLE_ENDIAN);
            VH_BITMAP_WORD = MethodHandles.byteBufferViewVarHandle(long.class, java.nio.ByteOrder.LITTLE_ENDIAN);
            VH_PAYLOAD_DOUBLE = MethodHandles.byteBufferViewVarHandle(double.class, java.nio.ByteOrder.LITTLE_ENDIAN);
        } catch (IllegalAccessException e) {
            throw new ExceptionInInitializerError(e);
        }
    }

    private final MemorySegment segment;
    private final Arena arena;
    private final long capacity;
    private final int stride;
    private final int bitmapWords;
    private final long bitmapOffset;
    private final long payloadOffset;

    /**
     * Allocates a new ForceQueueFFM in the given Arena.
     *
     * @param arena    Memory arena for allocation lifetime
     * @param capacity Number of slots (power of 2)
     * @param stride   f64 per slot: 6 (force only) or 7 (force + torque)
     * @return Allocated ForceQueueFFM
     */
    public static ForceQueueFFM allocate(Arena arena, long capacity, int stride) {
        if (capacity <= 0 || (capacity & (capacity - 1)) != 0) {
            throw new IllegalArgumentException("capacity must be a power of 2");
        }
        if (stride != 6 && stride != 7) {
            throw new IllegalArgumentException("stride must be 6 or 7");
        }

        int bitmapWords = (int) ((capacity + 63) / 64);
        long bitmapSize = (long) bitmapWords * 8L;
        long payloadSize = capacity * stride * 8L;
        long totalSize = HEADER_SIZE + bitmapSize + payloadSize;

        MemorySegment segment = arena.allocate(ValueLayout.JAVA_BYTE, totalSize);

        // Initialize header
        segment.set(ValueLayout.JAVA_LONG_UNALIGNED, OFF_CAPACITY, capacity);
        segment.set(ValueLayout.JAVA_LONG_UNALIGNED, OFF_HEAD, 0L);
        segment.set(ValueLayout.JAVA_LONG_UNALIGNED, OFF_TAIL, 0L);
        segment.set(ValueLayout.JAVA_LONG_UNALIGNED, OFF_GENERATION, 0L);
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, OFF_STRIDE, stride);
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, OFF_FLAGS, 0);

        return new ForceQueueFFM(segment, arena, capacity, stride, bitmapWords);
    }

    /**
     * Wraps an existing native MemorySegment (e.g., from JNI GetDirectBufferAddress).
     */
    public static ForceQueueFFM wrap(MemorySegment segment, long capacity, int stride) {
        if (capacity <= 0 || (capacity & (capacity - 1)) != 0) {
            throw new IllegalArgumentException("capacity must be a power of 2");
        }
        if (stride != 6 && stride != 7) {
            throw new IllegalArgumentException("stride must be 6 or 7");
        }
        int bitmapWords = (int) ((capacity + 63) / 64);
        return new ForceQueueFFM(segment, null, capacity, stride, bitmapWords);
    }

    private ForceQueueFFM(MemorySegment segment, Arena arena, long capacity, int stride, int bitmapWords) {
        this.segment = segment;
        this.arena = arena;
        this.capacity = capacity;
        this.stride = stride;
        this.bitmapWords = bitmapWords;
        this.bitmapOffset = HEADER_SIZE;
        this.payloadOffset = HEADER_SIZE + (long) bitmapWords * 8L;
    }

    /**
     * Enqueues force and optionally torque.
     */
    public long enqueue(long bodyId, double fx, double fy, double fz,
                        double tx, double ty, double tz) {
        long head = (long) VH_HEAD.getAcquire(segment, OFF_HEAD);
        long tail = (long) VH_TAIL.getAcquire(segment, OFF_TAIL);

        long nextHead = (head + 1) & (capacity - 1);
        if (nextHead == tail) {
            return -1; // queue full
        }

        long base = payloadOffset + head * stride * 8L;
        segment.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 0, (double) bodyId);
        segment.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 8, fx);
        segment.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 16, fy);
        segment.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 24, fz);

        if (stride == 7) {
            segment.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 32, tx);
            segment.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 40, ty);
            segment.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 48, tz);
        }

        setBitmapBit(head, true);
        VH_HEAD.setRelease(segment, OFF_HEAD, nextHead);

        return head;
    }

    public long enqueueForce(long bodyId, double fx, double fy, double fz) {
        if (stride != 6) {
            throw new IllegalStateException("stride is " + stride);
        }
        return enqueue(bodyId, fx, fy, fz, 0, 0, 0);
    }

    public long enqueueForceTorque(long bodyId, double fx, double fy, double fz,
                                   double tx, double ty, double tz) {
        if (stride != 7) {
            throw new IllegalStateException("stride is " + stride);
        }
        return enqueue(bodyId, fx, fy, fz, tx, ty, tz);
    }

    public void cancel(long index) {
        if (index < 0 || index >= capacity) {
            throw new IndexOutOfBoundsException("index=" + index + " capacity=" + capacity);
        }
        setBitmapBit(index, false);
    }

    public void setPaused(boolean paused) {
        int flags = (int) VH_FLAGS.getAcquire(segment, OFF_FLAGS);
        if (paused) flags |= 1;
        else flags &= ~1;
        VH_FLAGS.setRelease(segment, OFF_FLAGS, flags);
    }

    public boolean isPaused() {
        int flags = (int) VH_FLAGS.getAcquire(segment, OFF_FLAGS);
        return (flags & 1) != 0;
    }

    /** Returns the base address for downcall/FFI. */
    public long address() {
        return segment.address();
    }

    public long capacity() { return capacity; }
    public int stride() { return stride; }
    public long head() { return (long) VH_HEAD.getAcquire(segment, OFF_HEAD); }
    public long tail() { return (long) VH_TAIL.getAcquire(segment, OFF_TAIL); }
    public long generation() { return (long) VH_GENERATION.getAcquire(segment, OFF_GENERATION); }
    public MemorySegment segment() { return segment; }
    public Arena arena() { return arena; }

    private void setBitmapBit(long index, boolean value) {
        long wordIdx = index >>> 6;
        int bit = (int) (index & 63);
        long wordOffset = bitmapOffset + wordIdx * 8L;

        long word;
        do {
            word = (long) VH_BITMAP_WORD.getAcquire(segment, wordOffset);
            long newWord = value ? (word | (1L << bit)) : (word & ~(1L << bit));
        } while (!VH_BITMAP_WORD.compareAndSet(segment, wordOffset, word, value ? (word | (1L << bit)) : (word & ~(1L << bit))));
    }

    public boolean isBitSet(long index) {
        if (index < 0 || index >= capacity) return false;
        long wordIdx = index >>> 6;
        int bit = (int) (index & 63);
        long wordOffset = bitmapOffset + wordIdx * 8L;
        long word = (long) VH_BITMAP_WORD.getAcquire(segment, wordOffset);
        return (word & (1L << bit)) != 0;
    }
}