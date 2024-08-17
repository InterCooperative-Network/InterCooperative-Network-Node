# Changelog
All notable changes to this project will be documented in this file.
## [Unreleased]

### Other Changes

- Reorganized repository structure and moved crates to icn_repos directory (7050306)

- Fixed and re-initialized icn_api submodule (26254f0)

- Add icn_dao as submodule (d4463ee)

- Enter your commit message (end with an empty line): (2315b1b)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received.
test (05a547d)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received. (1d5d06f)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received.
Key improvements and additions:

Comprehensive error handling using IcnResult and IcnError.
Asynchronous methods for better performance and scalability.
Improved transaction verification process.
Enhanced proposal lifecycle management.
Implementation of cross-shard transactions.
Integration of ZKP (Zero-Knowledge Proof) functionality.
Extensive test suite covering various node operations. (21684fc)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received. (8d18427)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received. (edcfd20)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received.
Proof of Work Removal: All references to PoW, including mining difficulty and related methods, have been removed from the blockchain implementation.
Integration with Consensus: The blockchain is now fully integrated with the Proof of Cooperation (PoC) consensus mechanism, which is energy-efficient and better suited to the project's goals.
Updated Tests: Test cases have been updated to reflect the removal of PoW and to test the new PoC-based blockchain operations.
Network Updates: The network module has been revised to work with the new consensus model and maintain efficient peer-to-peer communication and gossip protocol. (1031884)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received.
Enhance blockchain integration with consensus and improve documentation

    Improved interaction between blockchain and consensus module.
    Added comprehensive comments and documentation throughout the code.
    Prepared the code for future implementation of signature verification.
    Refined block and transaction validation processes.
    Verified functionality through updated and comprehensive tests. (c760a6a)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received.
Enhance consensus mechanism with improved blockchain integration and documentation

    Improved synchronization between consensus mechanism and blockchain.
    Enhanced error handling and validation processes within consensus logic.
    Added detailed documentation for all functions and key areas of the code.
    Expanded and refined test cases to ensure robustness and correct functionality.
    Ensured compatibility and smooth integration with the blockchain module. (755aa7d)

- Added icn_common as a submodule (906d322)

- Removed icn_dao submodule from /opt/InterCooperative-Network-Node/icn_dao (5632303)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received.
added common back (9eaab70)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.):
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
Commit message received.
Refactor blockchain implementation and improve modularity

- Migrated core blockchain functionality to blockchain.rs, consolidating
  block and transaction management, chain validation, and consensus integration.
- Updated lib.rs to serve as the main entry point, initializing the blockchain
  and consensus mechanisms, and referencing the newly modularized components.
- Created asset_tokenization.rs for handling asset tokenization within the blockchain.
- Created transaction_validator.rs to handle transaction validation, including
  checks for double spends, sufficient balances, and signature verification.
- Retained and enhanced all original functionality, including signature handling
  and consensus operations, while improving code organization and modularity.
- Added comprehensive tests across all modules to ensure reliability and coverage.

Breaking change: The structure of the blockchain module has been significantly
refactored. This will require updates to any dependent modules or external
references to the blockchain functionality. (2c3b9e2)

- Please enter your commit message below.
Type your message and press Enter. To finish, enter a line with only a period (.) or just press Enter:
-------- BEGIN COMMIT MESSAGE --------
-------- END COMMIT MESSAGE --------
test (32a980e)

- moving some code to common (9c64c6c)

- Remove incorrect submodule clone for icn_core (a4ba1b0)

- commit (8855f88)

- Correctly add icn_core as a submodule (9f9f55f)

- added mock and simulation files to blockchain (bf45e64)

- Update submodule references (5b378fb)

- Removed submodule directories to prepare for subtree integration (7a916a8)

- Squashed 'icn_repos/icn_identity/' content from commit 87ce81e

git-subtree-dir: icn_repos/icn_identity
git-subtree-split: 87ce81e85bb743d5ba8ea32be611a52ab632b1e5 (babf614)

- Merge commit 'babf6148cb6ceb13916db9781880edc303b91a81' as 'icn_repos/icn_identity' (1c7aaaf)

- Squashed 'icn_repos/icn_blockchain/' content from commit ab9edb1

git-subtree-dir: icn_repos/icn_blockchain
git-subtree-split: ab9edb10a4d06b6cca6fd0a1837bbf765695319f (1c3a511)

- Merge commit '1c3a511835a773cb27a94d3ca7e1f7a355a3af2d' as 'icn_repos/icn_blockchain' (063c6b0)

- Squashed 'icn_repos/icn_consensus/' content from commit bf70c92

git-subtree-dir: icn_repos/icn_consensus
git-subtree-split: bf70c925140b2667f7f7ad4691fb74dc8ff2b064 (e738434)

- Merge commit 'e7384343530ee5fb93417c4909b6a1a6077d101d' as 'icn_repos/icn_consensus' (b553e58)

