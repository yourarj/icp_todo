# icp_todo

Welcome to your new icp_todo project.

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd icp_todo/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

## Testing app

You can test this todo app with

```bash
 dfx canister call icp_todo_backend create '(10:nat64, "Namaste")'
 dfx canister call icp_todo_backend get '(10:nat64)'
 dfx canister call icp_todo_backend update '(10:nat64, "Namaste & Hello")'
 dfx canister call icp_todo_backend fetch_all '(1:nat64, 10:nat64)'
 dfx canister call icp_todo_backend delete '(10:nat64)'
```
