# SPDX-License-Identifier: Unlicense
{
  description = "splora: Surmount production Bitcoin and Liquid indexer appliance";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    asset-registry = {
      url = "github:Blockstream/asset_registry_db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      crane,
      asset-registry,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSystem =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          })
        );

      # Shared builder so packages and checks do not drift.
      mkPkgs =
        pkgs:
        let
          inherit (pkgs) lib;
          rustToolchain = pkgs.rust-bin.stable."1.98.0".default;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # Link nixpkgs rocksdb. Use clang / default ld. Never gcc
          # -fuse-ld= plus a mold store path. Mold stays off.
          # src/config.rs packaging_pins_rust_198_edition_2024_and_system_rocksdb
          # asserts useSystemRocksdb = true. ELF NEEDED librocksdb is still
          # unproven until a builder runs rocksdbMoldLink.
          useSystemRocksdb = true;

          # Laptop `.cargo/config.toml` rewrites crates.io to Menhera.
          # Crane must not copy it: after vendor, cargo would still query
          # index.crates.menhera.org. Keep Cargo.lock.
          # cleanCargoSource omits flake.nix, nix/module.nix, and
          # rust-toolchain that src/config.rs tests include_str. Compose
          # filterCargoSources from the unfiltered tree so those stay in
          # src without pulling the Menhera rewrite.
          src = lib.cleanSourceWith {
            src = lib.cleanSource ./.;
            filter =
              path: type:
              let
                p = toString path;
                base = baseNameOf p;
                parent = baseNameOf (dirOf p);
                isMenheraCargoConfig =
                  parent == ".cargo" && (base == "config.toml" || base == "config");
                isIncludeStrPackaging =
                  base == "flake.nix"
                  || base == "rust-toolchain"
                  || (parent == "nix" && base == "module.nix");
              in
              !isMenheraCargoConfig
              && (craneLib.filterCargoSources path type || isIncludeStrPackaging);
            name = "source";
          };

          commonArgs = {
            inherit src;
            pname = "splora";
            version = "3.4.0-dev";
            strictDeps = true;
            cargoExtraArgs = "--offline --locked";
            nativeBuildInputs = with pkgs; [
              clang
              cmake
              # aws-lc-sys (rustls / splora-http). cmake is required.
              # nasm is the x86 assembler path; unused on aarch64.
              nasm
              pkg-config
              rustPlatform.bindgenHook
            ];
            buildInputs =
              with pkgs;
              [ ]
              ++ lib.optionals useSystemRocksdb [
                rocksdb
                snappy
                lz4
                zstd
                bzip2
                liburing
              ];
            LIBCLANG_PATH = "${lib.getLib pkgs.llvmPackages.libclang}/lib";
            GIT_HASH = self.shortRev or self.dirtyShortRev or "unknown";
          }
          // lib.optionalAttrs useSystemRocksdb {
            ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
            ROCKSDB_INCLUDE_DIR = "${pkgs.rocksdb}/include";
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          splora = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "splora";
              cargoExtraArgs = "${commonArgs.cargoExtraArgs} --bin splora --bin popular-scripts --bin splora-import --bin splora-queue";
              doCheck = false;
            }
          );

          splora-liquid = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "splora-liquid";
              cargoExtraArgs = "${commonArgs.cargoExtraArgs} --features liquid --bin splora --bin popular-scripts --bin splora-import --bin splora-queue";
              doCheck = false;
              passthru.asset-registry = asset-registry;
            }
          );

          # Default (non-liquid) lineage. Same Cargo.lock and cargoArtifacts
          # as splora. Third package next to splora and splora-liquid.
          splora-http = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "splora-http";
              cargoExtraArgs = "${commonArgs.cargoExtraArgs} --bin splora-http";
              doCheck = false;
            }
          );

          nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );
        in
        {
          inherit splora splora-liquid splora-http nextest useSystemRocksdb;
        };
    in
    {
      packages = forEachSystem (
        pkgs:
        let
          inherit (mkPkgs pkgs) splora splora-liquid splora-http;
          bitcoind = pkgs.callPackage ./nix/bitcoind.nix { };
          elementsd = pkgs.callPackage ./nix/elementsd.nix { };
        in
        {
          inherit splora splora-liquid splora-http bitcoind elementsd;
          default = splora;
        }
      );

      apps = forEachSystem (
        pkgs:
        let
          inherit (mkPkgs pkgs) splora;
        in
        {
          popular-scripts = {
            type = "app";
            program = "${splora}/bin/popular-scripts";
          };
        }
      );

      overlays.default = final: _prev: {
        splora = self.packages.${final.system}.splora;
        splora-liquid = self.packages.${final.system}.splora-liquid;
        splora-http = self.packages.${final.system}.splora-http;
        bitcoind = self.packages.${final.system}.bitcoind;
        elementsd = self.packages.${final.system}.elementsd;
      };

      checks = forEachSystem (
        pkgs:
        let
          inherit (pkgs) lib;
          built = mkPkgs pkgs;
          firstWord =
            s: lib.head (lib.filter (x: x != "") (lib.splitString " " s));
          # Dummy packages: this check is argv/unit eval, not a crane rebuild
          # and not a Bitcoin Core compile.
          mkSploraEval =
            extraSplora:
            import (pkgs.path + "/nixos/lib/eval-config.nix") {
              system = pkgs.stdenv.hostPlatform.system;
              modules = [
                {
                  nixpkgs.pkgs = pkgs;
                  nixpkgs.hostPlatform = pkgs.stdenv.hostPlatform.system;
                }
                (import ./nix/module.nix)
                {
                  boot.isContainer = true;
                  system.stateVersion = "25.05";
                  services.splora = {
                    enable = true;
                    package = pkgs.hello;
                    httpPackage = pkgs.hello;
                    liquidPackage = pkgs.hello;
                    bitcoindPackage = pkgs.hello;
                    elementsdPackage = pkgs.hello;
                    instances = {
                      mainnet.network = "mainnet";
                      testnet3.network = "testnet3";
                      testnet4.network = "testnet4";
                      mutinynet.network = "mutinynet";
                      liquid.network = "liquid";
                    };
                  }
                  // extraSplora;
                }
              ];
            };
          eval = mkSploraEval { };
          exec = name: eval.config.systemd.services.${name}.serviceConfig.ExecStart;
          countNeedle =
            needle: str:
            lib.length (lib.filter (x: x == needle) (lib.splitString " " str));
          indexerNames = [
            "splora-mainnet"
            "splora-testnet3"
            "splora-testnet4"
            "splora-mutinynet"
            "splora-liquid"
          ];
          daemonNames = [
            "bitcoind-mainnet"
            "bitcoind-testnet3"
            "bitcoind-testnet4"
            "bitcoind-mutinynet"
            "elementsd-liquid"
          ];
          indexersOk = lib.all (
            name:
            let
              s = exec name;
            in
            countNeedle "--allow-npubs-file" s == 1
            && !(lib.hasInfix "queue --bind" s)
          ) indexerNames;
          queueStart = exec "splora-queue";
          # Default unit prefers unix --socket-file. TCP --bind is only
          # when queueListen is set (queueSocketFile = null).
          queueOk =
            lib.hasInfix "splora-queue" queueStart
            && lib.hasInfix "--socket-file" queueStart
            && lib.hasInfix "/run/splora/queue.sock" queueStart
            && lib.hasInfix "--queue-file" queueStart
            && !(lib.hasInfix "--bind" queueStart)
            && !(lib.hasInfix " queue " (" " + queueStart + " "));
          allowParent = lib.any (p: p == "/var/lib/splora") (
            lib.toList eval.config.systemd.services.splora-mainnet.serviceConfig.ReadOnlyPaths
          );
          queueRw = lib.toList eval.config.systemd.services.splora-queue.serviceConfig.ReadWritePaths;
          queueDir = lib.any (p: p == "/var/lib/splora/queue") queueRw;
          queueNotAllowParent = !(lib.any (p: p == "/var/lib/splora") queueRw);
          nixosFiveInstances =
            assert indexersOk;
            assert queueOk;
            assert allowParent;
            assert queueDir;
            assert queueNotAllowParent;
            pkgs.runCommand "splora-nixos-five-instances" { } "echo ok > $out";
          bitcoindStarts = map exec [
            "bitcoind-mainnet"
            "bitcoind-testnet3"
            "bitcoind-testnet4"
            "bitcoind-mutinynet"
          ];
          bitcoindStore = firstWord (lib.head bitcoindStarts);
          tenUnitsPresent = lib.all (name: lib.hasAttr name eval.config.systemd.services) (
            indexerNames ++ daemonNames
          );
          sameBitcoind = lib.all (s: firstWord s == bitcoindStore) bitcoindStarts;
          disablewalletAll = lib.all (s: lib.hasInfix "-disablewallet" s) bitcoindStarts;
          cookiePaths =
            lib.hasInfix "/var/lib/bitcoind/mainnet/.cookie" (exec "bitcoind-mainnet")
            && lib.hasInfix "/var/lib/bitcoind/testnet3/.cookie" (exec "bitcoind-testnet3")
            && lib.hasInfix "/var/lib/bitcoind/testnet4/.cookie" (exec "bitcoind-testnet4")
            && lib.hasInfix "/var/lib/bitcoind/mutinynet/.cookie" (exec "bitcoind-mutinynet")
            && lib.hasInfix "/var/lib/elementsd/liquid/.cookie" (exec "elementsd-liquid");
          moduleText = builtins.readFile ./nix/module.nix;
          noUserPassword = !(lib.hasInfix "USER:PASSWORD" moduleText);
          indexerAfterDaemon =
            lib.elem "bitcoind-mainnet.service" eval.config.systemd.services.splora-mainnet.after
            && lib.elem "bitcoind-testnet3.service" eval.config.systemd.services.splora-testnet3.after
            && lib.elem "bitcoind-testnet4.service" eval.config.systemd.services.splora-testnet4.after
            && lib.elem "bitcoind-mutinynet.service" eval.config.systemd.services.splora-mutinynet.after
            && lib.elem "elementsd-liquid.service" eval.config.systemd.services.splora-liquid.after;
          # Mutinynet follower argv on stock Core 31.1: published unwrapped
          # challenge, addnode, dnsseed=0, wallet off. Mutinynet's 30-second
          # interval is a miner/network property; stock 31.1 has no
          # -signetblocktime (unregistered: the unit would refuse start).
          mutinynetExec = exec "bitcoind-mutinynet";
          mutinynetStockCoreArgv =
            lib.hasInfix "-signetchallenge=512102f7561d208dd9ae99bf497273e16f389bdbd6c4742ddb8e6b216e64fa2928ad8f51ae" mutinynetExec
            && lib.hasInfix "-addnode=45.79.52.207:38333" mutinynetExec
            && lib.hasInfix "-dnsseed=0" mutinynetExec
            && lib.hasInfix "-disablewallet" mutinynetExec
            && !(lib.hasInfix "-signetblocktime" mutinynetExec)
            && !(lib.hasInfix "-signetblocktime" (exec "bitcoind-mainnet"))
            && !(lib.hasInfix "-signetblocktime" (exec "bitcoind-testnet3"))
            && !(lib.hasInfix "-signetblocktime" (exec "bitcoind-testnet4"))
            && !(lib.hasInfix "-signetblocktime" (exec "elementsd-liquid"));
          nixosTenUnits =
            assert tenUnitsPresent;
            assert sameBitcoind;
            assert disablewalletAll;
            assert cookiePaths;
            assert noUserPassword;
            assert indexerAfterDaemon;
            assert mutinynetStockCoreArgv;
            pkgs.runCommand "splora-nixos-ten-units" { } "echo ok > $out";
          # Default socket plus queueListen must fail the module assertion,
          # not wait for clap to refuse both flags at start.
          evalQueueListenWithDefaultSocket = mkSploraEval {
            queueListen = "127.0.0.1:18493";
          };
          nixosQueueListenXorSocket =
            assert lib.any (
              a:
              (!a.assertion)
              && lib.hasInfix "queueSocketFile" a.message
              && lib.hasInfix "queueListen" a.message
            ) evalQueueListenWithDefaultSocket.config.assertions;
            pkgs.runCommand "splora-nixos-queue-listen-xor-socket" { } "echo ok > $out";
          # One-instance remote JSON-RPC: cookie path + rpc addr, no local
          # bitcoind datadir. Dummy package; not a crane rebuild. Cookie
          # bytes never appear in argv (path only). startLocalDaemon false
          # skips the appliance bitcoind unit.
          evalRemoteJsonrpc = mkSploraEval {
            instances = {
              mainnet = {
                network = "mainnet";
                jsonrpcImport = true;
                startLocalDaemon = false;
                daemonRpcAddr = "10.0.0.1:8332";
                cookieFile = "/run/bitcoind/.cookie";
                daemonDir = null;
              };
            };
          };
          remoteStart = evalRemoteJsonrpc.config.systemd.services.splora-mainnet.serviceConfig.ExecStart;
          remoteReadOnly = lib.toList evalRemoteJsonrpc.config.systemd.services.splora-mainnet.serviceConfig.ReadOnlyPaths;
          remoteModuleText = builtins.readFile ./nix/module.nix;
          nixosRemoteJsonrpcImport =
            assert lib.hasInfix "--jsonrpc-import" remoteStart;
            assert lib.hasInfix "--daemon-rpc-addr" remoteStart;
            assert lib.hasInfix "10.0.0.1:8332" remoteStart;
            assert lib.hasInfix "--cookie-file" remoteStart;
            assert lib.hasInfix "/run/bitcoind/.cookie" remoteStart;
            assert !(lib.hasInfix "--daemon-dir" remoteStart);
            assert !(lib.any (p: p == "/var/lib/bitcoind") remoteReadOnly);
            assert !(lib.hasInfix "/var/lib/bitcoind" (lib.concatStringsSep " " remoteReadOnly));
            assert !(lib.hasInfix "--cookie " (" " + remoteStart + " "));
            assert !(lib.hasInfix "USER:PASSWORD" remoteStart);
            assert !(lib.hasInfix "USER:PASSWORD" remoteModuleText);
            assert !(evalRemoteJsonrpc.config.systemd.services ? bitcoind-mainnet);
            pkgs.runCommand "splora-nixos-remote-jsonrpc-import" { } "echo ok > $out";
          # Named proof that crane linked nixpkgs rocksdb when
          # useSystemRocksdb is true. Do not require mold in .comment.
          # Mold does not link. clang / default ld only.
          rocksdbMoldLink =
            if built.useSystemRocksdb then
              pkgs.runCommand "splora-nixpkgs-rocksdb-mold" {
                nativeBuildInputs = [ pkgs.binutils-unwrapped ];
              } ''
                set -eu
                check_bin() {
                  local bin="$1"
                  readelf -d "$bin" | grep -E 'NEEDED.*librocksdb'
                }
                check_bin ${built.splora}/bin/splora
                check_bin ${built.splora-liquid}/bin/splora
                mkdir "$out"
                echo "nixpkgs rocksdb ${pkgs.rocksdb.version}" > "$out/ok"
                echo "bindgen ROCKSDB_INCLUDE_DIR ${pkgs.rocksdb}/include" >> "$out/ok"
                echo "SONAME librocksdb.so.10; mold is not required in .comment" >> "$out/ok"
              ''
            else
              pkgs.runCommand "splora-bundled-rocksdb" { } ''
                echo "useSystemRocksdb is false; bundled rocksdb 0.24.0" > "$out"
              '';
        in
        {
          splora = built.splora;
          splora-liquid = built.splora-liquid;
          splora-http = built.splora-http;
          nextest = built.nextest;
          inherit nixosFiveInstances;
          inherit nixosTenUnits;
          inherit nixosQueueListenXorSocket;
          inherit nixosRemoteJsonrpcImport;
          inherit rocksdbMoldLink;
        }
      );

      nixosModules.splora = import ./nix/module.nix;
    };
}
