package com.google.gemini.ffi;

import com.google.gemini.model.ForyRequest;
import com.google.gemini.model.PerConnectionData;
import org.apache.fory.Fory;
import org.apache.fory.config.Language;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.Stream;


public class UpCallMethodStub {
    @SuppressWarnings("unused")
    private static final long CALLBACK_ADDRESS_FORY_REQUEST_SUPPLIER;
    @SuppressWarnings("unused")
    private static final long CALLBACK_ADDRESS_INIT_CONNECTION;
    private static final List<PerConnectionData> PER_CONNECTION_DATA = Stream.<PerConnectionData>generate(() -> null).limit(50000).collect(Collectors.toCollection(ArrayList::new));

    static {
        MethodHandles.Lookup lookup = MethodHandles.lookup();
        @SuppressWarnings("unused")
        MethodType longReturnType = MethodType.methodType(long.class, int.class);
        MethodType voidReturnType = MethodType.methodType(void.class, int.class);
        @SuppressWarnings("unused")
        FunctionDescriptor longFunctionDescriptor = FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.JAVA_INT);
        FunctionDescriptor voidFunctionDescriptor = FunctionDescriptor.ofVoid(ValueLayout.JAVA_INT);
        // This method handle is to get the address where rust can call java to get the next request
        try {
            MethodHandle foryRequestSupplierMethodHandle = lookup.findStatic(UpCallMethodStub.class, "foryRequestSupplier", voidReturnType);
            CALLBACK_ADDRESS_FORY_REQUEST_SUPPLIER = Linker.nativeLinker().upcallStub(foryRequestSupplierMethodHandle, voidFunctionDescriptor, Arena.global()).address();
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
        try {
            MethodHandle foryRequestSupplierMethodHandle = lookup.findStatic(UpCallMethodStub.class, "initConnection", voidReturnType);
            CALLBACK_ADDRESS_INIT_CONNECTION = Linker.nativeLinker().upcallStub(foryRequestSupplierMethodHandle, voidFunctionDescriptor, Arena.global()).address();
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    public static void foryRequestSupplier(int connectionNumber) {
        PerConnectionData perConnectionData = PER_CONNECTION_DATA.get(connectionNumber);
        Fory fory = perConnectionData.getFory();
        MemorySegment memorySegment = perConnectionData.getMemorySegment();
        try {
            byte[] bytes = fory.serialize(new ForyRequest("default", "http://test.com/", "test.com", 80, "GET", "/", new int[]{200}, 1000L, true, Map.of("test", "value"), Map.of("test", "value"), ""));
            final int length = bytes.length;
            // First 4 bytes are reserved for message size so that rust knows how many bytes to read
            memorySegment.set(ValueLayout.JAVA_INT, 0, length);
            MemorySegment.copy(bytes, 0, memorySegment, ValueLayout.JAVA_BYTE, 4, length);
        } catch(Exception e) {
            throw new RuntimeException(e);
        }
    }

    public static void initConnection(int connectionNumber) {
        PerConnectionData perConnectionData = new PerConnectionData();
        Arena arena = Arena.ofShared();
        MemorySegment memorySegment = arena.allocate(Integer.parseInt(System.getenv().getOrDefault("SEGMENT_SIZE_IN_BYTES", "10240")));
        Fory fory = Fory.builder()
                .withLanguage(Language.RUST) // As we will be sending it to rust, better to serialize in rust format
                .withAsyncCompilation(true)
                .requireClassRegistration(true)
                .build();
        fory.register(ForyRequest.class, "com.google.gemini", "fory_request");
        // Warn up fory
        for (int i = 0; i < 1000; i++) {
            fory.serialize(new ForyRequest("default", "http://test.com/", "test.com", 80, "GET", "/", new int[]{200}, 1000L, true, Map.of("test", "value"), Map.of("test", "value"), "empty body"));
        }
        perConnectionData.setConnectionNumber(connectionNumber);
        perConnectionData.setArena(arena);
        perConnectionData.setMemorySegment(memorySegment);
        perConnectionData.setFory(fory);
        PER_CONNECTION_DATA.add(connectionNumber, perConnectionData);
    }
}
