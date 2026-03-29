// CLI deploy hook with provider from Anchor.toml.

import * as anchor from "@coral-xyz/anchor";

module.exports = async function (provider: anchor.AnchorProvider) {
  // Use injected provider.
  anchor.setProvider(provider);

  // Add deploy logic.
};