- Squashed 'icn_repos/icn_currency/' content from commit 1db2b6e

git-subtree-dir: icn_repos/icn_currency
git-subtree-split: 1db2b6e511aaa45700708c4e42fb0732d1255fda (471f8c3)

- Merge commit '471f8c3f8320c92fe71cc1e5b778a5421e1dd229' as 'icn_repos/icn_currency' (0be0cf9)

- Squashed 'icn_repos/icn_demo/' content from commit a3e31d7

git-subtree-dir: icn_repos/icn_demo
git-subtree-split: a3e31d7d4e2e229e49b17407078d6a09c76be03d (c90e03b)

- Merge commit 'c90e03be8efab31a54123db1eb2412101b0457da' as 'icn_repos/icn_demo' (749c2b8)

- Squashed 'icn_repos/icn_federation/' content from commit e75e4ff

git-subtree-dir: icn_repos/icn_federation
git-subtree-split: e75e4ff5733f7511e5fa9a877ac9c1897c917fd5 (19a0486)

- Merge commit '19a04866140c88c836cb2b25cf3abfc1c7a3ac98' as 'icn_repos/icn_federation' (05e0764)

- Squashed 'icn_repos/icn_governance/' content from commit 11d8235

git-subtree-dir: icn_repos/icn_governance
git-subtree-split: 11d82355821aefd0d4bd342df214788e8c31723f (994c729)

- Merge commit '994c7290c07bd0809f0d7575a2a67d032131bac3' as 'icn_repos/icn_governance' (ca686ce)

- Squashed 'icn_repos/icn_incentives/' content from commit d98343d

git-subtree-dir: icn_repos/icn_incentives
git-subtree-split: d98343df093c0c3303c50919cde995fa088dc63a (61210fa)

- Merge commit '61210fa6c62a6476986ffd2f5a084883f5ab45c5' as 'icn_repos/icn_incentives' (1f7a02b)

- Squashed 'icn_repos/icn_language/' content from commit ce15397

git-subtree-dir: icn_repos/icn_language
git-subtree-split: ce153977197077ecd3dbbb2b7b81ea89e06ebfa3 (3b21cd0)

- Merge commit '3b21cd0f2099bd727c1665f6745c3ae5856da546' as 'icn_repos/icn_language' (dee6528)

- Squashed 'icn_repos/icn_market/' content from commit 1d84b3a

git-subtree-dir: icn_repos/icn_market
git-subtree-split: 1d84b3a4425c9d0f04dc22a8cb9a87ef6a745a5a (def1e45)

- Merge commit 'def1e452b1c9a8bc87eb40cdab5e474cf19b6025' as 'icn_repos/icn_market' (c85e291)

- Squashed 'icn_repos/icn_network/' content from commit 11dabdf

git-subtree-dir: icn_repos/icn_network
git-subtree-split: 11dabdf2de708132a4fd5d0e307caae97646a5db (fd7c9db)

- Merge commit 'fd7c9dbd618f0df7b403cff53bdef7b99796e259' as 'icn_repos/icn_network' (f423859)

- Squashed 'icn_repos/icn_node_management/' content from commit 55d5bdd

git-subtree-dir: icn_repos/icn_node_management
git-subtree-split: 55d5bdd31f274949d6be997a44d2234833cbee3d (89a39d2)

- Merge commit '89a39d29706987454810573d67b2aa648f3a94c8' as 'icn_repos/icn_node_management' (bcf2591)

- Squashed 'icn_repos/icn_reputation/' content from commit 93e1cd5

git-subtree-dir: icn_repos/icn_reputation
git-subtree-split: 93e1cd54f25241a8c5f01f61aa46fe82777edf6a (ad088cb)

- Merge commit 'ad088cb9035407b9ed312ffd8d2a83950fb2d64d' as 'icn_repos/icn_reputation' (97862ea)

- Squashed 'icn_repos/icn_sharding/' content from commit d91612b

git-subtree-dir: icn_repos/icn_sharding
git-subtree-split: d91612b62268eedbc1de3c968b0e5b9a9d37269a (179ac91)

- Merge commit '179ac912b96163aa8507af9d805a15fb6a2788d9' as 'icn_repos/icn_sharding' (93b3946)

- Squashed 'icn_repos/icn_smart_contracts/' content from commit 5b287e3

git-subtree-dir: icn_repos/icn_smart_contracts
git-subtree-split: 5b287e36a6f077aff0b91fa2fff4f917ed8e5cb8 (35cebc2)

- Merge commit '35cebc241d3d38c60743d88216bd438ddd58791e' as 'icn_repos/icn_smart_contracts' (8a6bd32)

- Squashed 'icn_repos/icn_storage/' content from commit e427a7c

git-subtree-dir: icn_repos/icn_storage
git-subtree-split: e427a7c1eacbf41c94912cb4bef4e3a9c6d8564e (58fc26f)

