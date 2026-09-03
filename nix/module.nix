# NixOS module for the splora appliance. Import from a flake with:
#   nixosModules.splora = import ./nix/module.nix;
#
# Splora is the appliance: this module starts the indexer processes and,
# by default, local bitcoind / elementsd units built in this project.
# One indexer process per network. Do not pass npubs on ExecStart.
# Writes to the allowlist are:
#   splora-import approve --queue <queueFile> --allowlist <allowNpubsFile> <npub>
#   splora-import reject --queue <queueFile> <npub>
#   splora-import remove --allowlist <allowNpubsFile> <npub>
# Queue HTTP is only the `splora-queue` unit. The binary takes --socket-file
# or --bind, not both. Prefer --socket-file /run/splora/queue.sock. TCP is
# --bind from queueListen when queueSocketFile is null. Queue listen is
# never on indexer ExecStart.
# Pending queue file is CSV npub,email. Allowlist is approved npubs, one per
# line. splora-queue does not write the allowlist; splora-import does.
# Mutinynet uses --network signet --magic a5df2dcb.
# Default Bitcoin signet magic is 0x0A03CF40.
#
# Local daemons: four systemd bitcoind units from one bitcoindPackage
# (bitcoind-mainnet, bitcoind-testnet3, bitcoind-testnet4,
# bitcoind-mutinynet) plus elementsd-liquid. Wallet off. Cookie paths
# only. Never put rpc user/password pairs on argv. RPC on 127.0.0.1.
# P2P stays on (no -listen=0). Per-instance startLocalDaemon=false skips
# that chain's daemon unit and uses remote cookieFile + daemonRpcAddr +
# daemonDir=null.
#
# Core appends testnet3 / testnet4 / signet (and Elements liquidv1) under
# -datadir. -rpccookiefile=/var/lib/bitcoind/<net>/.cookie (and the
# elementsd liquid cookie) pins a stable cookie path. -blocksdir pins
# blk*.dat at that named datadir so indexer daemonDir can stay the spec
# path. jsonrpcImport defaults true, so the indexer talks JSON-RPC and
# does not require nested blk*.dat. Chainstate may still nest under
# Core's network subdirectory.
#
# Indexer HTTP is the unix socket /run/splora/<name>.http.sock. No nginx.
# Electrum is unix socket only (never TCP). Public TLS, HTTP/2 (TCP 443),
# and HTTP/3 (QUIC UDP 443) terminate on splora-http.service. That unit
# is distinct from indexer instances. It passes --socket-dir /run/splora
# and proxies only to the five <name>.http.sock files. It never opens
# *.electrum.sock. --tls-cert and --tls-key are file paths. Never put
# private key or certificate PEM bytes in Nix or git.
{
  config,
  lib,
  pkgs,
  ...
}:

let
  inherit (lib)
    concatStringsSep
    dirOf
    escapeShellArgs
    filterAttrs
    literalExpression
    mapAttrs'
    mkDefault
    mkEnableOption
    mkIf
    mkMerge
    mkOption
    nameValuePair
    optional
    optionalAttrs
    optionals
    optionalString
    types
    ;

  cfg = config.services.splora;

  cliNetwork = {
    mainnet = "mainnet";
    testnet3 = "testnet";
    testnet4 = "testnet4";
    # Mutinynet is signet with chain magic a5df2dcb.
    # Default Bitcoin signet magic is 0x0A03CF40 (hex 0a03cf40).
    mutinynet = "signet";
    liquid = "liquid";
  };

  defaultHttpPort = {
    mainnet = 3000;
    testnet3 = 3001;
    testnet4 = 3004;
    mutinynet = 3003;
    liquid = 3000;
  };

  defaultDaemonRpcPort = {
    mainnet = 8332;
    testnet3 = 18332;
    testnet4 = 48332;
    mutinynet = 38332;
    liquid = 7041;
  };

  # Mutinynet chain magic. Default signet is 0x0A03CF40.
  mutinynetMagic = "a5df2dcb";

  # Published Mutinynet faucet challenge (mutinynet.com / nobsbitcoin).
  # Indexer magic stays a5df2dcb. Do not wrap the challenge: stock Core
  # treats the hex as the script, so wrapping would change P2P magic away
  # from a5df2dcb. Mutinynet's 30-second interval is a miner/network
  # property. Stock Core 31.1 has no -signetblocktime (unregistered: the
  # unit would refuse start). A follower only needs the unwrapped
  # challenge, addnode, and dnsseed=0.
  mutinynetSignetChallenge = "512102f7561d208dd9ae99bf497273e16f389bdbd6c4742ddb8e6b216e64fa2928ad8f51ae";
  mutinynetAddnode = "45.79.52.207:38333";

  applianceDatadir = net: if net == "liquid" then "/var/lib/elementsd/liquid" else "/var/lib/bitcoind/${net}";

  applianceCookie = net: "${applianceDatadir net}/.cookie";

  daemonUnitName = net: if net == "liquid" then "elementsd-liquid" else "bitcoind-${net}";

  chainArgv = {
    mainnet = [ ];
    testnet3 = [ "-testnet" ];
    testnet4 = [ "-testnet4" ];
    mutinynet = [
      "-signet"
      "-signetchallenge=${mutinynetSignetChallenge}"
      "-addnode=${mutinynetAddnode}"
      "-dnsseed=0"
    ];
    liquid = [ "-chain=liquidv1" ];
  };

  instancePackage = inst: if inst.network == "liquid" then cfg.liquidPackage else cfg.package;

  enabledInstances = filterAttrs (_: inst: inst.enable) cfg.instances;

  localDaemonWanted =
    net:
    lib.any (inst: inst.startLocalDaemon && inst.network == net) (lib.attrValues enabledInstances);

  instanceOptions =
    { name, config, ... }:
    {
      options = {
        enable = mkEnableOption "this splora indexer process" // {
          default = true;
        };

        network = mkOption {
          type = types.enum [
            "mainnet"
            "testnet3"
            "testnet4"
            "mutinynet"
            "liquid"
          ];
          description = ''
            Chain this process indexes. Each network is its own process and
            RocksDB. CLI --network is mainnet, testnet, testnet4, signet
            (mutinynet), or liquid.
          '';
        };

        startLocalDaemon = mkOption {
          type = types.bool;
          default = true;
          description = ''
            Start this module's bitcoind or elementsd unit for this chain.
            When false, skip that daemon unit. Use remote cookieFile plus
            daemonRpcAddr and set daemonDir = null. Cookie path only.
            Never put rpc user/password pairs on argv.
          '';
        };

        daemonDir = mkOption {
          type = types.nullOr types.str;
          description = ''
            Bitcoind or elementsd data-directory root passed as --daemon-dir.
            The indexer appends the network subdirectory (testnet3, testnet4,
            signet, liquidv1) when it walks blk*.dat. Appliance defaults are
            /var/lib/bitcoind/<net> and /var/lib/elementsd/liquid. Local
            daemons pass -blocksdir at that same path and jsonrpcImport
            defaults true, so nested Core blocks are not required. Null is
            remote JSON-RPC: omit --daemon-dir and omit that path from
            ReadOnlyPaths. Remote mode requires cookieFile and daemonRpcAddr.
            Never put cookie bytes in this option.
          '';
        };

        daemonRpcAddr = mkOption {
          type = types.str;
          description = "JSON-RPC address of bitcoind or elementsd.";
        };

        cookieFile = mkOption {
          type = types.nullOr types.str;
          default = null;
          example = "/var/lib/bitcoind/mainnet/.cookie";
          description = ''
            Path to the bitcoind or elementsd cookie file, passed as
            --cookie-file. Never put cookie contents in Nix. Appliance
            default is the stable -rpccookiefile path under the named
            datadir. Required when daemonDir is null (remote JSON-RPC).
          '';
        };

        jsonrpcImport = mkOption {
          type = types.bool;
          default = true;
          description = ''
            Pass --jsonrpc-import so the indexer uses bitcoind JSON-RPC
            instead of local blk*.dat files. Default true on the appliance.
            Set true for a remote node.
          '';
        };

        publicHealth = mkOption {
          type = types.bool;
          default = false;
          description = ''
            Pass --public-health. Tip health is open; an empty allowlist
            still 401s address, tx, and mempool REST.
          '';
        };

        memoryMax = mkOption {
          type = types.nullOr types.str;
          default = null;
          example = "8G";
          description = ''
            Optional systemd MemoryMax for this indexer unit. Null leaves
            MemoryMax unset. Public sample leaves this unset.
          '';
        };

        dbDir = mkOption {
          type = types.str;
          default = "/var/lib/splora/${name}/db";
          description = "RocksDB directory passed as --db-dir.";
        };

        httpAddr = mkOption {
          type = types.nullOr types.str;
          default = null;
          example = "0.0.0.0";
          description = ''
            Optional HTTP TCP bind host. Used only when httpSocketFile is
            null. Production prefers the unix socket. Set this (for example
            0.0.0.0) and httpSocketFile = null to listen on a host port.
          '';
        };

        httpPort = mkOption {
          type = types.port;
          description = "HTTP TCP port. Unused when httpSocketFile is set.";
        };

        httpSocketFile = mkOption {
          type = types.nullOr types.str;
          default = "/run/splora/${name}.http.sock";
          example = "/run/splora/${name}.http.sock";
          description = ''
            HTTP Unix socket (--http-socket-file) for REST, POST /electrum,
            and /api/v1/ws. Default is under RuntimeDirectory splora. Set
            null and set httpAddr to bind TCP instead. When this is set,
            --http-addr is not passed.
          '';
        };

        electrumSocketFile = mkOption {
          type = types.str;
          default = "/run/splora/${name}.electrum.sock";
          description = ''
            Electrum JSON-RPC Unix socket (--rpc-socket-file). Raw TCP
            Electrum is not opened. POST /electrum remains on the HTTP
            socket.
          '';
        };

        assetDbPath = mkOption {
          type = types.nullOr types.str;
          description = "Liquid/Elements asset DB (--asset-db-path). Liquid only.";
        };

        dbBlockCacheMb = mkOption {
          type = types.ints.positive;
          default = cfg.dbBlockCacheMb;
          defaultText = literalExpression "config.services.splora.dbBlockCacheMb";
          description = "RocksDB block cache size in MiB (--db-block-cache-mb).";
        };

        dbParallelism = mkOption {
          type = types.ints.positive;
          default = cfg.dbParallelism;
          defaultText = literalExpression "config.services.splora.dbParallelism";
          description = "RocksDB background threads (--db-parallelism).";
        };

        extraArgs = mkOption {
          type = types.listOf types.str;
          default = [ ];
          description = "Extra argv appended to the indexer.";
        };
      };

      config = {
        daemonDir = mkDefault (applianceDatadir config.network);
        cookieFile = mkDefault (applianceCookie config.network);
        httpPort = mkDefault defaultHttpPort.${config.network};
        daemonRpcAddr = mkDefault "127.0.0.1:${toString defaultDaemonRpcPort.${config.network}}";
        assetDbPath = mkDefault (
          if config.network == "liquid" then "/var/lib/splora/${name}/asset-db" else null
        );
      };
    };

  indexerArgs =
    _name: inst:
    [
      "-vvv"
      "--timestamp"
      "--network"
      cliNetwork.${inst.network}
      "--daemon-rpc-addr"
      inst.daemonRpcAddr
      "--db-dir"
      inst.dbDir
      "--db-block-cache-mb"
      (toString inst.dbBlockCacheMb)
      "--db-parallelism"
      (toString inst.dbParallelism)
      "--allow-npubs-file"
      cfg.allowNpubsFile
      "--rpc-socket-file"
      inst.electrumSocketFile
    ]
    ++ optionals (inst.daemonDir != null) [
      "--daemon-dir"
      inst.daemonDir
    ]
    ++ optionals (inst.httpSocketFile != null) [
      "--http-socket-file"
      inst.httpSocketFile
    ]
    ++ optionals (inst.httpSocketFile == null && inst.httpAddr != null) [
      "--http-addr"
      "${inst.httpAddr}:${toString inst.httpPort}"
    ]
    ++ optionals (inst.cookieFile != null) [
      "--cookie-file"
      inst.cookieFile
    ]
    ++ optional inst.jsonrpcImport "--jsonrpc-import"
    ++ optional inst.publicHealth "--public-health"
    ++ optional (inst.network == "mutinynet") "--magic"
    ++ optional (inst.network == "mutinynet") mutinynetMagic
    ++ optional (inst.network == "liquid" && inst.assetDbPath != null) "--asset-db-path"
    ++ optional (inst.network == "liquid" && inst.assetDbPath != null) inst.assetDbPath
    ++ inst.extraArgs;

  indexerService = name: inst: {
    description = "splora indexer (${name}, ${inst.network})";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" ] ++ optional inst.startLocalDaemon "${daemonUnitName inst.network}.service";
    wants = optional inst.startLocalDaemon "${daemonUnitName inst.network}.service";
    serviceConfig = {
      Type = "simple";
      User = cfg.user;
      Group = cfg.group;
      ExecStart = "${lib.getExe (instancePackage inst)} ${escapeShellArgs (indexerArgs name inst)}";
      Environment = "RUST_BACKTRACE=1";
      Restart = "on-failure";
      RestartSec = 10;
      StateDirectory = "splora/${name}";
      # Shared /run/splora so HTTP, Electrum, and queue sockets sit next
      # to each other. Preserve so one indexer stop does not delete the
      # directory out from under the others.
      RuntimeDirectory = "splora";
      RuntimeDirectoryMode = "0750";
      RuntimeDirectoryPreserve = true;
      UMask = "0007";
      # Bind the allowlist parent directory, not the file inode, so
      # splora-import rename(tmp -> allow-npubs) is visible to inotify.
      BindReadOnlyPaths = optional (inst.cookieFile != null) inst.cookieFile;
      # Do not pass --lightmode. Light mode stays off.
      NoNewPrivileges = true;
      PrivateTmp = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      ReadWritePaths = [
        inst.dbDir
      ]
      ++ optional (inst.network == "liquid" && inst.assetDbPath != null) inst.assetDbPath;
      ReadOnlyPaths =
        [ (dirOf cfg.allowNpubsFile) ]
        ++ optional (inst.daemonDir != null) inst.daemonDir;
      MemoryDenyWriteExecute = true;
    }
    // optionalAttrs (inst.memoryMax != null) {
      MemoryMax = inst.memoryMax;
    };
  };

  daemonArgv =
    net:
    [
      "-datadir=${applianceDatadir net}"
      "-blocksdir=${applianceDatadir net}"
      "-rpccookiefile=${applianceCookie net}"
      "-disablewallet"
      "-txindex=1"
      "-server=1"
      "-rpcallowip=127.0.0.1"
      "-rpcbind=127.0.0.1"
      "-rpcport=${toString defaultDaemonRpcPort.${net}}"
      "-printtoconsole"
    ]
    ++ optional (net != "liquid") "-rpccookieperms=group"
    ++ optional (net == "liquid") "-startupnotify=chmod g+r ${applianceCookie net}"
    ++ chainArgv.${net};

  daemonService =
    net: pkg: userName:
    {
      description = "splora appliance ${if net == "liquid" then "elementsd" else "bitcoind"} (${net})";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      serviceConfig = {
        Type = "simple";
        User = userName;
        Group = userName;
        ExecStart = "${lib.getExe pkg} ${escapeShellArgs (daemonArgv net)}";
        Restart = "on-failure";
        RestartSec = 10;
        StateDirectory = if net == "liquid" then "elementsd/liquid" else "bitcoind/${net}";
        StateDirectoryMode = "0750";
        UMask = "0027";
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        MemoryDenyWriteExecute = true;
      };
    };

  queueHasSocket = cfg.queueSocketFile != null;
  queueHasListen = cfg.queueListen != null;

  # Exclusive: --socket-file XOR --bind. clap refuses both. TCP needs
  # queueSocketFile = null so this unit does not pass both flags.
  queueListenArgs =
    optionals queueHasSocket [
      "--socket-file"
      cfg.queueSocketFile
    ]
    ++ optionals queueHasListen [
      "--bind"
      cfg.queueListen
    ];

  popularInstance =
    if cfg.popularScripts.enable then cfg.instances.${cfg.popularScripts.instance} or null else null;

  daemonUnits =
    optionalAttrs (localDaemonWanted "mainnet") {
      bitcoind-mainnet = daemonService "mainnet" cfg.bitcoindPackage "bitcoind";
    }
    // optionalAttrs (localDaemonWanted "testnet3") {
      bitcoind-testnet3 = daemonService "testnet3" cfg.bitcoindPackage "bitcoind";
    }
    // optionalAttrs (localDaemonWanted "testnet4") {
      bitcoind-testnet4 = daemonService "testnet4" cfg.bitcoindPackage "bitcoind";
    }
    // optionalAttrs (localDaemonWanted "mutinynet") {
      bitcoind-mutinynet = daemonService "mutinynet" cfg.bitcoindPackage "bitcoind";
    }
    // optionalAttrs (localDaemonWanted "liquid") {
      elementsd-liquid = daemonService "liquid" cfg.elementsdPackage "elementsd";
    };

