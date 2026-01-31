package com.google.gemini.ffi;

import com.google.gemini.model.ForyRequest;
import com.google.gemini.model.PerConnectionData;
import org.apache.fory.Fory;
import org.apache.fory.config.CompatibleMode;
import org.apache.fory.config.Language;
import org.apache.fory.logging.Logger;
import org.apache.fory.logging.LoggerFactory;

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
    private final static Logger log = LoggerFactory.getLogger(UpCallMethodStub.class);
    @SuppressWarnings("unused")
    private static final long CALLBACK_ADDRESS_FORY_REQUEST_SUPPLIER;
    @SuppressWarnings("unused")
    private static final long CALLBACK_ADDRESS_INIT_CONNECTION;
    @SuppressWarnings("unused")
    public static final long CALLBACK_GET_MEMORY_SEGMENT_ADDRESS;
    // Support max 50,000 connections per container or rust binary executing it.
    private static final List<PerConnectionData> PER_CONNECTION_DATA = Stream.<PerConnectionData>generate(() -> null).limit(50000).collect(Collectors.toCollection(ArrayList::new));

    static {
        MethodHandles.Lookup lookup = MethodHandles.lookup();
        MethodType longReturnType = MethodType.methodType(long.class, int.class);
        MethodType voidReturnType = MethodType.methodType(void.class, int.class);
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
            MethodHandle initConnectionMethodHandle = lookup.findStatic(UpCallMethodStub.class, "initConnection", voidReturnType);
            CALLBACK_ADDRESS_INIT_CONNECTION = Linker.nativeLinker().upcallStub(initConnectionMethodHandle, voidFunctionDescriptor, Arena.global()).address();
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
        try {
            MethodHandle memorySegmentAddressMethodHandle = lookup.findStatic(UpCallMethodStub.class, "memorySegmentAddress", longReturnType);
            CALLBACK_GET_MEMORY_SEGMENT_ADDRESS = Linker.nativeLinker().upcallStub(memorySegmentAddressMethodHandle, longFunctionDescriptor, Arena.global()).address();
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    @SuppressWarnings("unused")
    public static long getMethodHandleForInitConnection() {
        return CALLBACK_ADDRESS_INIT_CONNECTION;
    }

    @SuppressWarnings("unused")
    public static long getMethodHandleForForyRequestSupplier() {
        return CALLBACK_ADDRESS_FORY_REQUEST_SUPPLIER;
    }

    @SuppressWarnings("unused")
    public static long getMethodHandleForMemorySegmentAddress() {
        return CALLBACK_GET_MEMORY_SEGMENT_ADDRESS;
    }

    /**
     * Gives the address of the shared arena memory segment for the connection number
     * @param connectionNumber The connection number of the thread
     * @return The shared arena memory address
     */
    public static long memorySegmentAddress(int connectionNumber) {
        return PER_CONNECTION_DATA.get(connectionNumber).getMemorySegment().address();
    }

    /**
     * The ForyRequest supplier which serializes the ForyRequest object
     * and writes it in shared arena with first 4 bytes in the litten endian way having the length of the byte[]
     * @param connectionNumber The connection number of the thread
     */
    public static void foryRequestSupplier(final int connectionNumber) {
        //log.info("Inside the fory request supplier method for connection: {}", connectionNumber);
        final PerConnectionData perConnectionData = PER_CONNECTION_DATA.get(connectionNumber);
        try {
            byte[] bytes = perConnectionData.getFory().serialize(new ForyRequest("default", "https://jsonplaceholder.typicode.com/todos/1", "jsonplaceholder.typicode.com", 443, "GET", "/todos/1", new int[]{200}, 1000L, true, null, null, ""));
            final int length = bytes.length;
            //log.info("Byte length from Java: {}", length);
            // First 4 bytes are reserved for message size so that rust knows how many bytes to read
            perConnectionData.getMemorySegment().set(ValueLayout.JAVA_INT, 0, length);
            // This will throw exception if the memory segment is smaller than the whole thing
            MemorySegment.copy(bytes, 0, perConnectionData.getMemorySegment(), ValueLayout.JAVA_BYTE, 4, length);
        } catch(Exception e) {
            throw new RuntimeException(e);
        }
    }

    /**
     * Initializes the arena, memory segment and fory for the connection thread number.
     * @param connectionNumber The connection number of the thread
     */
    public static void initConnection(int connectionNumber) {
        log.info("Inside the init connection method for connectionNumber: {}", connectionNumber);
        PerConnectionData perConnectionData = new PerConnectionData();
        final Arena arena = Arena.ofShared();
        final MemorySegment memorySegment = arena.allocate(Integer.parseInt(System.getenv().getOrDefault("SEGMENT_SIZE_IN_BYTES", "10240")));
        final Fory fory = Fory.builder()
                .withLanguage(Language.XLANG) // As we will be sending it to rust, better to serialize in rust format
                .withCompatibleMode(CompatibleMode.COMPATIBLE)
                .withAsyncCompilation(true)
                .requireClassRegistration(true)
                .build();
        fory.register(ForyRequest.class, "com.google.gemini", "fory_request");
        // Warm up fory. It is ok if we take some time to warm up the JVM as the test has not started yet
        for (int i = 0; i < 1000; i++) {
            fory.serialize(new ForyRequest("default", "http://test.com/", "test.com", 80, "GET", "/", new int[]{200}, 1000L, true, Map.of("test", "value"), Map.of("test", "value"), "empty body"));
        }
        perConnectionData.setConnectionNumber(connectionNumber);
        perConnectionData.setArena(arena);
        perConnectionData.setMemorySegment(memorySegment);
        perConnectionData.setFory(fory);
        PER_CONNECTION_DATA.set(connectionNumber, perConnectionData);
    }
}