- Merge commit '58fc26f8e8c52ec2d3e69f05201d76b82d8a213e' as 'icn_repos/icn_storage' (742a178)

- Squashed 'icn_repos/icn_testnet/' content from commit 5c88499

git-subtree-dir: icn_repos/icn_testnet
git-subtree-split: 5c88499839843266b300b5b7367cd3bb4978c7fe (4097811)

- Merge commit '40978115d114220469e14ff216cbc4e0cae74db1' as 'icn_repos/icn_testnet' (3f0a663)

- Squashed 'icn_repos/icn_utils/' content from commit 1c5313a

git-subtree-dir: icn_repos/icn_utils
git-subtree-split: 1c5313a991c86ea894cd7e078dae066e1e9fcb94 (0478dff)

- Merge commit '0478dff3130e7b2e85f56168cc58bdc3c7dde7b3' as 'icn_repos/icn_utils' (dba967c)

- Squashed 'icn_repos/icn_vm/' content from commit 47732a0

git-subtree-dir: icn_repos/icn_vm
git-subtree-split: 47732a0aa7092b1c26d26242df9a496ac268952e (ff89fea)

- Merge commit 'ff89fea7651fc9d4c72a6295f45cc5a17be39d15' as 'icn_repos/icn_vm' (0ba209e)

- Squashed 'icn_repos/icn_zkp/' content from commit 57561b5

git-subtree-dir: icn_repos/icn_zkp
git-subtree-split: 57561b507bbbc5809f56f4ec8340bf69a9dcc614 (f53207c)

- Merge commit 'f53207c199871644b00cfb6b1609e44300f2f689' as 'icn_repos/icn_zkp' (0de3734)

- Squashed 'icn_repos/icn_api/' content from commit e60f843

git-subtree-dir: icn_repos/icn_api
git-subtree-split: e60f843961482106b29217407f94ea9910e269ed (d185923)

- Merge commit 'd185923990b7cada193999677752ff28c443d743' as 'icn_repos/icn_api' (4d06a7e)

- Squashed 'icn_repos/icn_dao/' content from commit 6d3cc0d

git-subtree-dir: icn_repos/icn_dao
git-subtree-split: 6d3cc0d21dcd5208bcdff4edf6ab7805fce864a6 (8aaca15)

- Merge commit '8aaca1541bca43e1fe463b49e611aeb20709203e' as 'icn_repos/icn_dao' (5e2d0b6)

- Squashed 'icn_repos/icn_common/' content from commit bf40eb8

git-subtree-dir: icn_repos/icn_common
git-subtree-split: bf40eb8b57a1b8aeb7f0251fedc62222df70e79d (5d0198d)

- Merge commit '5d0198db6ced866d8548d764ebb6561d119bdd90' as 'icn_repos/icn_common' (aa78900)

- Squashed 'icn_repos/icn_core/' content from commit c180386

git-subtree-dir: icn_repos/icn_core
git-subtree-split: c1803865ac697488a4581680ca835302c01d2260 (605ccf9)

- Merge commit '605ccf9a77efbf323166de6208d1a7efefe66dac' as 'icn_repos/icn_core' (6e54d84)


### Testing

- test (254428d)

## [de] - 2024-08-14

### Bug Fixes

- fixing errors (6caf241)


### Other Changes

- Initial commit (2ab2e42)

- Initial project setup with modular structure (68fe36e)

- added update script (9614841)

- Moved all files to new project folder (abb0c1b)

- Moving files (78c033d)

- Continue Refactoring (b7bd921)

- working through errors after refactor (96abc3d)

- Removed icn_smart_contract folder (4f122fd)

- Changed toml file? (1b0d460)

- Updating and clearing errors (77b162e)

- Fixing icn_blockchain... Still have errors (256c0a1)

- Fixing errors (7c84b15)

- working on clearing errors from icn_blockchain (3982b14)

- Enter your commit message:
testing script (0331643)

- Enter your commit message:
testing script (bc27c77)

- Enter your commit message:
Testing (928ed0e)

- Enter your commit message:
Fixing errors (885d6c1)

- Enter your commit message:
added icn_utils (72edaf8)

- Enter your commit message:
Modified icn_blockchain/src/lib.rs, added tests (e647cfa)

- Enter your commit message:
Fixing errors (02eadb9)

- Enter your commit message:
removed icn_common, moved code into icn_util. (ba76494)

- Enter your commit message:
working to fix cyclic dependency (a9f4758)

- Enter your commit message:
Refactoring code with sonnet 3.5 (2b3a634)

- Enter your commit message:
Refactored all of the lib files and testnet main.rs (f259616)

- Enter your commit message:
Correcting .toml errors in crates (32bebbb)

- Enter your commit message:
Updated icn_types to provide common types and error handling. (277b99f)

- Enter your commit message:
rewriting the icn_core/src/lib.rs file (d8dfaba)

