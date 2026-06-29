/**
 * Vitest setup — wire full compile-time checker host for integration tests.
 * @module
 */

import { setCompileCheckerHost } from "../src/compile.js";
import { createFullCheckerHost } from "../src/cli/checker-host.js";
import { setDeployCertificationProver } from "../src/deploy-service.js";
import { defaultCertificationProver } from "../src/cli/certify-bridge.js";
import { setDefaultHardwareVerifyHost } from "../src/hardware-verify.js";
import { createHardwareVerifyHost } from "../src/cli/hardware-verify-bridge.js";

setCompileCheckerHost(createFullCheckerHost());
setDeployCertificationProver(defaultCertificationProver);
setDefaultHardwareVerifyHost(createHardwareVerifyHost());
