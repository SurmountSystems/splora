# NixOS module for splora. Import from a flake with:
#   nixosModules.splora = import ./nix/module.nix;
#
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
# No nginx in this module. Public TLS, HTTP/2, and HTTP/3 terminate on the
# Surmount Axum edge (surmount-server). That edge should proxy to the unix
# sockets below and forward Host and X-Forwarded-Proto.
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

  instancePackage = inst: if inst.network == "liquid" then cfg.liquidPackage else cfg.package;

  enabledInstances = filterAttrs (_: inst: inst.enable) cfg.instances;

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

        daemonDir = mkOption {
          type = types.nullOr types.str;
          description = ''
            Bitcoind or elementsd data-directory root passed as --daemon-dir.
            The indexer appends the network subdirectory (testnet3, testnet4,
            signet, liquidv1). Mutinynet uses the bitcoind signet datadir
            under this root. Null is remote JSON-RPC: omit --daemon-dir and
            omit that path from ReadOnlyPaths. Remote mode requires cookieFile
            and daemonRpcAddr. Never put cookie bytes in this option.
          '';
        };

        daemonRpcAddr = mkOption {
          type = types.str;
          description = "JSON-RPC address of bitcoind or elementsd.";
        };

        cookieFile = mkOption {
          type = types.nullOr types.str;
          default = null;
          example = "/var/lib/bitcoind/.cookie";
          description = ''
            Path to the bitcoind or elementsd cookie file, passed as
            --cookie-file. Never put cookie contents in Nix. When null, the
            indexer reads the cookie from daemonDir's network subdirectory.
            Required when daemonDir is null (remote JSON-RPC).
          '';
        };

        jsonrpcImport = mkOption {
          type = types.bool;
          default = false;
          description = ''
            Pass --jsonrpc-import so the indexer uses bitcoind JSON-RPC
            instead of local blk*.dat files. Set true for a remote node.
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
        daemonDir = mkDefault (
          if config.network == "liquid" then "/var/lib/elementsd" else "/var/lib/bitcoind"
        );
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
    after = [ "network.target" ];
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

in
{
  options.services.splora = {
    enable = mkEnableOption "splora indexer, import queue HTTP, and shared allowlist";

    package = mkOption {
      type = types.package;
      default = pkgs.splora;
      defaultText = literalExpression "pkgs.splora";
      description = "splora package for Bitcoin-family instances and the queue service.";
    };

    liquidPackage = mkOption {
      type = types.package;
      default = pkgs.splora-liquid;
      defaultText = literalExpression "pkgs.splora-liquid";
      description = "splora package built with the liquid feature (assetDbPath).";
    };

    user = mkOption {
      type = types.str;
      default = "splora";
      description = "System user for indexer and queue units.";
    };

    group = mkOption {
      type = types.str;
      default = "splora";
      description = "System group for indexer and queue units.";
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

    dbBlockCacheMb = mkOption {
      type = types.ints.positive;
      default = 24;
      description = ''
        Default RocksDB block cache in MiB for new instances. Matches the
        CLI default of 24. REST does not require 4096. Set more if the
        operator wants a larger cache.
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
      default = { };
      example = literalExpression ''
        {
          mainnet.network = "mainnet";
          testnet3.network = "testnet3";
          testnet4.network = "testnet4";
          mutinynet.network = "mutinynet";
          liquid.network = "liquid";
        }
      '';
      description = "One indexer process per network name.";
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
      };
      users.groups.${cfg.group} = { };

      systemd.tmpfiles.rules = [
        "d /var/lib/splora 0750 ${cfg.user} ${cfg.group} -"
        "d /run/splora 0750 ${cfg.user} ${cfg.group} -"
        "d ${dirOf cfg.queueFile} 0750 ${cfg.user} ${cfg.group} -"
        # Type f creates the file only when missing. Age - and no argument
        # leaves an empty allowlist or queue. An empty allowlist means nobody
        # is authorized. The queue directory is not the allowlist parent.
        "f ${cfg.allowNpubsFile} 0640 ${cfg.user} ${cfg.group} -"
        "f ${cfg.queueFile} 0640 ${cfg.user} ${cfg.group} -"
      ]
      ++ optional cfg.popularScripts.enable "d ${dirOf cfg.popularScripts.outputFile} 0750 ${cfg.user} ${cfg.group} -"
      ++ lib.mapAttrsToList (_name: inst: "d ${inst.dbDir} 0750 ${cfg.user} ${cfg.group} -") enabledInstances
      ++ lib.filter (r: r != "") (
        lib.mapAttrsToList (
          _name: inst:
          optionalString (inst.assetDbPath != null)
            "d ${inst.assetDbPath} 0750 ${cfg.user} ${cfg.group} -"
        ) enabledInstances
      );

      systemd.services = mapAttrs' (
        name: inst: nameValuePair "splora-${name}" (indexerService name inst)
      ) enabledInstances;
    }

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