- Enter your commit message:
Fixing the core crate and implementing main.rs (2fda856)

- Enter your commit message:
accidently introduced cyclic dependency... need to fix (b63c3d4)

- Refactor to resolve cyclic dependencies by utilizing icn_common crate (db7e1ee)

- Enter your commit message:
Detangling the crates... wip. (85ed701)

- Enter your commit message:
Continuing to refactor... (7731c56)

- Enter your commit message:
Expanded crates/blockchain/lib.rs (319c8bf)

- Enter your commit message:
Fixing errors (16cbc4a)

- Enter your commit message:
Still fixing circular dependency (d0a7d62)

- Enter your commit message:
refactored the icn_types, icn_utils, and icn_common crates (34b93a8)

- Enter your commit message:
fix utils and types (a8d0182)

- Enter your commit message:
fixing lib files... getting closer (b6f351b)

- Enter your commit message:
fixing api (aabcb8e)

- Enter your commit message:
Renamed types common_types (197305a)

- Enter your commit message:
Refactored storage lib, cross shard com, cross shard sync, shard cargo, api cargo (6596739)

- Enter your commit message:
consolidated common_types and common crates (18d3c12)

- Enter your commit message:
Almost done refactoring... (364336d)

- Enter your commit message:
purging any lingering common_types (30937d5)

- Enter your commit message:
Almost done fixing errors... so close (8c6cb34)

- Enter your commit message:
almost... (b220678)

- Enter your commit message:
refactored common, blockchain, consensus, currency, governance, crates (8a1991b)

- Enter your commit message:
Refactored network and sharding (2b0300c)

- Enter your commit message:
Fixed storage and VM, need to update crates and integrate into blockchain and consensus (dbe911d)

- Enter your commit message:
modified core (7e12a74)

- Enter your commit message:
added zkp and smart_contract crates, added example file contract (3c413e7)

- Enter your commit message:
added reputation crate. moved new crates into crate folder (3c9274d)

- Enter your commit message:
added new crates, added quadratic voting system (6a65595)

- Enter your commit message:
expanded smart contract lib (921246a)

- Enter your commit message:
Improved cross crate functions within core lib (c82e0e7)

- Enter your commit message:
improved api (f87732c)

- Enter your commit message:
modified api again, fixes cargo files (3d40d36)

- Enter your commit message:
fixed src blockchain.rs, bft_poc, and api (e373077)

- Enter your commit message:
Making consistant error method. (b759007)

- Enter your commit message:
fixed common lib (1ab9335)

- Improved error handling and trait implementations

- Updated error handling and propagation in 'icn_blockchain/src/blockchain.rs' to consistently use 'IcnResult<T>' and the '?' operator.
- Enhanced trait implementations and interfaces in 'icn_api/src/lib.rs' for better flexibility and extensibility.
- Added comprehensive comments and documentation for improved code readability and maintainability.
- Ensured adherence to Rust's ownership and borrowing rules to avoid runtime errors.
- Optimized performance by minimizing unnecessary clones and using iterators. (a4e7985)

- Enhanced documentation, improved error handling, defined comprehensive traits and interfaces, enhanced unit tests, and ensured concurrency and thread safety. (cb72117)

- Enter your commit message:
feat: Enhance ICN Project with Improved API, Transaction Validation, Blockchain Implementation, and Consensus Mechanism (575b782)

- Enter your commit message (end with an empty line): (8e69e79)

- Enter your commit message (end with an empty line):
Standardized error handling using IcnResult and IcnError across all modules.
Added comprehensive doc comments for all functions, methods, and modules.
Expanded unit tests to cover more edge cases and scenarios.
Ensured proper use of synchronization primitives for shared mutable state.
Optimized memory management by reducing unnecessary clones and using iterators.
Implemented security best practices and optimized critical code paths for performance. (54d55bc)

- Enter your commit message (end with an empty line):
Standardized error handling in network module using IcnResult and IcnError.
Added comprehensive doc comments to all functions, methods, and modules in network.rs.
Ensured proper use of synchronization primitives for thread safety.
Expanded unit tests to cover network start and stop scenarios. (2bd3997)

- Enter your commit message (end with an empty line):
Standardized error handling in the networking module using IcnResult and IcnError.
Added comprehensive doc comments to all functions, methods, and modules in discovery.rs, protocol.rs, routing.rs, security.rs, naming.rs, and packet.rs.
Ensured proper use of synchronization primitives for thread safety.
Expanded unit tests to cover start and stop scenarios for node discovery, network protocol, router, network security, naming service, and packet validation. (749576d)

- Enter your commit message (end with an empty line):
Standardized error handling in the storage module using IcnResult and IcnError.
Added comprehensive doc comments to all functions, methods, and modules in lib.rs, storage_manager.rs, and storage_node.rs.
Ensured proper use of synchronization primitives for thread safety.
Expanded unit tests to cover adding/removing nodes and storing/retrieving data. (83f3068)

