package com.nous.mps;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.VarHandle;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

/**
 * Zero-copy shared-memory force queue between Java and Rust (Rapier).
 * Uses DirectByteBuffer (or MemorySegment) for off-heap memory.
 *
 * Memory layout (matches Rust ForceQueueHeader):
 * - Header (64 bytes, cache-line aligned):
 *   capacity (u64), head (u64), tail (u64), generation (u64), stride (u32), flags (u32)
 * - Bitmap: (capacity + 63) / 64 u64 words (1 bit per slot)
 * - Payload: capacity * stride * 8 bytes (f64 per component)
 *   stride = 6 (body_id + force[3]) or 7 (body_id + force[3] + torque[3])
 *
 * Single-producer / single-consumer lock-free:
 * - Java is sole writer to each slot's payload and its bitmap bit.
 * - Rust (native) is sole reader of slots where bitmap bit = 1.
 * - head/tail use release/acquire ordering via VarHandle.
 */
public final class ForceQueue {
    // Header offsets (bytes)
    private static final long OFF_CAPACITY = 0;
    private static final long OFF_HEAD = 8;
    private static final long OFF_TAIL = 16;
    private static final long OFF_GENERATION = 24;
    private static final long OFF_STRIDE = 32;
    private static final long OFF_FLAGS = 36;
    private static final int HEADER_SIZE = 64; // cache-line aligned

    // VarHandles for atomic access to header fields
    private static final VarHandle VH_CAPACITY;
    private static final VarHandle VH_HEAD;
    private static final VarHandle VH_TAIL;
    private static final VarHandle VH_GENERATION;
    private static final VarHandle VH_STRIDE;
    private static final VarHandle VH_FLAGS;

    // Bitmap and payload
    private final MemorySegment buffer;
    private final long capacity;
    private final int stride;
    private final int bitmapWords;
    private final long bitmapOffset;
    private final long payloadOffset;

    static {
        try {
            VH_CAPACITY = MethodHandles.byteBufferViewVarHandle(long.class, ByteOrder.LITTLE_ENDIAN);
            VH_HEAD = MethodHandles.byteBufferViewVarHandle(long.class, ByteOrder.LITTLE_ENDIAN);
            VH_TAIL = MethodHandles.byteBufferViewVarHandle(long.class, ByteOrder.LITTLE_ENDIAN);
            VH_GENERATION = MethodHandles.byteBufferViewVarHandle(long.class, ByteOrder.LITTLE_ENDIAN);
            VH_STRIDE = MethodHandles.byteBufferViewVarHandle(int.class, ByteOrder.LITTLE_ENDIAN);
            VH_FLAGS = MethodHandles.byteBufferViewVarHandle(int.class, ByteOrder.LITTLE_ENDIAN);
        } catch (IllegalAccessException e) {
            throw new ExceptionInInitializerError(e);
        }
    }

