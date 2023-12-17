## Logy
# Description
## Supply Chain Management Canister
 
 This Rust-based Internet Computer (IC) canister provides a basic supply chain management system.
 It includes functionality for managing farmers, products, wholesalers, and supply orders.
 Each entity **(farmer, product, wholesaler, supply order)** has associated CRUD operations.
 
 The canister utilizes the IC stable structures for memory management and BoundedStorable trait to define the maximum size of stored entities. It leverages thread-local storage for efficient and thread-safe management of memory, IDs, and storage structures.

 The canister defines various data structures for entities such as Farmer, Product, Wholesaler, and SupplyOrder, each implementing serialization and deserialization traits for storage purposes.

 ## Technologies Used
 - Rust


## icp_Logy_contract

### Requirements
* rustc 1.64 or higher
```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```
* rust wasm32-unknown-unknown target
```bash
$ rustup target add wasm32-unknown-unknown
```
* candid-extractor
```bash
$ cargo install candid-extractor
```
* install `dfx`
```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

```bash
$ cd icp_rust_boilerplate/
$ dfx help
$ dfx canister --help
```

## Update dependencies

update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:
```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## did autogenerate

Add this script to the root directory of the project:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

Update line 16 with the name of your canister:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

After this run this script to generate Candid.
Important note!

You should run this script each time you modify/add/remove exported functions of the canister.
Otherwise, you'll have to modify the candid file manually.

Also, you can add package json with this content:
```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

and use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```

**Main Functions**

- **add_farmer(payload: FarmerPayload) -> Option<Farmer>**
  Creates a new farmer based on the provided payload and adds it to the farmer storage. Returns the created farmer if successful.

- **add_product(payload: ProductPayload) -> Option<Product>**
  Creates a new product based on the provided payload and adds it to the product storage. Returns the created product if successful.

- **add_wholesaler(payload: WholesalerPayload) -> Option<Wholesaler>**
  Creates a new wholesaler based on the provided payload and adds it to the wholesaler storage. Returns the created wholesaler if successful.

- **add_supply_order(payload: SupplyOrderPayload) -> Option<SupplyOrder>**
  Creates a new supply order based on the provided payload and adds it to the supply order storage. Returns the created supply order if successful.

- **add_supply_order_wholesaler(payload: AddSupplyOrderWholesalerPayload) -> Result<SupplyOrder, Error>**
  Associates a wholesaler with an existing supply order identified by the given order ID. Returns the updated supply order if successful, otherwise returns a NotFound error if the supply order is not found.

- **complete_supply_order(id: u64) -> Result<SupplyOrder, Error>**
  Marks a supply order as complete based on the provided ID. Returns the completed supply order if successful, otherwise returns a NotFound error if the supply order is not found.

- **update_supply_order(id: u64, payload: SupplyOrderPayload) -> Option<SupplyOrder>**
  Updates the information of an existing supply order identified by the given ID with the provided payload. Returns the updated supply order if successful.

- **delete_supply_order(id: u64) -> Result<SupplyOrder, Error>**
  Deletes a supply order based on the provided ID. Returns the deleted supply order if successful, otherwise returns a NotFound error if the supply order is not found.