- Enter your commit message (end with an empty line):
Standardized error handling in the node management module using IcnResult and IcnError.
Added comprehensive doc comments to all functions, methods, and modules in lib.rs, content_store.rs, fib.rs, icn_node.rs, and pit.rs.
Ensured proper use of synchronization primitives for thread safety.
Expanded unit tests to cover adding/removing entries and processing interests. (526c77d)

- Enter your commit message (end with an empty line):
Refactored icn_api module for better error handling and code consistency. Improved documentation and expanded tests for comprehensive coverage. Standardized coding style and reduced redundancy. (9fa08d2)

- Enter your commit message (end with an empty line):
Refactored icn_blockchain module for consistent error handling and reduced redundancy. Improved documentation and expanded tests for better coverage. Standardized coding style and error handling using IcnError. (9dd4ad2)

- Enter your commit message (end with an empty line):
Refactored icn_blockchain/transaction_validator module for consistent error handling and reduced redundancy. Improved documentation and expanded tests for better coverage. Standardized coding style and error handling using IcnError. (122d6ef)

- Enter your commit message (end with an empty line):
Refactored icn_blockchain/lib module for consistent error handling and reduced redundancy. Improved documentation and expanded tests for better coverage. Standardized coding style and error handling using IcnError. (575195a)

- Refactor code, implement centralized error handling, establish coding style, and add Proof of Contribution consensus mechanism. (ac535a2)

- Enter your commit message (end with an empty line):
Implement core structure for ICN domain-specific language (77be784)

- Enter your commit message (end with an empty line):
Integrate icn_language crate and update dependencies (7272975)

- Enter your commit message (end with an empty line):
significant update (076bec5)

- Enter your commit message (end with an empty line):
 BitVec and bit manipulation utilities (ed05eef)

- Enter your commit message (end with an empty line):
modified blockchain and consensus (1c44f6f)

- Enter your commit message (end with an empty line): (ca57a1c)

- Enter your commit message (end with an empty line):
deleted vendor (0dbb55b)

- Enter your commit message (end with an empty line):
Modified language/vm (b563f75)

- Enter your commit message (end with an empty line):
Further developing Language (d13c4a5)

- Enter your commit message (end with an empty line):
- Implemented the complete icn_language crate, including parsing, compiling, and bytecode generation.- Implemented the complete icn_vm crate, including the VM execution logic and tests.- Ensured proper integration between icn_language and icn_vm.- Added comprehensive unit tests for both crates. (2fe56ec)

- Enter your commit message (end with an empty line):
Starting to prep demo (386843a)

- Enter your commit message (end with an empty line):
Stuck with dependency issue... (10c8d88)

- Enter your commit message (end with an empty line):
Replaced bellman with bulletproofs (e55b7e9)

- Enter your commit message (end with an empty line):
Improve ZKP implementation with Bulletproofs (bb79bf5)

- Enter your commit message (end with an empty line):
Working out cirecular dependency (90a2a46)

- Enter your commit message (end with an empty line): (2a7363e)

- Enter your commit message (end with an empty line):
Refactored ICN project to improve code quality and address identified issues:- Enhanced ownership and borrowing rules by using Arc<Mutex<T>> correctly.- Improved error handling by replacing unwrap() with proper error propagation using Result and ? operator.- Implemented traits for transaction validation to reduce code duplication and ensure consistency.- Improved concurrency handling by using appropriate synchronization primitives.- Refactored code to use idiomatic Rust practices, including efficient memory management and pattern matching.- Standardized coding style and naming conventions across all files.- Added comprehensive documentation for functions, methods, and modules using rustdoc.Tested all changes thoroughly to ensure system stability and correctness. (218dc26)

- Enter your commit message (end with an empty line):
added demo crate (d51b9fc)

- Enter your commit message (end with an empty line):
added icn_dao crate (5f4dec0)

- Enter your commit message (end with an empty line):
added code to dao crate (2f248aa)

- Enter your commit message (end with an empty line):
Preparing demo (53fc6fb)

- Enter your commit message (end with an empty line):
fixing errors, refactoring demo (0b7b05b)

- Enter your commit message (end with an empty line):
Refactor and improve the ICN project:- Standardized error handling using `IcnError` and `IcnResult`.- Consolidated transaction validation logic into `TransactionValidator` trait.- Removed `unwrap` and `expect` to prevent panics and enhance error handling.- Improved synchronization efficiency by using `tokio::sync::RwLock`.- Added comprehensive documentation across multiple files. (c65876e)

- Enter your commit message (end with an empty line):
down to 12 problems (1674a59)

- Enter your commit message (end with an empty line):
fixing more errors (573f6bf)

- Enter your commit message (end with an empty line):
more errors (06f8390)

- Enter your commit message (end with an empty line): (ce89eb8)

