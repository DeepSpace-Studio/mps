package org.polaris2023.mps_rigid_body.ffm;

import java.lang.foreign.*;
import java.lang.foreign.MemoryLayout.PathElement;

/**
 * Zero-JNI/FFM physics state access via shared memory arena.  Uses Java 25
 * {@code MemorySegment} for direct native memory access.</p>
 */
public final class SharedPhysicsArena {

    static final int HEADER_SIZE = 128;
    static final int BODY_SLOT_STRIDE = 96;
    static final int CMD_SLOT_STRIDE = 32;

    static final long OFF_BODY_COUNT = 32;
    static final long OFF_EVENT_COUNT = 40;
    static final long OFF_CMD_WRITE = 44;
    // Region offsets (u64, written by Rust — read these, never recompute)
    static final long OFF_CMD_RING_OFFSET = 96;
    static final long OFF_EVENT_RING_OFFSET = 104;

    static final int CMD_ADD_FORCE = 0;

    private final MemorySegment seg;
    private final int maxBodies;
    private final long bodySlotsStart;
    private final long cmdRingStart;
    private final long eventRingStart;

    public SharedPhysicsArena(MemorySegment segment) {
        this.seg = segment;
        long magic = seg.get(ValueLayout.JAVA_LONG_UNALIGNED, 0);
        if (magic != 0x4D50535F4152454EL) {
            throw new IllegalArgumentException("invalid arena magic: 0x" + Long.toHexString(magic));
        }
        this.maxBodies = seg.get(ValueLayout.JAVA_INT_UNALIGNED, 16);
        this.bodySlotsStart = HEADER_SIZE;
        this.cmdRingStart = seg.get(ValueLayout.JAVA_LONG_UNALIGNED, OFF_CMD_RING_OFFSET);
        this.eventRingStart = seg.get(ValueLayout.JAVA_LONG_UNALIGNED, OFF_EVENT_RING_OFFSET);
    }

    public int getMaxBodies()   { return maxBodies; }
    public int getBodyCount()   { return seg.get(ValueLayout.JAVA_INT_UNALIGNED, OFF_BODY_COUNT); }
    public int getEventCount()  { return seg.get(ValueLayout.JAVA_INT_UNALIGNED, OFF_EVENT_COUNT); }

    private long bodyAddr(int i) { return bodySlotsStart + (long) i * BODY_SLOT_STRIDE; }
    public double getBodyPX(int i)  { return seg.get(ValueLayout.JAVA_DOUBLE_UNALIGNED, bodyAddr(i) + 8); }
    public double getBodyPY(int i)  { return seg.get(ValueLayout.JAVA_DOUBLE_UNALIGNED, bodyAddr(i) + 16); }
    public double getBodyPZ(int i)  { return seg.get(ValueLayout.JAVA_DOUBLE_UNALIGNED, bodyAddr(i) + 24); }
    public double getBodyVX(int i)  { return seg.get(ValueLayout.JAVA_DOUBLE_UNALIGNED, bodyAddr(i) + 32); }
    public double getBodyVY(int i)  { return seg.get(ValueLayout.JAVA_DOUBLE_UNALIGNED, bodyAddr(i) + 40); }
    public double getBodyVZ(int i)  { return seg.get(ValueLayout.JAVA_DOUBLE_UNALIGNED, bodyAddr(i) + 48); }

    private long cmdAddr(int i) { return cmdRingStart + (long)i * CMD_SLOT_STRIDE; }
    public void cmdAddForce(int bodyIdx, double fx, double fy, double fz) {
        // Protocol: write the slot, then bump the write index at header
        // offset 44; Rust drains [0, write) at worldStep and resets it to 0.
        int write = seg.get(ValueLayout.JAVA_INT_UNALIGNED, OFF_CMD_WRITE);
        long a = cmdAddr(write);
        seg.set(ValueLayout.JAVA_INT_UNALIGNED, a, CMD_ADD_FORCE);
        seg.set(ValueLayout.JAVA_INT_UNALIGNED, a + 4, bodyIdx);
        seg.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, a + 8, fx);
        seg.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, a + 16, fy);
        seg.set(ValueLayout.JAVA_DOUBLE_UNALIGNED, a + 24, fz);
        seg.set(ValueLayout.JAVA_INT_UNALIGNED, OFF_CMD_WRITE, write + 1);
    }
}