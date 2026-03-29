import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BondingCurve } from "../target/types/bonding_curve";

describe("bonding_curve", () => {
  // Use local cluster provider.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.bondingCurve as Program<BondingCurve>;

  it("Is initialized!", async () => {
    // Add test logic.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