- Enter your commit message (end with an empty line):
Writing update for text file. (5852e13)

- Enter your commit message (end with an empty line):
working on demo. (1a710b0)

- Enter your commit message (end with an empty line):
made significant improvements to the `icn_core/src/lib.rs` and `icn_demo/src/main.rs` files. (5f22179)

- Enter your commit message (end with an empty line):
futher linting project. Working on governance crate (b0ec2de)

- Enter your commit message (end with an empty line): (2884475)

- Enter your commit message (end with an empty line):
Fixing demo (1e12a73)

- Enter your commit message (end with an empty line): (26a1fdc)

- Enter your commit message (end with an empty line):
demo main and core lib (4c1ea0d)

- Enter your commit message (end with an empty line):
core lib and demo main (2e06c94)

- Enter your commit message (end with an empty line):
fix api lib (babfeca)

- Enter your commit message (end with an empty line):
added code to demo main (ed3a4a8)

- Enter your commit message (end with an empty line): (c465802)

- Enter your commit message (end with an empty line):
Add all required methods to IcnNode struct (0fe36b9)

- Enter your commit message (end with an empty line):
updated blockchain and consensus. (f1ebcdf)

- Enter your commit message (end with an empty line): (2c0a27e)

- Enter your commit message (end with an empty line):
main and core (d10b6e8)

- Enter your commit message (end with an empty line):
fixing other crates (a7d9ecc)

- Enter your commit message (end with an empty line): (83bb939)

- Enter your commit message (end with an empty line):
update blockchain lib (b53cf69)

- Enter your commit message (end with an empty line):
Worked on currency and blockchain (443af97)

- Enter your commit message (end with an empty line):
Update blockchain and currency (c6a7878)

- Enter your commit message (end with an empty line):
Fixed blockchain and common (6cf2403)

- Enter your commit message (end with an empty line):
Completed IcnNode implementation and test suite (91c8ec9)

- Enter your commit message (end with an empty line):
fixed sharding (140cb09)

- Enter your commit message (end with an empty line):
Improve and expand icn_sharding crate (024c529)

- Enter your commit message (end with an empty line):
added comments to sharding manager (501b6ee)

- Enter your commit message (end with an empty line): (5d0d2c0)

- Enter your commit message (end with an empty line): (3545324)

- Enter your commit message (end with an empty line): (b249ccc)

- Enter your commit message (end with an empty line):
The `ShardingManager` implementation has been thoroughly revised and expanded (18bd3c5)

- Enter your commit message (end with an empty line):
Testnet and core (91758d9)

- Enter your commit message (end with an empty line):
storage and sharding (6b1ff9e)

- Enter your commit message (end with an empty line): (87bf0cd)

- Enter your commit message (end with an empty line):
update readme (5a8ac8d)

- Enter your commit message (end with an empty line):
Refactored transaction validation in icn_blockchain to enhance security and robustness. Updated consensus algorithm in icn_consensus to handle edge cases more effectively. Improved test coverage with comprehensive unit tests. (fe1fc55)

- Enter your commit message (end with an empty line):
Refactored transaction validation logic to be centralized and consistent across modules. Enhanced error handling to cover more cases within the ICN project. Maintained all existing functionality while improving code quality. (8b97566)

- Enter your commit message (end with an empty line):
Refactor `transaction_validator.rs` to improve signature verification and error handling.- Implemented real signature verification using `ed25519_dalek`, ensuring that transactions are authenticated properly.- Improved error handling by adding checks for missing signatures or public keys, reducing the chances of runtime errors.- Optimized the double-spend check by keeping the existing logic while making minor improvements for clarity.- Ensured that all transaction validation steps (double-spend, currency/amount, balance, signature, timestamp) are robust and comprehensive.- Updated tests to reflect the changes and ensure all functionalities are intact.These changes maintain the original functionality while enhancing the security and reliability of transaction validation within the blockchain. (96e1d56)

- Enter your commit message (end with an empty line):
Enhanced error handling in the API layer to provide detailed error information. Improved transaction validation logic, including better double-spend detection. Added comprehensive tests and validations to the consensus algorithm to ensure robustness. Reviewed concurrency patterns and ensured safe use of Arc<RwLock<...>>. Expanded documentation and increased test coverage to improve code maintainability and reliability. (5ad8acf)

- Enter your commit message (end with an empty line):
Refactor ICN API lib.rs to improve error handling, consistency, and async usage.- Updated error handling to use `warp::Rejection` properly.- Ensured consistent async/await usage across all API handlers.- Fixed missing imports and standardized error propagation.- Improved test coverage for API functions. (1b9df88)

- Enter your commit message (end with an empty line):
Refactor and enhance the icn_blockchain crate.- Consolidated transaction validation into a single module (transaction_validator.rs).- Improved block creation and validation processes in blockchain.rs.- Added thread safety mechanisms and optimized balance calculations.- Removed redundant code and ensured consistency across the crate. (e26a4b9)