in
{
  options.services.splora = {
    enable = mkEnableOption "splora indexer, import queue HTTP, TLS HTTP front, local bitcoind/elementsd, and shared allowlist";

    package = mkOption {
      type = types.package;
      default = pkgs.splora;
      defaultText = literalExpression "pkgs.splora";
      description = "splora package for Bitcoin-family instances and the queue service.";
    };

    httpPackage = mkOption {
      type = types.package;
      default = pkgs.splora-http;
      defaultText = literalExpression "pkgs.splora-http";
      description = "splora-http path-routing TLS front (packages.splora-http). Not the indexer package; that crane drv does not install this bin.";
    };

    liquidPackage = mkOption {
      type = types.package;
      default = pkgs.splora-liquid;
      defaultText = literalExpression "pkgs.splora-liquid";
      description = "splora package built with the liquid feature (assetDbPath).";
    };

    bitcoindPackage = mkOption {
      type = types.package;
      default = pkgs.bitcoind;
      defaultText = literalExpression "pkgs.bitcoind";
      description = ''
        One bitcoind derivation for mainnet, testnet3, testnet4, and
        mutinynet units. Prefer this flake overlay so pkgs.bitcoind is
        Bitcoin Core 31.1 from nix/bitcoind.nix.
      '';
    };

    elementsdPackage = mkOption {
      type = types.package;
      default = pkgs.elementsd;
      defaultText = literalExpression "pkgs.elementsd";
      description = "elementsd derivation for the liquid unit. Prefer this flake overlay.";
    };

    user = mkOption {
      type = types.str;
      default = "splora";
      description = "System user for indexer, queue, and splora-http units.";
    };

    group = mkOption {
      type = types.str;
      default = "splora";
      description = "System group for indexer, queue, and splora-http units.";
    };

    allowNpubsFile = mkOption {
      type = types.str;
      default = "/var/lib/splora/allow-npubs";
      description = ''
        Approved npubs, one per line, shared by every instance. The indexer
        opens it read-only. The writer is splora-import
        approve|reject|remove with --queue and --allowlist. An empty file
        means nobody is authorized. This is not the pending queue file.
      '';
    };

    queueFile = mkOption {
      type = types.str;
      default = "/var/lib/splora/queue/import-queue";
      description = ''
        Pending import-request file written by the queue HTTP service.
        One line npub,email. Must live in its own directory so splora-queue
        ReadWritePaths is not the allowlist parent or an indexer dbDir.
        Queue HTTP does not write the allowlist.
      '';
    };

    queueSocketFile = mkOption {
      type = types.nullOr types.str;
      default = "/run/splora/queue.sock";
      description = ''
        Unix socket for unauthenticated queue HTTP (--socket-file).
        Preferred production bind. TCP needs this set to null and
        queueListen set. Do not set both. The Surmount Axum edge should
        point at this path. No nginx.
      '';
    };

    queueListen = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "127.0.0.1:18493";
      description = ''
        Optional TCP bind for splora-queue (--bind) when the operator wants
        a host port. Requires queueSocketFile = null. Do not set both.
        Null means no TCP. This is not required to be localhost. The queue
        is unauthenticated; prefer the unix socket plus the Surmount edge.
        Never passed on indexer ExecStart.
      '';
    };

    queueEnable = mkOption {
      type = types.bool;
      default = true;
      description = "Start the import-queue HTTP service (splora-queue).";
    };

    http = {
      enable = mkEnableOption "splora-http TLS front (TCP 443 and QUIC UDP 443)" // {
        # Default false so existing NixOS evals that do not set TLS file
        # paths still succeed. Set true and set tlsCertFile plus tlsKeyFile
        # on the appliance.
        default = false;
      };

      listen = mkOption {
        type = types.str;
        default = "0.0.0.0:443";
        description = ''
          TCP listen address for TLS HTTP/1.1 and HTTP/2 (--listen).
          Required by splora-http. Default is all interfaces port 443.
        '';
      };

      quic = mkOption {
        type = types.str;
        default = "0.0.0.0:443";
        description = ''
          UDP listen address for HTTP/3 QUIC (--quic). QUIC is UDP, not a
          Unix domain socket. Default matches --listen (0.0.0.0:443).
        '';
      };

      tlsCertFile = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "/var/lib/splora/tls/fullchain.pem";
        description = ''
          Path to the PEM certificate chain file, passed as --tls-cert.
          File path only. Never put certificate bytes in this option.
          Never use types.path (that copies the file into the Nix store).
          Required when http.enable is true.
        '';
      };

      tlsKeyFile = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "/var/lib/splora/tls/privkey.pem";
        description = ''
          Path to the PEM private key file, passed as --tls-key. File
          path only. Never put private key bytes in this option, in git,
          or in the Nix store. Never use types.path. Required when
          http.enable is true.
        '';
      };

      socketDir = mkOption {
        type = types.str;
        default = "/run/splora";
        description = ''
          Directory of indexer HTTP unix sockets (--socket-dir). Default
          /run/splora, created as RuntimeDirectory on indexer units and
          on this front. Backends are only <instance>.http.sock (the five
          default instances: mainnet, testnet3, testnet4, mutinynet,
          liquid). Never point this at *.electrum.sock.
        '';
      };
    };

    dbBlockCacheMb = mkOption {
      type = types.ints.positive;
      default = 24;
      description = ''
        Default RocksDB block cache in MiB for new instances. Matches the
        CLI default of 24. REST does not require 4096. Set more if the
        operator wants a larger cache. LSM indexes live on NVMe. The block
        cache is RAM.
      '';
    };

    dbParallelism = mkOption {
      type = types.ints.positive;
      default = 32;
      description = "Default RocksDB parallelism for new instances.";
    };

    popularScripts = {
      enable = mkEnableOption "periodic popular-scripts scan of an instance history DB";

      instance = mkOption {
        type = types.str;
        default = "mainnet";
        description = "instances.<name> whose --db-dir the timer reads.";
      };

      outputFile = mkOption {
        type = types.str;
        default = "/var/lib/splora/popular-scripts/popular-scripts.txt";
        description = ''
          Where the oneshot writes popular-scripts stdout. Must live in its
          own directory so ReadWritePaths is not the allowlist parent.
        '';
      };

      onCalendar = mkOption {
        type = types.str;
        default = "weekly";
        description = "systemd OnCalendar for the popular-scripts timer.";
      };
    };

    instances = mkOption {
      type = types.attrsOf (types.submodule instanceOptions);
      default = {
        mainnet.network = "mainnet";
        testnet3.network = "testnet3";
        testnet4.network = "testnet4";
        mutinynet.network = "mutinynet";
        liquid.network = "liquid";
      };
      example = literalExpression ''
        {
          mainnet.network = "mainnet";
          testnet3.network = "testnet3";
          testnet4.network = "testnet4";
          mutinynet.network = "mutinynet";
          liquid.network = "liquid";
        }
      '';
      description = ''
        One indexer process per network name. Default is five instances
        on: mainnet, testnet3, testnet4, mutinynet, liquid.
      '';
    };
  };

  config = mkIf cfg.enable (mkMerge [
    {
      assertions = [
        {
          assertion = !(cfg.queueEnable && !queueHasSocket && !queueHasListen);
          message = "services.splora.queueEnable requires queueSocketFile (preferred /run/splora/queue.sock) or queueListen for TCP --bind.";
        }
        {
          assertion = !(cfg.queueEnable && queueHasSocket && queueHasListen);
          message = "services.splora.queueEnable: set queueSocketFile or queueListen, not both. TCP needs queueSocketFile = null.";
        }
        {
          assertion = dirOf cfg.queueFile != dirOf cfg.allowNpubsFile;
          message = "services.splora.queueFile must live in a different directory than the allowlist so the unauthenticated queue unit cannot write allowNpubsFile.";
        }
        {
          assertion = !cfg.popularScripts.enable || popularInstance != null;
          message = "services.splora.popularScripts.instance must name an enabled services.splora.instances key.";
        }
        {
          assertion =
            !cfg.popularScripts.enable
            || dirOf cfg.popularScripts.outputFile != dirOf cfg.allowNpubsFile;
          message = "services.splora.popularScripts.outputFile must live in a different directory than the allowlist so the oneshot cannot write allowNpubsFile.";
        }
        {
          assertion = !(cfg.http.enable && (cfg.http.tlsCertFile == null || cfg.http.tlsKeyFile == null));
          message = "services.splora.http.enable requires http.tlsCertFile and http.tlsKeyFile as filesystem paths to PEM files. Never put PEM bytes in Nix.";
        }
      ]
      ++ lib.mapAttrsToList (name: inst: {
        assertion = inst.httpSocketFile != null || inst.httpAddr != null;
        message = "services.splora.instances.${name} needs httpSocketFile (preferred /run/splora/${name}.http.sock) or httpAddr for TCP.";
      }) enabledInstances
      ++ lib.mapAttrsToList (name: inst: {
        assertion = inst.daemonDir != null || (inst.cookieFile != null && inst.daemonRpcAddr != "");
        message = "services.splora.instances.${name}: remote mode (daemonDir = null) requires cookieFile and daemonRpcAddr.";
      }) enabledInstances;

      warnings = lib.filter (w: w != "") (
        lib.mapAttrsToList (
          name: inst:
          optionalString (inst.network != "liquid" && inst.assetDbPath != null)
            "services.splora.instances.${name}.assetDbPath is set but network is not liquid."
        ) enabledInstances
      );

      users.users.${cfg.user} = {
        isSystemUser = true;
        group = cfg.group;
        home = "/var/lib/splora";
        createHome = true;
        extraGroups = [
          "bitcoind"
          "elementsd"
        ];
      };
      users.groups.${cfg.group} = { };

      users.users.bitcoind = {
        isSystemUser = true;
        group = "bitcoind";
      };
      users.groups.bitcoind = { };

      users.users.elementsd = {
        isSystemUser = true;
        group = "elementsd";
      };
      users.groups.elementsd = { };

      systemd.tmpfiles.rules = [
        "d /var/lib/splora 0750 ${cfg.user} ${cfg.group} -"
        "d /run/splora 0750 ${cfg.user} ${cfg.group} -"
        "d ${dirOf cfg.queueFile} 0750 ${cfg.user} ${cfg.group} -"
        # Type f creates the file only when missing. Age - and no argument
        # leaves an empty allowlist or queue. An empty allowlist means nobody
        # is authorized. The queue directory is not the allowlist parent.
        "f ${cfg.allowNpubsFile} 0640 ${cfg.user} ${cfg.group} -"
        "f ${cfg.queueFile} 0640 ${cfg.user} ${cfg.group} -"
        "d /var/lib/bitcoind 0750 bitcoind bitcoind -"
        "d /var/lib/elementsd 0750 elementsd elementsd -"
      ]
      ++ optional cfg.popularScripts.enable "d ${dirOf cfg.popularScripts.outputFile} 0750 ${cfg.user} ${cfg.group} -"
      ++ lib.mapAttrsToList (_name: inst: "d ${inst.dbDir} 0750 ${cfg.user} ${cfg.group} -") enabledInstances
      ++ lib.filter (r: r != "") (
        lib.mapAttrsToList (
          _name: inst:
          optionalString (inst.assetDbPath != null)
            "d ${inst.assetDbPath} 0750 ${cfg.user} ${cfg.group} -"
        ) enabledInstances
      )
      ++ optional (localDaemonWanted "mainnet") "d ${applianceDatadir "mainnet"} 0750 bitcoind bitcoind -"
      ++ optional (localDaemonWanted "testnet3") "d ${applianceDatadir "testnet3"} 0750 bitcoind bitcoind -"
      ++ optional (localDaemonWanted "testnet4") "d ${applianceDatadir "testnet4"} 0750 bitcoind bitcoind -"
      ++ optional (localDaemonWanted "mutinynet") "d ${applianceDatadir "mutinynet"} 0750 bitcoind bitcoind -"
      ++ optional (localDaemonWanted "liquid") "d ${applianceDatadir "liquid"} 0750 elementsd elementsd -";

      systemd.services =
        (mapAttrs' (name: inst: nameValuePair "splora-${name}" (indexerService name inst)) enabledInstances)
        // daemonUnits;
    }

    (mkIf (cfg.http.enable && cfg.http.tlsCertFile != null && cfg.http.tlsKeyFile != null) {
      networking.firewall.allowedTCPPorts = [ 443 ];
      networking.firewall.allowedUDPPorts = [ 443 ];

      systemd.services.splora-http = {
        description = "splora TLS HTTP front (HTTP/2 TCP 443, HTTP/3 QUIC UDP 443)";
        wantedBy = [ "multi-user.target" ];
        # Indexer units own RuntimeDirectory=splora so /run/splora exists.
        # There is no umbrella splora.service; instances are splora-<name>.
        after = [ "network.target" ] ++ lib.mapAttrsToList (name: _: "splora-${name}.service") enabledInstances;
        wants = lib.mapAttrsToList (name: _: "splora-${name}.service") enabledInstances;
        serviceConfig = {
          Type = "simple";
          User = cfg.user;
          Group = cfg.group;
          ExecStart = concatStringsSep " " (
            [
              (lib.getExe' cfg.httpPackage "splora-http")
            ]
            ++ map lib.escapeShellArg [
              "--listen"
              cfg.http.listen
              "--quic"
              cfg.http.quic
              "--tls-cert"
              cfg.http.tlsCertFile
              "--tls-key"
              cfg.http.tlsKeyFile
              "--socket-dir"
              cfg.http.socketDir
            ]
          );
          Restart = "on-failure";
          RestartSec = 5;
          # Shared /run/splora with the indexers. Preserve so stopping this
          # front does not delete indexer HTTP sockets.
          RuntimeDirectory = "splora";
          RuntimeDirectoryMode = "0750";
          RuntimeDirectoryPreserve = true;
          UMask = "0007";
          # Bind TCP 443 and UDP 443 as User=splora.
          AmbientCapabilities = "CAP_NET_BIND_SERVICE";
          CapabilityBoundingSet = "CAP_NET_BIND_SERVICE";
          # File paths only. Never PEM bytes.
          BindReadOnlyPaths = [
            cfg.http.tlsCertFile
            cfg.http.tlsKeyFile
          ];
          NoNewPrivileges = true;
          PrivateTmp = true;
          ProtectSystem = "strict";
          ProtectHome = true;
          MemoryDenyWriteExecute = true;
        };
      };
    })

    (mkIf cfg.queueEnable {
      systemd.services.splora-queue = {
        description = "splora import-queue HTTP (unauthenticated POST)";
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];
        serviceConfig = {
          Type = "simple";
          User = cfg.user;
          Group = cfg.group;
          ExecStart = concatStringsSep " " (
            [
              (lib.getExe' cfg.package "splora-queue")
            ]
            ++ map lib.escapeShellArg (
              queueListenArgs
              ++ [
                "--queue-file"
                cfg.queueFile
              ]
            )
          );
          Restart = "on-failure";
          RestartSec = 5;
          RuntimeDirectory = "splora";
          RuntimeDirectoryMode = "0750";
          RuntimeDirectoryPreserve = true;
          UMask = "0007";
          NoNewPrivileges = true;
          PrivateTmp = true;
          ProtectSystem = "strict";
          ProtectHome = true;
          # Only the queue directory. Sibling .tmp writes work. This must
          # not be the allowlist parent or an indexer dbDir. Queue HTTP
          # never writes the allowlist; splora-import does that.
          ReadWritePaths = [ (dirOf cfg.queueFile) ];
          MemoryDenyWriteExecute = true;
        };
      };
    })

    (mkIf (cfg.popularScripts.enable && popularInstance != null) {
      systemd.services.splora-popular-scripts = {
        description = "splora popular-scripts scan (${cfg.popularScripts.instance})";
        after = [ "splora-${cfg.popularScripts.instance}.service" ];
        serviceConfig = {
          Type = "oneshot";
          User = cfg.user;
          Group = cfg.group;
          ExecStart = concatStringsSep " " (
            [
              (lib.getExe' cfg.package "popular-scripts")
              "--db-dir"
              (lib.escapeShellArg popularInstance.dbDir)
              "--network"
              cliNetwork.${popularInstance.network}
            ]
            ++ optionals (popularInstance.daemonDir != null) [
              "--daemon-dir"
              (lib.escapeShellArg popularInstance.daemonDir)
            ]
            ++ optional (popularInstance.network == "mutinynet") "--magic"
            ++ optional (popularInstance.network == "mutinynet") mutinynetMagic
          );
          StandardOutput = "truncate:${cfg.popularScripts.outputFile}";
          StandardError = "journal";
          NoNewPrivileges = true;
          PrivateTmp = true;
          ProtectSystem = "strict";
          ProtectHome = true;
          ReadOnlyPaths =
            [ popularInstance.dbDir ]
            ++ optional (popularInstance.daemonDir != null) popularInstance.daemonDir;
          ReadWritePaths = [ (dirOf cfg.popularScripts.outputFile) ];
          MemoryDenyWriteExecute = true;
        };
      };

      systemd.timers.splora-popular-scripts = {
        description = "splora popular-scripts timer";
        wantedBy = [ "timers.target" ];
        timerConfig = {
          OnCalendar = cfg.popularScripts.onCalendar;
          Persistent = true;
          Unit = "splora-popular-scripts.service";
        };
      };
    })
  ]);
}