    /**
     * Creates a new ForceQueue with the given capacity and stride.
     * Capacity must be a power of 2.
     *
     * @param capacity Number of slots (power of 2, e.g., 1024)
     * @param stride   f64 count per slot: 6 (body_id + force) or 7 (body_id + force + torque)
     * @return Allocated ForceQueue
     */
    public static ForceQueue allocate(long capacity, int stride) {
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

        // Allocate DirectByteBuffer (off-heap)
        ByteBuffer bb = ByteBuffer.allocateDirect((int) totalSize);
        bb.order(ByteOrder.LITTLE_ENDIAN);

        MemorySegment segment = MemorySegment.ofBuffer(bb);

        // Initialize header
        segment.set(ValueLayout.JAVA_LONG_UNALIGNED, OFF_CAPACITY, capacity);
        segment.set(ValueLayout.JAVA_LONG_UNALIGNED, OFF_HEAD, 0L);
        segment.set(ValueLayout.JAVA_LONG_UNALIGNED, OFF_TAIL, 0L);
        segment.set(ValueLayout.JAVA_LONG_UNALIGNED, OFF_GENERATION, 0L);
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, OFF_STRIDE, stride);
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, OFF_FLAGS, 0);

        // Zero bitmap and payload (already zeroed by allocateDirect)

        return new ForceQueue(segment, capacity, stride, bitmapWords);
    }

    /**
     * Wraps an existing DirectByteBuffer or MemorySegment as a ForceQueue.
     * Useful when memory is allocated externally (e.g., via JNI/FFM).
     */
    public static ForceQueue wrap(MemorySegment segment, long capacity, int stride) {
        if (capacity <= 0 || (capacity & (capacity - 1)) != 0) {
            throw new IllegalArgumentException("capacity must be a power of 2");
        }
        if (stride != 6 && stride != 7) {
            throw new IllegalArgumentException("stride must be 6 or 7");
        }
        int bitmapWords = (int) ((capacity + 63) / 64);
        return new ForceQueue(segment, capacity, stride, bitmapWords);
    }

    private ForceQueue(MemorySegment buffer, long capacity, int stride, int bitmapWords) {
        this.buffer = buffer;
        this.capacity = capacity;
        this.stride = stride;
        this.bitmapWords = bitmapWords;
        this.bitmapOffset = HEADER_SIZE;
        this.payloadOffset = HEADER_SIZE + (long) bitmapWords * 8L;
    }

    /**
     * Enqueues a force (and optionally torque) for a rigid body.
     * Returns the slot index, or -1 if queue is full.
     */
    public long enqueue(long bodyId, double fx, double fy, double fz,
                        double tx, double ty, double tz) {
        // Load head with acquire semantics
        long head = (long) VH_HEAD.getAcquire(buffer, OFF_HEAD);
        long tail = (long) VH_TAIL.getAcquire(buffer, OFF_TAIL);

        // Check if full: (head + 1) % capacity == tail
        long nextHead = (head + 1) & (capacity - 1);
        if (nextHead == tail) {
            return -1; // queue full
        }

        // Write payload
        long base = payloadOffset + head * stride * 8L;
        buffer.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 0, (double) bodyId);
        buffer.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 8, fx);
        buffer.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 16, fy);
        buffer.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 24, fz);

        if (stride == 7) {
            buffer.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 32, tx);
            buffer.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 40, ty);
            buffer.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, base + 48, tz);
        }

        // Set bitmap bit with release semantics
        setBitmapBit(head, true);

        // Advance head with release semantics
        VH_HEAD.setRelease(buffer, OFF_HEAD, nextHead);

        return head;
    }

    /**
     * Enqueues a force only (stride=6).
     */
    public long enqueueForce(long bodyId, double fx, double fy, double fz) {
        if (stride != 6) {
            throw new IllegalStateException("stride is " + stride + ", use enqueue() for stride=7");
        }
        return enqueue(bodyId, fx, fy, fz, 0, 0, 0);
    }

    /**
     * Enqueues force and torque (stride=7).
     */
    public long enqueueForceTorque(long bodyId, double fx, double fy, double fz,
                                   double tx, double ty, double tz) {
        if (stride != 7) {
            throw new IllegalStateException("stride is " + stride + ", use enqueueForce() for stride=6");
        }
        return enqueue(bodyId, fx, fy, fz, tx, ty, tz);
    }

    /**
     * Cancels a previously enqueued slot by clearing its bitmap bit.
     * O(1) — no array shifting needed.
     */
    public void cancel(long index) {
        if (index < 0 || index >= capacity) {
            throw new IndexOutOfBoundsException("index=" + index + " capacity=" + capacity);
        }
        setBitmapBit(index, false);
    }

    /**
     * Sets the paused flag. When true, native consumer skips processing this frame.
     */
    public void setPaused(boolean paused) {
        int flags = (int) VH_FLAGS.getAcquire(buffer, OFF_FLAGS);
        if (paused) {
            flags |= 1;
        } else {
            flags &= ~1;
        }
        VH_FLAGS.setRelease(buffer, OFF_FLAGS, flags);
    }

    public boolean isPaused() {
        int flags = (int) VH_FLAGS.getAcquire(buffer, OFF_FLAGS);
        return (flags & 1) != 0;
    }

    /** Returns the base address of the buffer for JNI/FFM calls. */
    public long address() {
        return buffer.address();
    }

    /** Returns the capacity (number of slots). */
    public long capacity() {
        return capacity;
    }

    /** Returns the stride (f64 per slot: 6 or 7). */
    public int stride() {
        return stride;
    }

    /** Returns the current head index (next write position). */
    public long head() {
        return (long) VH_HEAD.getAcquire(buffer, OFF_HEAD);
    }

    /** Returns the current tail index (next read position). */
    public long tail() {
        return (long) VH_TAIL.getAcquire(buffer, OFF_TAIL);
    }

    /** Returns the generation counter. */
    public long generation() {
        return (long) VH_GENERATION.getAcquire(buffer, OFF_GENERATION);
    }

    /** Returns the underlying MemorySegment. */
    public MemorySegment segment() {
        return buffer;
    }

    // --- Internal bitmap manipulation ---

    private void setBitmapBit(long index, boolean value) {
        long wordIdx = index >>> 6; // index / 64
        int bit = (int) (index & 63); // index % 64
        long wordOffset = bitmapOffset + wordIdx * 8L;

        // Use VarHandle for atomic getAndSet on the u64 word
        VarHandle vhWord = MethodHandles.byteBufferViewVarHandle(long.class, ByteOrder.LITTLE_ENDIAN);
        long word;
        do {
            word = (long) vhWord.getAcquire(buffer, wordOffset);
            long newWord = value ? (word | (1L << bit)) : (word & ~(1L << bit));
            // Compare-and-set loop (single-writer per word, but use CAS for safety)
        } while (!vhWord.compareAndSet(buffer, wordOffset, word, value ? (word | (1L << bit)) : (word & ~(1L << bit))));
    }

    /**
     * Checks if a bitmap bit is set (for debugging/inspection).
     */
    public boolean isBitSet(long index) {
        if (index < 0 || index >= capacity) return false;
        long wordIdx = index >>> 6;
        int bit = (int) (index & 63);
        long wordOffset = bitmapOffset + wordIdx * 8L;
        VarHandle vhWord = MethodHandles.byteBufferViewVarHandle(long.class, ByteOrder.LITTLE_ENDIAN);
        long word = (long) vhWord.getAcquire(buffer, wordOffset);
        return (word & (1L << bit)) != 0;
    }
}