- Enter your commit message (end with an empty line):
Enhance icn_common crate with improved error handling, bit utilities, and shared structures.- Expanded IcnError enum for more descriptive error messages.- Optimized bit manipulation utilities in bit_utils.rs with additional methods.- Improved shared utilities in lib.rs, including better transaction and proposal handling.- Added unit tests to ensure robustness of new features. (9aa5210)

- Enter your commit message (end with an empty line): (6c68258)

- Enter your commit message (end with an empty line):
Implemented and refined blockchain and bit manipulation utilities- **Blockchain Implementation (`lib.rs`)**  - Added a comprehensive blockchain structure with features such as block creation, transaction management, mining with proof-of-work, and validation.  - Implemented Merkle root calculation to ensure the integrity of transactions within each block.  - Included mechanisms for validating transactions and ensuring the blockchain's integrity.  - Developed methods to manage currency balances using a mock currency system.  - Added unit tests covering block creation, transaction handling, mining, and blockchain validity to ensure robust functionality.- **Bit Manipulation Utilities (`bit_utils.rs`)**  - Created a custom BitVec structure to manage a vector of bits, including operations to set, clear, toggle, and count bits.  - Implemented utility functions for manipulating bits in a u64 value, including setting, clearing, toggling, and rotating bits.  - Developed unit tests to cover all bit vector operations and utility functions, ensuring correctness and handling edge cases.This commit enhances the project's core functionality by providing essential blockchain features and utilities for bit-level operations. The added tests ensure the reliability and integrity of the codebase. (df7fcd8)

- Enter your commit message (end with an empty line):
added license.md (24ca1a9)

- Enter your commit message (end with an empty line):
added static pages (4fe7aeb)

- Enter your commit message (end with an empty line):
Refactored icn_api crate to improve error handling, optimize ownership/borrowing patterns, and standardize coding style. Enhanced test coverage with additional unit tests for critical API functions. Fixed potential issues with Arc<RwLock<>> usage to avoid unnecessary locking and potential deadlocks. (e8ca698)

- Create static.yml (f7a3e75)

- Enter your commit message (end with an empty line):
core lib (f715372)

- Enter your commit message (end with an empty line):
Refactor and expand icn_api/src/lib.rs (628c453)

- Enter your commit message (end with an empty line):
Improve and complete blockchain implementationComplete add_block function to properly add new blocks and update balancesAdd get_latest_block function for retrieving the most recent blockFully implement is_chain_valid function for robust blockchain validationAdd helper functions get_block_by (45c9b34)

- Enter your commit message (end with an empty line):
Refactored project structure and code base to improve error handling, ownership and borrowing practices, and overall code organization. Added new features including proposal status retrieval and blockchain enhancements. Expanded test coverage and improved documentation. Addressed thread safety and performance optimization in critical areas. (a779402)

- Enter your commit message (end with an empty line): (5ba29dd)

- Enter your commit message (end with an empty line):
Refactored and completed PoCConsensus implementation:- Enhanced logging across block validation, transaction validation, and consensus processes.- Improved error handling with more context and consistent logging.- Added checks to prevent duplicate validators in the consensus mechanism.- Refined validation logic for blocks and transactions.- Updated and expanded unit tests to cover edge cases and ensure robust functionality. (d83ce8b)

- Enter your commit message (end with an empty line):
Refactored and enhanced the CurrencySystem implementation:- Improved error handling and validation in minting, burning, and transferring currency.- Added detailed comments and documentation for better code understanding.- Enhanced the CurrencySystem logic to prevent duplicate currencies and handle edge cases.- Expanded the test suite to include edge cases and comprehensive coverage of currency operations. (f262a1e)

- Enter your commit message (end with an empty line):
Implement currency and governance system improvementsComplete mint, burn, and transfer functions in CurrencySystemAdd create_proposal, get_proposal, and vote_on_proposal functions to GovernanceSystemImplement finalize_proposal and mark_as_executed functionsAdd comprehensive test suites for both currency and governance systemsEnsure proper error handling and input validationImplement proposal quorum and majority checksAdd functions to retrieve votes and proposal results (de971bc)

- Enter your commit message (end with an empty line):
Enhance identity management and networking functionalityComplete implementation of IdentityService in icn_identity/src/lib.rsAdd create_identity, get_identity, and update_reputation functionsImplement proper error handlingAdd unit tests for new functionalityImprove NetworkManager in icn_network/src/lib.rsComplete start function implementationAdd broadcast_transaction functionImplement connect_to_peer and handle_connection functionsAdd comprehensive error handlingImplement unit tests for peer connections, message broadcasting, and disconnections (186e894)

- Enter your commit message (end with an empty line):
A fully implemented store_data function that handles data storage across multiple nodes.A new retrieve_data function to fetch stored data from the nodes.A complete remove_data function to handle data deletion across nodes.Additional helper functions like get_node_count and get_data_distribution.Comprehensive unit tests to cover various scenarios and edge cases. (b749865)

