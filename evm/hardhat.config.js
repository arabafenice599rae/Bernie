// Hardhat 3 esegue gli stessi test Solidity di Foundry (cheatcode vm.* comprese):
// serve per provarli in locale dove Foundry non è installabile. La CI usa Foundry.
import { defineConfig } from "hardhat/config";

export default defineConfig({
  solidity: {
    version: "0.8.30",
    settings: { evmVersion: "cancun", optimizer: { enabled: true, runs: 200 } },
  },
  paths: { sources: "src", tests: { solidity: "test" } },
  test: { solidity: { fsPermissions: { readDirectory: ["../vectors"], readFile: ["../vectors/scale_1e18.flat.json"] } } },
});
