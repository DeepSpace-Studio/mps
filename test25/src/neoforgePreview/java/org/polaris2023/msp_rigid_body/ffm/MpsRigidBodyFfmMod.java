package org.polaris2023.msp_rigid_body.ffm;

import net.neoforged.fml.common.Mod;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Mod(MpsRigidBodyFfmMod.MOD_ID)
public final class MpsRigidBodyFfmMod {
    public static final String MOD_ID = "mps_rigid_body";
    private static final Logger LOGGER = LoggerFactory.getLogger(MpsRigidBodyFfmMod.class);

    public MpsRigidBodyFfmMod() {
        LOGGER.info("mps_rigid_body Java 25 FFM NeoForge preview loaded");
    }
}
