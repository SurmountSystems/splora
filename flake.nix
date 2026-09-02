# SPDX-License-Identifier: Unlicense
{
  description = "splora: Surmount production Bitcoin and Liquid indexer";

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

          # Prefer one nixpkgs rocksdb for both binaries when the builder
          # can link it. 2026-09-01 `just check-remote` never reached
          # `rocksdbMoldLink`: `splora-deps` failed because gcc rejected
          # `-fuse-ld=` plus the mold store path. Mold does not still
          # link. Fallback is bundled rocksdb 0.24.0 (no ROCKSDB_* env,
          # no mold RUSTFLAGS). See doc/supply-chain.md.
          useSystemRocksdb = false;

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
          inherit splora splora-liquid nextest useSystemRocksdb;
        };
    in
    {
      packages = forEachSystem (
        pkgs:
        let
          inherit (mkPkgs pkgs) splora splora-liquid;
        in
        {
          inherit splora splora-liquid;
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
      };

      checks = forEachSystem (
        pkgs:
        let
          inherit (pkgs) lib;
          built = mkPkgs pkgs;
          # Dummy packages: this check is argv/unit eval, not a crane rebuild.
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
                    liquidPackage = pkgs.hello;
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
          # bytes never appear in argv (path only).
          evalRemoteJsonrpc = mkSploraEval {
            instances = {
              mainnet = {
                network = "mainnet";
                jsonrpcImport = true;
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
            pkgs.runCommand "splora-nixos-remote-jsonrpc-import" { } "echo ok > $out";
          # Named proof that crane linked nixpkgs rocksdb and mold when
          # useSystemRocksdb is true. 2026-09-01 `just check-remote` failed
          # in splora-deps on mold (`gcc: unrecognized -fuse-ld=` path).
          # useSystemRocksdb is false; this check records bundled 0.24.0.
          rocksdbMoldLink =
            if built.useSystemRocksdb then
              pkgs.runCommand "splora-nixpkgs-rocksdb-mold" {
                nativeBuildInputs = [ pkgs.binutils-unwrapped ];
              } ''
                set -eu
                check_bin() {
                  local bin="$1"
                  readelf -d "$bin" | grep -E 'NEEDED.*librocksdb'
                  readelf -p .comment "$bin" | grep -qi mold
                }
                check_bin ${built.splora}/bin/splora
                check_bin ${built.splora-liquid}/bin/splora
                mkdir "$out"
                echo "nixpkgs rocksdb ${pkgs.rocksdb.version}" > "$out/ok"
                echo "bindgen ROCKSDB_INCLUDE_DIR ${pkgs.rocksdb}/include" >> "$out/ok"
                echo "SONAME librocksdb.so.10 (10.10.1); crate vendor is 10.4.2 unused" >> "$out/ok"
              ''
            else
              pkgs.runCommand "splora-bundled-rocksdb" { } ''
                echo "useSystemRocksdb is false; bundled rocksdb 0.24.0" > "$out"
              '';
        in
        {
          splora = built.splora;
          splora-liquid = built.splora-liquid;
          nextest = built.nextest;
          inherit nixosFiveInstances;
          inherit nixosQueueListenXorSocket;
          inherit nixosRemoteJsonrpcImport;
          inherit rocksdbMoldLink;
        }
      );

      nixosModules.splora = import ./nix/module.nix;
    };
}