- Enter your commit message (end with an empty line):
icn_sharding (2fd5c59)

- Enter your commit message (end with an empty line):
"Enhance icn_storage crate with improved functionality and error handling- Add key_exists function to check for key presence- Implement get_total_storage_size for storage size calculation- Add get_key_count function to retrieve number of stored keys- Implement list_keys function to retrieve all stored keys- Expand test suite to cover new functionality and edge cases- Improve error handling using IcnResult and IcnError- Enhance documentation for better code understanding" (0fd5e1c)

- Enter your commit message (end with an empty line):
Contract deploymentContract executionState managementContract querying and removal (899d07a)

- Enter your commit message (end with an empty line):
zkp (b50eb8a)

- Enter your commit message (end with an empty line):
market additions (d70d346)

- Enter your commit message (end with an empty line):
update email in static page (b5be366)

- Enter your commit message (end with an empty line):
update funding (4aae8b2)

- Enter your commit message (end with an empty line): (0942822)

- Enter your commit message (end with an empty line): (a552007)

- Enter your commit message (end with an empty line): (5550db1)

- Enter your commit message (end with an empty line): (c1d8f9d)

- Enter your commit message (end with an empty line): (0923d47)

- Enter your commit message (end with an empty line):
Expand API functionality and improve error handlingAdd new API endpoints:Retrieve block information by hash or heightGet current network difficultySubmit new smart contractsExecute smart contractsImplement proper error handling for all API endpointsReturn appropriate HTTP status codes and error messagesUpdate ApiLayer struct with new methodsUpdate web server implementation to include new routesAdd comprehensive documentation and unit tests for new functionalityImprove overall code quality and modularity (5762fbc)

- Enter your commit message (end with an empty line):
Enhance Blockchain implementation with improved block validation and fork handling- Add timestamp validation in add_block method- Implement transaction validation in add_block method- Add Merkle root validation in add_block method- Implement handle_fork method for managing blockchain forks- Add helper methods for fork handling (is_valid_chain, find_fork_point, rollback_transactions, apply_transactions)- Update and expand test cases to cover new functionality- Add test case for handle_fork method (9ef6a89)

- Enter your commit message (end with an empty line):
Implement block proposal and validation in PoCConsensus- Add propose_block method to collect transactions and create new blocks- Implement validate_proposed_block method for block validation and vote collection- Expand validate_block method with more comprehensive checks- Add fields to PoCConsensus for managing pending transactions and proposed blocks- Implement vote collection and consensus reaching mechanism- Add broadcast functionality for proposed blocks- Improve error handling and logging- Update and expand unit tests for new functionality (1f766ef)

- Enter your commit message (end with an empty line):
Implement block proposal and validation in PoCConsensus- Add propose_block method to collect transactions and create new blocks- Implement validate_proposed_block method for block validation and vote collection- Expand validate_block method with more comprehensive checks- Add fields to PoCConsensus for managing pending transactions and proposed blocks- Implement vote collection and consensus reaching mechanism- Add broadcast functionality for proposed blocks- Improve error handling and logging- Update and expand unit tests for new functionality (fb81b2e)

- Enter your commit message (end with an empty line):
Implement execute_proposal method in GovernanceSystemAdd execute_proposal method to GovernanceSystem structImplement execution logic for different proposal typesUpdate Proposal struct with execution_timestamp fieldAdd comprehensive unit tests for execute_proposal functionalityEnsure proper error handling for proposal executionUpdate documentation for new and modified methodsThis commit completes the governance lifecycle by allowing passed proposals to be automatically executed. It enhances the decentralized decision-making capabilities of the InterCooperative Network. (2abfba3)

- Enter your commit message (end with an empty line):
Implement identity revocation and update features in icn_identity crate (bca128c)

- Enter your commit message (end with an empty line): (c0aeec0)

- Enter your commit message (end with an empty line):
Implement gossip protocol in NetworkManager- Add GossipMessage enum and NetworkUpdate struct for gossip communication- Implement gossip_protocol method in NetworkManager- Add helper methods for peer selection and gossip message sending- Update start method to spawn gossip protocol task- Add new test case for gossip protocol functionality- Enhance existing tests to cover new functionality- Improve error handling and logging for network operationsThis commit enhances the network layer by adding a gossip protocol, whichimproves information propagation and network consistency. The gossipprotocol regularly selects a random subset of peers and shares networkupdates with them, allowing for more efficient and robust communicationacross the network. (ea12136)

- Test commit (04665aa)

- Commit message describing the changes (2ff64d0)

- Added submodules to the InterCooperative-Network-Node repository (9209fc7)

- Finalized submodule setup and updated repository structure (91eef26)


### Refactor

- refactoring code (08e60ed)

<!-- generated by git-cliff -->
