package com.google.gemini.model;

import org.apache.fory.Fory;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;

public class PerConnectionData {
    private Arena arena;
    private MemorySegment memorySegment;
    private Fory fory;
    private int connectionNumber;

    public final Arena getArena() {
        return arena;
    }

    public final void setArena(Arena arena) {
        this.arena = arena;
    }

    public final MemorySegment getMemorySegment() {
        return memorySegment;
    }

    public final void setMemorySegment(MemorySegment memorySegment) {
        this.memorySegment = memorySegment;
    }

    public final Fory getFory() {
        return fory;
    }

    public final void setFory(Fory fory) {
        this.fory = fory;
    }

    public final int getConnectionNumber() {
        return connectionNumber;
    }

    public final void setConnectionNumber(int connectionNumber) {
        this.connectionNumber = connectionNumber;
    }
}
