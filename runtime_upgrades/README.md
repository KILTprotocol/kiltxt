TODO

* Feature gate did create extrinsics
* Feature gate migration extrinsics
* Feature gate metadata (default pre migration)
* Parse 
  * number of dids to spawn
  * action
  * endpoint
  * account mnemonic
* Complete Readme



Migration

* Takes Option<Vec<AccountId32>> as Input
    * If None, queries all account ids
    * Else executes migration for given AccountIDs
  * TODO: Support reading from file (maybe have list for successful migrations and non successful migrations)
  * TODO: Improve entire extrinsic, less manual magic
  * TODO: Conversion AccountId32 <> String
* Compare migrated/failed account ids with list of account ids
  * Execute `try_finalize` if done


Clappy
* Signer DID Accounts
* Signer migration extrinsic
* Input file failed migrations
* SUDO hex for set code
* WASM for set code