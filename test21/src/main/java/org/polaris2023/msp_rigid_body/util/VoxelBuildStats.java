package org.polaris2023.msp_rigid_body.util;

public record VoxelBuildStats(
        int cellCount,
        int solidCount,
        int selectedMode,
        int estimatedParts,
        int estimatedVertices,
        int estimatedTriangles,
        int sizeX,
        int sizeY,
        int sizeZ) {
}
