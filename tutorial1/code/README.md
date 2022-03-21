# 🚀 SCRT SIBYL CONTRACT

![scrt sibyl image](./images/logo_horizontal.png)

This is a secret contract in Rust to run in
[Secret Network](https://github.com/enigmampc/SecretNetwork).
To understand the framework better, please read the overview in the
[cosmwasm repo](https://github.com/CosmWasm/cosmwasm/blob/master/README.md),
and dig into the [cosmwasm docs](https://www.cosmwasm.com).


---
## :gear: Compile the Contract
---


### Execute Locally
_Note: this helper guide assumes you coded and compiled your Secret Contract in Rust._

if you never worked with rust, you will first need to install some tooling. The standard approach is to use _rustup_ to maintain dependencies and handle updating multiple versions of _cargo_ and _rustc_, which you will be using. First install a recent version of rust and cargo via [rustup](https://rustup.rs/).

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Next install the wasm32 target:

```sh
rustup default stable
cargo version
# If this is lower than 1.55.0+, update
rustup update stable

rustup target list --installed
rustup target add wasm32-unknown-unknown
```

Export the path to `cargo` if needed
```sh
export PATH="$HOME/.cargo/bin:$PATH"
```


Then install [cargo-generate](https://github.com/ashleygwilliams/cargo-generate).

```sh
cargo install cargo-generate --features vendored-openssl
```

Clone the repo:

```sh
git clone https://github.com/BalloonBox-Inc/SCRTSibyl-Contract.git  #OR https://github.com/scrtlabs/secret-template
```

Either in the main folder or in the src folder, run the following commands.
For testing:

```sh
cargo test
```

Note: add args `-- --nocapture` to debug tests

You'll need to override the Apple M1 default version causing dependencies issues and use the LLVM version of Clang instead. Run 

```sh
export AR=/opt/homebrew/cellar/llvm/13.0.1_1/bin/llvm-ar
export CC=/opt/homebrew/cellar/llvm/13.0.1_1/bin/clang-13
```

To build a wasm file:

```sh
cargo wasm
```

You'll find the built file in /target/wasm32-unknown-unknown/release/name_of_your_contract.wasm





---
## :link: Upload your Contract
---


_The next commands should all be executed from inside the CLI fodler of your project repo._

#### Requirements

- Yarn
- Node
- A compiled WASM smart contract

1. **Install dependencies**

   Run `yarn install` in the cli folder

2. **Generate keypairs**

   In the same folder run `yarn keypair` 
   
   This will generate a mnemonic + address and write it to a file called keys.json. Don't worry, these are just testnet keys.

3. **Add testnet funds to your keypair**

   Visit the faucent to get tokens: https://faucet.secrettestnet.io - enter your address from the keys.json file and request tokens.

4. **Verify the max_size and your compiled wasm contract path**

   Declare the following variables in the cli/index.ts file:
   ```sh
   const MAX_SIZE = 1000;
   const WASM = fs.readFileSync("../contract.wasm");
   ```

   Ensure your `.wasm` file is named `contract.wasm`, consistently with the readFileSync() function.
   
   Move the `contract.wasm` file in the cli folder.


5. **Upload and Initiate your contract**

   Run `yarn go` to upload and initiate your contract
   
   :fire: Congratulations! :fire: You've just uploaded a Secret Smart Contract to the Secret testnet.
   
 
6. **Interact with the Contract**

    ```sh
    npx ts-node keypair.ts # to generate a mnemonic and address pair
    npx ts-node index.ts 
    npx ts-node queryStats.ts # returns contract status
    npx ts-node setData.ts
    npx ts-node getData.ts # returns score query response
    ```
