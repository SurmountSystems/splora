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
          rustToolchain = pkgs.rust-bin.stable."1.87.0".default;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # Prefer one nixpkgs rocksdb for both binaries. Named check
          # `rocksdbMoldLink` (ELF NEEDED librocksdb + mold .comment) is the
          # proof path. If that check fails, set this to false (bundled
          # rocksdb 0.24.0). See doc/supply-chain.md.
          useSystemRocksdb = true;

          src = craneLib.cleanCargoSource ./.;

          commonArgs = {
            inherit src;
            pname = "splora";
            version = "3.4.0-dev";
            strictDeps = true;
            nativeBuildInputs = with pkgs; [
              clang
              cmake
              pkg-config
              mold
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
              ];
            LIBCLANG_PATH = "${lib.getLib pkgs.llvmPackages.libclang}/lib";
            GIT_HASH = self.shortRev or self.dirtyShortRev or "unknown";
            RUSTFLAGS = "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
          }
          // lib.optionalAttrs useSystemRocksdb {
            ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
            ROCKSDB_INCLUDE_DIR = "${pkgs.rocksdb}/include";
          };

          cargoArtifacts = craneLib.buildDepsOnly (
            commonArgs
            // {
              cargoExtraArgs = "--locked";
            }
          );

          splora = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "splora";
              cargoExtraArgs = "--locked --bin splora --bin popular-scripts --bin splora-import --bin splora-queue";
              doCheck = false;
            }
          );

          splora-liquid = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "splora-liquid";
              cargoExtraArgs = "--locked --features liquid --bin splora --bin popular-scripts --bin splora-import --bin splora-queue";
              doCheck = false;
              passthru.asset-registry = asset-registry;
            }
          );

          nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoExtraArgs = "--locked";
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
          # Named proof that crane linked nixpkgs rocksdb and mold. Operators
          # run this via `just check-remote` (`nix flake check`). Agents do
          # not run that recipe. If it fails (liburing, header skew), set
          # useSystemRocksdb = false and keep bundled rocksdb 0.24.0.
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
                echo ok > "$out/ok"
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
          inherit rocksdbMoldLink;
        }
      );

      nixosModules.splora = import ./nix/module.nix;
    };
}
